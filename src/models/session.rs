use base64::prelude::*;
use chrono::{Duration, Utc};
use loco_rs::prelude::*;
use rand::RngExt;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, IntoActiveModel, QueryFilter, QueryOrder,
};
use uuid::Uuid;

pub use super::_entities::session::{ActiveModel, Column, Entity, Model};
pub type Session = Entity;

/// 默认会话有效期（7 天）
pub const SESSION_TTL_DAYS: i64 = 7;
/// 滑动窗口检测阈值：距离上次续期经过超过 1 天则自动延长
pub const SESSION_SLIDE_THRESHOLD_SECS: i64 = 86400;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut this = self;
        let now = Utc::now().into();
        if insert {
            if this.id.is_not_set() {
                this.id = Set(Uuid::now_v7().to_string());
            }
            if this.created_at.is_not_set() {
                this.created_at = Set(now);
            }
            this.updated_at = Set(now);
        } else if this.updated_at.is_unchanged() {
            this.updated_at = Set(now);
        }
        Ok(this)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ① 单实例读写方法 (Model)
// ─────────────────────────────────────────────────────────────────────────────
impl Model {
    /// 会话是否已过期
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        self.expires_at < now
    }

    /// 7 天滑动窗口续期（若距离到期时间不足 6 天则自动延长至 7 天）
    pub async fn touch_sliding_window(&self, db: &DatabaseConnection) -> ModelResult<Self> {
        let now = Utc::now();
        let target_expiry = now + Duration::days(SESSION_TTL_DAYS);
        let threshold = now + Duration::days(SESSION_TTL_DAYS - 1);

        let current_expiry: chrono::DateTime<Utc> = self.expires_at.with_timezone(&Utc);
        if current_expiry < threshold {
            let mut active = self.clone().into_active_model();
            active.expires_at = Set(target_expiry.into());
            let updated = active.update(db).await?;
            Ok(updated)
        } else {
            Ok(self.clone())
        }
    }

    /// 设置会话当前激活的组织
    pub async fn set_active_organization<C>(
        &self,
        db: &C,
        org_id: Option<String>,
    ) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let mut active = self.clone().into_active_model();
        active.active_organization_id = Set(org_id);
        active.updated_at = Set(Utc::now().into());
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 清除会话的模拟登录状态
    pub async fn clear_impersonation(&self, db: &DatabaseConnection) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.impersonated_by = Set(None);
        active.updated_at = Set(Utc::now().into());
        let updated = active.update(db).await?;
        Ok(updated)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ② ActiveModel 钩子与字段行为
// ─────────────────────────────────────────────────────────────────────────────
impl ActiveModel {}

// ─────────────────────────────────────────────────────────────────────────────
// ③ 集合级查询与创建 (Entity)
// ─────────────────────────────────────────────────────────────────────────────
impl Entity {
    /// 生成 32 字节高熵安全的 URL-Safe Token
    #[must_use]
    pub fn generate_token() -> String {
        let mut bytes = [0u8; 32];
        rand::rng().fill(&mut bytes);
        BASE64_URL_SAFE_NO_PAD.encode(bytes)
    }

    /// 创建持久化会话记录
    pub async fn create_session<C>(
        db: &C,
        user_id: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> ModelResult<Model>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();
        let expires_at = (now + Duration::days(SESSION_TTL_DAYS)).into();
        let token = Self::generate_token();

        let active = ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            token: Set(token),
            expires_at: Set(expires_at),
            ip_address: Set(ip_address),
            user_agent: Set(user_agent),
            active_organization_id: Set(None),
            impersonated_by: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };
        let session = active.insert(db).await?;
        Ok(session)
    }

    /// 管理员模拟指定用户签发专用会话
    pub async fn create_impersonated_session(
        db: &DatabaseConnection,
        target_user_id: &str,
        admin_user_id: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> ModelResult<Model> {
        let now = Utc::now();
        let expires_at = (now + Duration::days(SESSION_TTL_DAYS)).into();
        let token = Self::generate_token();

        let active = ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            user_id: Set(target_user_id.to_string()),
            token: Set(token),
            expires_at: Set(expires_at),
            ip_address: Set(ip_address),
            user_agent: Set(user_agent),
            active_organization_id: Set(None),
            impersonated_by: Set(Some(admin_user_id.to_string())),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };
        let session = active.insert(db).await?;
        Ok(session)
    }

    /// 根据 Session Token 查找有效活跃会话
    pub async fn find_valid_by_token(
        db: &DatabaseConnection,
        token: &str,
    ) -> ModelResult<Option<Model>> {
        let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let session = Self::find()
            .filter(Column::Token.eq(token))
            .filter(Column::ExpiresAt.gt(now))
            .one(db)
            .await?;
        Ok(session)
    }

    /// 获取用户所有未过期的活跃会话列表
    pub async fn find_active_by_user(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> ModelResult<Vec<Model>> {
        let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let sessions = Self::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ExpiresAt.gt(now))
            .order_by_desc(Column::CreatedAt)
            .all(db)
            .await?;
        Ok(sessions)
    }

    /// 吊销指定 Token 的会话
    pub async fn revoke_by_token(db: &DatabaseConnection, token: &str) -> ModelResult<bool> {
        let res = Self::delete_many()
            .filter(Column::Token.eq(token))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// 吊销用户指定 ID 的会话（限本人会话）
    pub async fn revoke_by_id_for_user(
        db: &DatabaseConnection,
        session_id: &str,
        user_id: &str,
    ) -> ModelResult<bool> {
        let res = Self::delete_many()
            .filter(Column::Id.eq(session_id))
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// 吊销当前用户除当前 Token 之外的所有会话
    pub async fn revoke_other_sessions(
        db: &DatabaseConnection,
        user_id: &str,
        current_token: &str,
    ) -> ModelResult<u64> {
        let res = Self::delete_many()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Token.ne(current_token))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }

    /// 吊销当前用户的所有会话
    pub async fn revoke_all_sessions(db: &impl ConnectionTrait, user_id: &str) -> ModelResult<u64> {
        let res = Self::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }

    /// 清理所有已过期的会话记录
    pub async fn delete_expired(db: &DatabaseConnection) -> ModelResult<u64> {
        let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let res = Self::delete_many()
            .filter(Column::ExpiresAt.lt(now))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }
}
