//! 管理端用户服务 (Admin User Service)
//!
//! 业务管理域的用户运维：分页检索、详情、创建、编辑、软删注销、Excel 批量导入导出。
//!
//! **职责边界**：封禁 / 解封 / 角色变更属于认证域操作，统一复用
//! [`crate::services::AuthAdminService::admin_ban_user`] 等方法，本模块不重复实现。

use bytes::Bytes;
use loco_rs::prelude::*;

use crate::{
    common::excel::{
        Excel, ExportOptions, ImportResult, export_chunked_to_file, spawn_excel_blocking,
    },
    dtos::{
        admin::{
            AdminCreateUserRequest, AdminListUsersQuery, AdminUpdateUserRequest, AdminUserResource,
            NullableString, UserImportError, UserImportRow, UserImportSummary,
        },
        common::{PageMeta, PaginatedResource},
    },
    errors::{AppError, AppResult},
    models::user::{AdminUserQueryParams, Entity as UserEntity},
};

// Excel 的 `ExcelExport` / `ExcelImport` trait 实现位于 `crate::exports::user`
// 与 `crate::imports::user` (二者已在 lib.rs 声明为 pub mod)，此处只需按类型使用。

/// 管理端用户服务
pub struct AdminUserService;

impl AdminUserService {
    /// 分页检索用户列表，支持关键词搜索、角色 / 状态 / 封禁过滤与排序。
    pub async fn list(
        db: &DatabaseConnection,
        query: &AdminListUsersQuery,
    ) -> AppResult<PaginatedResource<AdminUserResource>> {
        let pagination = query.pagination();
        let params = AdminUserQueryParams {
            page_size: pagination.page_size(),
            offset: pagination.offset(),
            search: query.search.as_deref(),
            role: query.role.as_deref(),
            status: query.status.as_deref(),
            banned: query.banned,
            sort_by: query.sort_by.as_deref(),
            include_deleted: query.include_deleted,
        };

        let (users, total) = UserEntity::find_all_admin_paginated(db, &params).await?;

        Ok(PaginatedResource {
            items: users.into_iter().map(AdminUserResource::from).collect(),
            meta: PageMeta::new(pagination.page(), pagination.page_size(), total),
        })
    }

    /// 查询单个用户详情。
    ///
    /// `include_deleted = true` 时连已注销用户一并返回，供后台审计查看。
    pub async fn get_one(
        db: &DatabaseConnection,
        id: &str,
        include_deleted: bool,
    ) -> AppResult<AdminUserResource> {
        let user = if include_deleted {
            UserEntity::find_by_id_with_deleted(db, id).await?
        } else {
            UserEntity::find_by_id(db, id).await?
        };

        user.map(AdminUserResource::from)
            .ok_or_else(|| AppError::not_found("用户不存在"))
    }

    /// 创建用户 (无密码凭证)。
    ///
    /// 新用户需由本人通过「忘记密码」或 Magic Link 完成首次登录。
    /// 邮箱 / 用户名 / 手机号任一冲突即拒绝 (不静默忽略)。
    pub async fn create(
        db: &DatabaseConnection,
        params: &AdminCreateUserRequest,
    ) -> AppResult<AdminUserResource> {
        if UserEntity::find_by_email(db, &params.email)
            .await?
            .is_some()
        {
            return Err(AppError::conflict("邮箱已被占用"));
        }
        if let Some(username) = params.username.as_deref()
            && UserEntity::find_by_username(db, username).await?.is_some()
        {
            return Err(AppError::conflict("用户名已被占用"));
        }
        if let Some(phone) = params.phone_number.as_deref()
            && UserEntity::find_by_phone_number(db, phone).await?.is_some()
        {
            return Err(AppError::conflict("手机号已被占用"));
        }

        let user = UserEntity::create_user(
            db,
            &params.email,
            &params.name,
            params.role.as_deref(),
            params.image.clone(),
            params.username.clone(),
        )
        .await?;

        if let Some(phone) = params.phone_number.as_deref() {
            let updated = user.set_phone_number(db, phone, false).await?;
            tracing::info!(user_id = %updated.id, email = %updated.email, "管理端创建用户成功");
            return Ok(AdminUserResource::from(updated));
        }

        tracing::info!(user_id = %user.id, email = %user.email, "管理端创建用户成功");
        Ok(AdminUserResource::from(user))
    }

    /// 局部更新用户资料 (PATCH 三态语义)。
    pub async fn update(
        db: &DatabaseConnection,
        id: &str,
        params: &AdminUpdateUserRequest,
    ) -> AppResult<AdminUserResource> {
        let user = UserEntity::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("用户不存在"))?;

        // 用户名变更时需校验唯一性 (仅当传入具体新值，显式置空无需校验)
        if let Some(NullableString::Value(username)) = params.username.as_ref()
            && let Some(occupied) = UserEntity::find_by_username(db, username).await?
            && occupied.id != user.id
        {
            return Err(AppError::conflict("用户名已被占用"));
        }

        let updated = user
            .update_profile(
                db,
                params.name.clone(),
                params.role.clone(),
                params.status.clone(),
                params.username.clone().map(NullableString::into_option),
                params.image.clone().map(NullableString::into_option),
                params.email_verified,
            )
            .await?;

