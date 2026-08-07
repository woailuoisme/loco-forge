use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

/// Excel 文件下载响应（仿 Laravel `Excel::download()` 流式下载 API）
///
/// # 使用方式
///
/// ## 典型控制器场景：查询数据库 → 导出为 xlsx → 浏览器下载
///
/// ```ignore
/// use crate::common::excel::{export::export_to_bytes, response::ExcelResponse};
///
/// async fn export_users(State(ctx): State<AppContext>) -> impl IntoResponse {
///     // 1. 查询数据（替换为实际 ORM 调用）
///     let users = User::find_all(&ctx.db).await?;
///
///     // 2. 映射为实现了 ExcelExport 的导出结构体
///     let rows: Vec<UserExport> = users.into_iter().map(UserExport::from).collect();
///
///     // 3. 序列化为 xlsx 字节流
///     let bytes = export_to_bytes(&rows)?;
///
///     // 4. 返回文件下载响应（Content-Disposition: attachment）
///     Ok(ExcelResponse::download(bytes, "users-2026-08-06.xlsx"))
/// }
/// ```
///
/// ## 动态文件名示例（包含日期戳）
///
/// ```ignore
/// use chrono::Local;
///
/// let filename = format!("orders-{}.xlsx", Local::now().format("%Y%m%d%H%M%S"));
/// Ok(ExcelResponse::download(bytes, filename))
/// ```
///
/// ## 路由注册
///
/// ```ignore
/// // src/controllers/user.rs
/// router.get("/users/export", export_users);
/// ```
enum ExcelResponseBody {
    Bytes(Vec<u8>),
    File(std::path::PathBuf),
}

pub struct ExcelResponse {
    body: ExcelResponseBody,
    filename: String,
}

impl ExcelResponse {
    /// 构造 `.xlsx` 文件内存字节流下载响应（适合小数据量导出）
    ///
    /// # 参数
    /// - `bytes`: 由 [`export_to_bytes`](crate::common::excel::export::export_to_bytes) 生成的 Excel 字节流
    /// - `filename`: 浏览器收到的文件名（如 `"users-2026-08-06.xlsx"`）
    pub fn download(bytes: Vec<u8>, filename: impl Into<String>) -> Self {
        Self {
            body: ExcelResponseBody::Bytes(bytes),
            filename: filename.into(),
        }
    }

    /// 构造 `.xlsx` 海量大文件物理/临时文件流式下载响应（适合 100 万+ 数据量导出）
    ///
    /// # 参数
    /// - `path`: 本地物理或临时 Excel `.xlsx` 文件路径
    /// - `filename`: 浏览器收到的文件名
    pub fn download_file(path: impl AsRef<std::path::Path>, filename: impl Into<String>) -> Self {
        Self {
            body: ExcelResponseBody::File(path.as_ref().to_path_buf()),
            filename: filename.into(),
        }
    }
}

impl IntoResponse for ExcelResponse {
    fn into_response(self) -> Response {
        let disposition_value = format!("attachment; filename=\"{}\"", self.filename);

        let body = match self.body {
            ExcelResponseBody::Bytes(b) => Body::from(b),
            ExcelResponseBody::File(path) => match std::fs::File::open(&path) {
                Ok(file) => {
                    let tokio_file = tokio::fs::File::from_std(file);
                    let stream = tokio_util::io::ReaderStream::new(tokio_file);
                    Body::from_stream(stream)
                }
                Err(e) => {
                    tracing::error!("打开 Excel 下载文件失败 [{:?}]: {:?}", path, e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "无法读取文件").into_response();
                }
            },
        };

        let mut response = Response::new(body);
        *response.status_mut() = StatusCode::OK;

        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&disposition_value).unwrap_or_else(|_| HeaderValue::from_static("attachment")),
        );
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        );

        response
    }
}
