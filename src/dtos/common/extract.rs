//! 自定义 Axum Extractor 扩展模块
//!
//! 提供请求自动反序列化与校验的智能提取器。

use axum::{
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use validator::Validate;

/// 自动 JSON 参数解析与格式校验 Extractor [`ValidJson<T>`]
///
/// 提取客户端提交的 JSON Body，并自动调用 `validator::Validate::validate` 进行校验。
///
/// # 响应格式规范 (Response Formats)
///
/// - **反序列化失败（HTTP 400 Bad Request）**：当 JSON 语法错误或字段类型无法匹配时返回
///   ```json
///   {
///     "success": false,
///     "code": 400,
///     "message": "Invalid JSON payload: ...",
///     "errors": null
///   }
///   ```
///
/// - **业务校验失败（HTTP 422 Unprocessable Entity）**：符合 Loco 规范 (`https://loco.rs/docs/how-to/validate-requests/`) 返回结构化 `errors` 映射：
///   ```json
///   {
///     "success": false,
///     "code": 422,
///     "message": "Invalid request payload parameters",
///     "errors": {
///       "title": [
///         {
///           "code": "length",
///           "message": "title must be at least 3 characters",
///           "params": { "min": 3, "value": "ab" }
///         }
///       ],
///       "email": [
///         {
///           "code": "email",
///           "message": null,
///           "params": { "value": "not-an-email" }
///         }
///       ]
///     }
///   }
///   ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidJson<T>
where
    T: DeserializeOwned + Validate + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(data) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|err| {
                super::format_res::bad_request(format!("Invalid JSON payload: {err}"))
                    .unwrap_or_else(|_| (axum::http::StatusCode::BAD_REQUEST, "Invalid JSON").into_response())
            })?;

        if let Err(validation_errors) = data.validate() {
            let details = serde_json::to_value(&validation_errors).ok();
            return Err(
                super::format_res::validation_error_with_details("Invalid request payload parameters", details)
                    .unwrap_or_else(|_| (axum::http::StatusCode::UNPROCESSABLE_ENTITY, "Validation Error").into_response()),
            );
        }

        Ok(Self(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::post, Router};
    use serde::{Deserialize, Serialize};
    use tower::ServiceExt;

    #[derive(Debug, Serialize, Deserialize, Validate)]
    struct DummyPayload {
        #[validate(length(min = 3, message = "Name must be at least 3 chars"))]
        name: String,
    }

    async fn dummy_handler(ValidJson(payload): ValidJson<DummyPayload>) -> Response {
        (axum::http::StatusCode::OK, payload.name).into_response()
    }

    #[tokio::test]
    async fn test_valid_json_success() {
        let app = Router::new().route("/test", post(dummy_handler));

        let req = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name": "Alice"}"#))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_valid_json_validation_failure_422() {
        let app = Router::new().route("/test", post(dummy_handler));

        let req = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name": "ab"}"#))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    }
}
