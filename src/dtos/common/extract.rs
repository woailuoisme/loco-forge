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
                use axum::extract::rejection::JsonRejection;
                // api.md §2.4：字段语义/结构不合法返回 422；纯语法错误或 Content-Type 缺失返回 400
                match err {
                    JsonRejection::JsonDataError(e) => super::response::ApiError::validation(
                        format!("Invalid request payload: {e}"),
                        None,
                    )
                    .into_response(),
                    other => super::response::ApiError::bad_request(format!(
                        "Invalid JSON payload: {other}"
                    ))
                    .into_response(),
                }
            })?;

        if let Err(validation_errors) = data.validate() {
            return Err(format_validation_rejection(
                "Invalid request payload parameters",
                &validation_errors,
            ));
        }

        Ok(Self(data))
    }
}

/// 自动 Query 参数解析与格式校验 Extractor [`ValidQuery<T>`]
///
/// 提取客户端 URL Query 参数，并自动调用 `validator::Validate::validate` 进行校验。
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidQuery<T>(pub T);

impl<T, S> axum::extract::FromRequestParts<S> for ValidQuery<T>
where
    T: DeserializeOwned + Validate + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(data) =
            axum::extract::Query::<T>::from_request_parts(parts, state)
                .await
                .map_err(|err| {
                    super::response::ApiError::bad_request(format!(
                        "Invalid Query parameters: {err}"
                    ))
                    .into_response()
                })?;

        if let Err(validation_errors) = data.validate() {
            return Err(format_validation_rejection(
                "Invalid request query parameters",
                &validation_errors,
            ));
        }

        Ok(Self(data))
    }
}

/// 把 validator 的结构化错误压平成 api.md §5.2 要求的 `{字段: ["文案"]}` 形状
fn format_validation_rejection(msg: &str, errors: &validator::ValidationErrors) -> Response {
    let mut map = serde_json::Map::new();
    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<serde_json::Value> = field_errors
            .iter()
            .filter_map(|e| e.message.as_ref())
            .map(|m| serde_json::Value::String(m.to_string()))
            .collect();
        if !messages.is_empty() {
            map.insert(field.to_string(), serde_json::Value::Array(messages));
        }
    }

    let details = if map.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(map))
    };
    super::response::ApiError::validation(msg, details).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::Request,
        routing::{get, post},
    };
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

    async fn dummy_query_handler(ValidQuery(payload): ValidQuery<DummyPayload>) -> Response {
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

    #[tokio::test]
    async fn test_valid_query_success() {
        let app = Router::new().route("/query", get(dummy_query_handler));

        let req = Request::builder()
            .method("GET")
            .uri("/query?name=Bob123")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_valid_query_validation_failure_422() {
        let app = Router::new().route("/query", get(dummy_query_handler));

        let req = Request::builder()
            .method("GET")
            .uri("/query?name=b")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    }
}