        tracing::info!(user_id = %updated.id, "管理端更新用户成功");
        Ok(AdminUserResource::from(updated))
    }

    /// 软删 (注销) 用户。
    ///
    /// 仅写入 `deleted_at` 并撤销全部活跃会话；数据完整保留供审计。
    /// 因唯一索引带 `WHERE deleted_at IS NULL`，邮箱 / 用户名 / 手机号随即释放。
    pub async fn soft_delete(
        db: &DatabaseConnection,
        current_admin_id: &str,
        id: &str,
    ) -> AppResult<()> {
        if id == current_admin_id {
            return Err(AppError::bad_request("不能注销自己的账号"));
        }

        let user = UserEntity::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("用户不存在"))?;

        user.soft_delete(db).await?;
        crate::models::session::Entity::revoke_all_sessions(db, &user.id).await?;

        tracing::info!(user_id = %user.id, email = %user.email, "管理端软删用户成功");
        Ok(())
    }

    /// 导出用户列表为 Excel 临时文件 (流式，支持海量数据)。
    ///
    /// 采用 [`export_chunked_to_file`]：游标分批 fetch + `constant_memory` 流式落盘，
    /// 内存占用维持在几 MB；行数超过单 Sheet 上限时自动拆分为 `Sheet2`、`Sheet3`…
    ///
    /// 返回 `(临时文件路径, 导出总行数)`，由调用方以
    /// [`ExcelResponse::download_file`] 流式回传给客户端并在响应结束后自行清理。
    ///
    /// [`ExportOptions`]: crate::common::excel::ExportOptions
    /// [`ExcelResponse::download_file`]: crate::common::excel::ExcelResponse::download_file
    pub async fn export_excel(
        db: &DatabaseConnection,
        query: &AdminListUsersQuery,
    ) -> AppResult<(std::path::PathBuf, usize)> {
        // 查询闭包运行在 spawn_blocking 线程 (非 runtime 线程)，用捕获的 Handle 驱动 async 查询
        let db = db.clone();
        let handle = tokio::runtime::Handle::current();

        // 闭包要求 'static，故将查询条件转为 owned
        let search = query.search.clone();
        let role = query.role.clone();
        let status = query.status.clone();
        let banned = query.banned;
        let sort_by = query.sort_by.clone();
        let include_deleted = query.include_deleted;

        let tmp_path = std::env::temp_dir().join(format!(
            "locos-users-{}.xlsx",
            uuid::Uuid::now_v7().simple()
        ));
        let write_path = tmp_path.clone();

        let total = spawn_excel_blocking(move || {
            let options = ExportOptions::default();
            export_chunked_to_file::<AdminUserResource, _>(
                |page, page_size| {
                    let db = db.clone();
                    let search = search.clone();
                    let role = role.clone();
                    let status = status.clone();
                    let sort_by = sort_by.clone();
                    handle
                        .block_on(async move {
                            let params = AdminUserQueryParams {
                                page_size: page_size as u64,
                                offset: ((page - 1) * page_size) as u64,
                                search: search.as_deref(),
                                role: role.as_deref(),
                                status: status.as_deref(),
                                banned,
                                sort_by: sort_by.as_deref(),
                                include_deleted,
                            };
                            UserEntity::find_all_admin_paginated(&db, &params)
                                .await
                                .map(|(users, _)| {
                                    users.into_iter().map(AdminUserResource::from).collect()
                                })
                                .map_err(|e| {
                                    tracing::error!("导出用户数据查询失败: {e}");
                                    Error::InternalServerError
                                })
                        })
                        .map_err(|e| {
                            tracing::error!("导出分块查询失败: {e:?}");
                            Error::InternalServerError
                        })
                },
                &write_path,
                &options,
            )
        })
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Excel 任务执行失败");
            AppError::internal("Internal task error")
        })?;

        tracing::info!(rows = total, "管理端导出用户列表成功");
        Ok((tmp_path, total))
    }

    /// 从 Excel 批量导入用户 (仅创建语义)。
    ///
    /// 邮箱已存在、格式非法或校验失败的行会计入 `errors` 明细并跳过，
    /// 不影响其余合法行落库。
    pub async fn import_excel(
        db: &DatabaseConnection,
        bytes: Bytes,
    ) -> AppResult<UserImportSummary> {
        // calamine 解析为 CPU 密集型同步操作，放入阻塞线程池避免阻塞 Tokio worker
        let result: ImportResult<UserImportRow> =
            spawn_excel_blocking(move || Excel::import::<UserImportRow>(bytes))
                .await
                .map_err(|e| AppError::bad_request(format!("Excel 解析失败: {e}")))?;

        let mut created = 0_u64;
        // 解析 / 校验阶段产生的错误已带精确 Excel 行号
        let mut errors: Vec<UserImportError> = result
            .errors
            .iter()
            .map(|e| UserImportError {
                line: e.line as u64,
                column: e.column.clone(),
                raw_value: e.raw_value.clone(),
                message: e.message.clone(),
            })
            .collect();

        for row in &result.valid_rows {
            match Self::create(
                db,
                &AdminCreateUserRequest {
                    email: row.email.clone(),
                    name: row.name.clone(),
                    role: row.role.clone(),
                    username: row.username.clone(),
                    phone_number: row.phone_number.clone(),
                    image: None,
                },
            )
            .await
            {
                Ok(_) => created += 1,
                Err(err) => errors.push(UserImportError {
                    line: 0, // 落库冲突行无法回溯原 Excel 行号，以 0 表示
                    column: "email".to_string(),
                    raw_value: row.email.clone(),
                    message: err.message().to_string(),
                }),
            }
        }

        let summary = UserImportSummary {
            created,
            failed: errors.len() as u64,
            errors,
        };

        tracing::info!(
            created = summary.created,
            failed = summary.failed,
            "管理端导入用户完成"
        );
        Ok(summary)
    }
}

// Excel 的 `ExcelExport` / `ExcelImport` trait 实现分别定义在
// `src/exports/user.rs` 与 `src/imports/user.rs`，本模块只负责编排调用。
