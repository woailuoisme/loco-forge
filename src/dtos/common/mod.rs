//! 通用数据传输对象与辅助类型 (Common DTOs & Helpers)
//!
//! 此模块为项目全局公共 DTO 基础设施，按职责划分为若干子模块：
//!
//! | 子模块            | 职责说明                                |
//! |-------------------|-----------------------------------------|
//! | [`pagination`]    | 分页元数据 [`PageMeta`]、分页响应容器 [`PaginatedData`] |
//! | [`response`]      | 统一 API 错误信封 [`ApiError`]           |
//! | [`format_res`]    | Controller 响应快速格式化辅助函数        |
//!
//! # 对外 Re-export
//!
//! 所有常用类型均在此 `mod.rs` 做扁平化 re-export，调用方只需：
//! ```rust,ignore
//! use crate::dtos::common::{ApiError, format_res};
//! ```

pub mod extract;
pub mod format_res;
pub mod pagination;
pub mod rbac;
pub mod response;

// 扁平化 re-export：对外公开常用类型
pub use extract::{ValidJson, ValidQuery};
pub use pagination::{PageMeta, PaginatedData, PaginatedResource, PaginationRequest};
pub use rbac::{AdminUser, AuthenticatedUser};
pub use response::{ApiError, error_code_for, status_for_code};

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode, response::Response};

    use super::format_res;
    use crate::common::consts::biz_code;

    async fn parse_response(res: Response) -> (StatusCode, serde_json::Value) {
        let status = res.status();
        if status == StatusCode::NO_CONTENT {
            return (status, serde_json::Value::Null);
        }
        let body_bytes = to_bytes(res.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes).expect("Invalid JSON response");
        (status, json)
    }

    // =========================================================================
    // format_res — 2xx (裸 JSON 契约)
    // =========================================================================

    #[tokio::test]
    async fn test_format_res_success_returns_bare_json() {
        let (status, json) = parse_response(format_res::ok("payload").unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        // 裸 JSON：无 success / code / message 外壳
        assert_eq!(json, serde_json::json!("payload"));

        let (status, json) =
            parse_response(format_res::created(serde_json::json!({"id": 7})).unwrap()).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["id"], 7);
        assert!(json.get("success").is_none(), "不得包含 success 外壳");

        let (status, json) = parse_response(
            format_res::accepted(serde_json::json!({"taskId": "t1", "status": "PENDING"})).unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(json["taskId"], "t1");

        let (status, _) = parse_response(format_res::no_content().unwrap()).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_format_res_page_envelope() {
        let (status, json) =
            parse_response(format_res::page(vec!["a", "b"], 1, 10, 25).unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        // 全系统唯一分页信封
        assert_eq!(json["items"], serde_json::json!(["a", "b"]));
        assert_eq!(json["meta"]["page"], 1);
        assert_eq!(json["meta"]["pageSize"], 10);
        assert_eq!(json["meta"]["total"], 25);
        assert_eq!(json["meta"]["totalPages"], 3);
        // 无 success / code / message / extras 残留
        assert!(json.get("success").is_none());
        assert!(json.get("data").is_none());
        assert!(json["meta"].get("extras").is_none());
        assert!(json.get("extras").is_none());
    }

    #[tokio::test]
    async fn test_format_res_page_empty_items_is_array() {
        let (_, json) =
            parse_response(format_res::page(Vec::<String>::new(), 1, 20, 0).unwrap()).await;
        // 空值保底：items 必须是 []，严禁 null
        assert_eq!(json["items"], serde_json::json!([]));
        assert!(!json["items"].is_null());
        assert_eq!(json["meta"]["total"], 0);
    }

    // =========================================================================
    // format_res — 4xx / 5xx (嵌套 error 信封)
    // =========================================================================

    #[rstest::rstest]
    #[case::bad_request(
        format_res::bad_request("bad req"),
        StatusCode::BAD_REQUEST,
        "BAD_REQUEST",
        "bad req"
    )]
    #[case::bad_request_default(
        format_res::bad_request_default(),
        StatusCode::BAD_REQUEST,
        "BAD_REQUEST",
        "请求参数无效"
    )]
    #[case::unauthorized(
        format_res::unauthorized("unauth"),
        StatusCode::UNAUTHORIZED,
        "AUTH_TOKEN_INVALID",
        "unauth"
    )]
    #[case::forbidden(
        format_res::forbidden("forbidden"),
        StatusCode::FORBIDDEN,
        "FORBIDDEN",
        "forbidden"
    )]
    #[case::not_found(
        format_res::not_found("not found"),
        StatusCode::NOT_FOUND,
        "NOT_FOUND",
        "not found"
    )]
    #[case::route_not_found(
        format_res::route_not_found(),
        StatusCode::NOT_FOUND,
        "NOT_FOUND",
        "请求的接口路径不存在"
    )]
    #[case::method_not_allowed(
        format_res::method_not_allowed("method err"),
        StatusCode::METHOD_NOT_ALLOWED,
        "METHOD_NOT_ALLOWED",
        "method err"
    )]
    #[case::conflict(
        format_res::conflict("conflict"),
        StatusCode::CONFLICT,
        "CONFLICT",
        "conflict"
    )]
    #[case::validation(
        format_res::validation_error("invalid"),
        StatusCode::UNPROCESSABLE_ENTITY,
        "VALIDATION_FAILED",
        "invalid"
    )]
    #[case::too_many(
        format_res::too_many_requests("rate limit"),
        StatusCode::TOO_MANY_REQUESTS,
        "RATE_LIMITED",
        "rate limit"
    )]
    #[case::internal(
        format_res::internal_error("internal err"),
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_SERVER_ERROR",
        "internal err"
    )]
    #[case::unavailable(
        format_res::service_unavailable("down"),
        StatusCode::SERVICE_UNAVAILABLE,
        "SERVICE_UNAVAILABLE",
        "down"
    )]
    #[tokio::test]
    async fn test_format_res_error_helpers(
        #[case] res_result: loco_rs::prelude::Result<Response>,
        #[case] expected_status: StatusCode,
        #[case] expected_code: &str,
        #[case] expected_msg: &str,
    ) {
        let (status, json) = parse_response(res_result.unwrap()).await;
        assert_eq!(status, expected_status);
        // 扁平错误体：{ code, message }，无 success 外壳、无 errors 字段
        assert!(json.get("success").is_none());
        assert!(json.get("error").is_none());
        assert_eq!(json["code"], expected_code);
        assert_eq!(json["message"], expected_msg);
        assert!(json.get("errors").is_none());
    }

    #[tokio::test]
    async fn test_format_res_error_with_details() {
        let details = serde_json::json!({ "field": ["is required"] });
        let (status, json) = parse_response(
            format_res::validation_error_with_details("val error", details.clone()).unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["code"], "VALIDATION_FAILED");
        assert_eq!(json["errors"], details);

        let (status, json) = parse_response(
            format_res::bad_request_with_details("bad req", details.clone()).unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], "BAD_REQUEST");
        assert_eq!(json["errors"], details);
    }

    #[tokio::test]
    async fn test_format_res_internal_error_with_debug_does_not_leak() {
        let (status, json) = parse_response(
            format_res::internal_error_with_debug(
                "服务暂时不可用",
                "EyreError: connection reset",
                &["stack line 1".to_string()],
            )
            .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["code"], "INTERNAL_SERVER_ERROR");
        assert_eq!(json["message"], "服务暂时不可用");
        // 生产脱敏：异常名与 trace 不得进入响应体
        assert!(json.get("trace").is_none());
        assert!(!json.to_string().contains("EyreError"));
        assert!(!json.to_string().contains("stack line 1"));
    }

    // =========================================================================
    // format_res — 600..999 custom / biz error codes
    // =========================================================================

    #[rstest::rstest]
    #[case(601, "frozen user", StatusCode::BAD_REQUEST)]
    #[case(700, "payment required", StatusCode::BAD_REQUEST)]
    #[case(888, "stock exhausted", StatusCode::BAD_REQUEST)]
    #[tokio::test]
    async fn test_format_res_custom_errors(
        #[case] custom_code: u32,
        #[case] msg: &str,
        #[case] expected_status: StatusCode,
    ) {
        let (status, json) =
            parse_response(format_res::custom_error(custom_code, msg).unwrap()).await;
        assert_eq!(status, expected_status);
        assert_eq!(json["code"], format!("BIZ_{custom_code}"));
        assert_eq!(json["message"], msg);

        let details = serde_json::json!({ "reason": "blocked" });
        let (status_det, json_det) = parse_response(
            format_res::custom_error_with_details(custom_code, msg, details.clone()).unwrap(),
        )
        .await;
        assert_eq!(status_det, expected_status);
        assert_eq!(json_det["code"], format!("BIZ_{custom_code}"));
        assert_eq!(json_det["errors"], details);
    }

    #[tokio::test]
    async fn test_format_res_biz_helpers() {
        let (status, json) =
            parse_response(format_res::biz(biz_code::USER_FROZEN, "账号已被冻结").unwrap()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], "BIZ_601");
        assert_eq!(json["message"], "账号已被冻结");

        let details = serde_json::json!({ "required": 100 });
        let (status, json) = parse_response(
            format_res::biz_with_details(
                biz_code::INSUFFICIENT_BALANCE,
                "余额不足",
                details.clone(),
            )
            .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], "BIZ_602");
        assert_eq!(json["errors"], details);

        let (status, json) = parse_response(
            format_res::biz_with_status(
                StatusCode::UNPROCESSABLE_ENTITY,
                biz_code::USER_FROZEN,
                "账号冻结",
            )
            .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["code"], "BIZ_601");
    }
}
