//! 通用数据传输对象与辅助类型 (Common DTOs & Helpers)
//!
//! 此模块为项目全局公共 DTO 基础设施，按职责划分为若干子模块：
//!
//! | 子模块            | 职责说明                                |
//! |-------------------|-----------------------------------------|
//! | [`pagination`]    | 分页元数据 [`PageMeta`]、分页响应容器 [`PaginatedData`] |
//! | [`response`]      | 统一 API 响应结构 [`ApiResponse`]        |
//! | [`format_res`]    | Controller 响应快速格式化辅助函数        |
//!
//! # 对外 Re-export
//!
//! 所有常用类型均在此 `mod.rs` 做扁平化 re-export，调用方只需：
//! ```rust,ignore
//! use crate::dtos::common::{ApiResponse, ApiError, format_res};
//! ```

pub mod extract;
pub mod format_res;
pub mod pagination;
pub mod rbac;
pub mod response;

// 扁平化 re-export：对外公开常用类型
pub use extract::ValidJson;
pub use pagination::{Page, PageMeta, PaginatedData, PaginatedResource, PaginationRequest};
pub use rbac::{AdminUser, AuthenticatedUser};
pub use response::{ApiError, ApiResponse};

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode, response::Response};
    use rstest::rstest;

    use super::{ApiResponse, format_res};
    use crate::common::consts::biz_code;

    async fn parse_response(res: Response) -> (StatusCode, serde_json::Value) {
        let status = res.status();
        let body_bytes = to_bytes(res.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes).expect("Invalid JSON response");
        (status, json)
    }

    // =========================================================================
    // PageMeta
    // =========================================================================

    #[test]
    fn test_page_meta_calculation() {
        use super::PageMeta;

        let meta_zero = PageMeta::new(1, 10, 0);
        assert_eq!(meta_zero.last_page, 1);
        assert_eq!(meta_zero.total, 0);

        let meta_exact = PageMeta::new(1, 10, 20);
        assert_eq!(meta_exact.last_page, 2);

        let meta_ceil = PageMeta::new(1, 10, 25);
        assert_eq!(meta_ceil.last_page, 3);
    }

    // =========================================================================
    // ApiResponse
    // =========================================================================

    #[test]
    fn test_api_response_serialization() {
        let ok_res = ApiResponse::ok("hello");
        assert!(ok_res.success);
        assert_eq!(ok_res.code, 200);
        assert_eq!(ok_res.message, "操作成功完成");
        assert_eq!(ok_res.data, Some("hello"));

        let serialized = serde_json::to_string(&ok_res).unwrap();
        // 验证 JSON 字段排布顺序: success -> code -> message -> data
        assert!(serialized.starts_with(r#"{"success":true,"code":200,"message":"操作成功完成""#));

        let empty_res = ApiResponse::success_empty("done");
        assert!(empty_res.success);
        assert_eq!(empty_res.data, None);
        assert_eq!(empty_res.errors, None);

        let err_res = ApiResponse::error_simple(404, "not found");
        assert!(!err_res.success);
        assert_eq!(err_res.code, 404);
        assert_eq!(err_res.message, "not found");
        assert_eq!(err_res.data, None);
    }

    // =========================================================================
    // format_res — 2xx
    // =========================================================================

    #[tokio::test]
    async fn test_format_res_2xx_responses() {
        let (status, json) = parse_response(format_res::ok("payload").unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["success"], true);
        assert_eq!(json["code"], 200);
        assert_eq!(json["data"], "payload");

        let (status, json) = parse_response(format_res::ok_msg("payload", "msg_ok").unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"], "msg_ok");

        let (status, json) = parse_response(format_res::created("created_item").unwrap()).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["code"], 201);
        assert_eq!(json["message"], "创建成功");

        let (status, json) =
            parse_response(format_res::created_msg("created_item", "item created").unwrap()).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["code"], 201);
        assert_eq!(json["message"], "item created");

        let (status, json) = parse_response(format_res::accepted_msg("accepted").unwrap()).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(json["code"], 202);
        assert_eq!(json["message"], "accepted");

        let (status, json) = parse_response(format_res::empty_msg("deleted").unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["code"], 200);
        assert_eq!(json["message"], "deleted");
        assert!(json["data"].is_null());

        let (status, json) =
            parse_response(format_res::page(vec!["a", "b"], 1, 10, 25).unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["data"]["items"], serde_json::json!(["a", "b"]));
        assert_eq!(json["data"]["meta"]["total"], 25);
        assert_eq!(json["data"]["meta"]["last_page"], 3);
        assert!(json["data"]["extras"].is_null());

        let (status, json) = parse_response(
            format_res::page_with_extras(vec!["a"], 1, 10, 1, serde_json::json!({"summary": "ok"}))
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["data"]["extras"]["summary"], "ok");

        let (status, json) =
            parse_response(format_res::retrieved(serde_json::json!({"id": 1})).unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"], "数据获取成功");
        assert_eq!(json["data"]["id"], 1);

        let (status, json) =
            parse_response(format_res::updated(serde_json::json!({"id": 3})).unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"], "数据更新成功");

        let (status, json) = parse_response(format_res::deleted().unwrap()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"], "数据删除成功");
        assert!(json["data"].is_null());

        let (status, json) =
            parse_response(format_res::deleted_with_data(serde_json::json!({"id": 4})).unwrap())
                .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"], "数据删除成功");
        assert_eq!(json["data"]["id"], 4);
    }

    // =========================================================================
    // format_res — 4xx / 5xx
    // =========================================================================

    #[rstest]
    #[case::bad_request(
        format_res::bad_request("bad req"),
        StatusCode::BAD_REQUEST,
        400,
        "bad req"
    )]
    #[case::bad_request_default(
        format_res::bad_request_default(),
        StatusCode::BAD_REQUEST,
        400,
        "请求参数无效"
    )]
    #[case::unauthorized(
        format_res::unauthorized("unauth"),
        StatusCode::UNAUTHORIZED,
        401,
        "unauth"
    )]
    #[case::forbidden(
        format_res::forbidden("forbidden"),
        StatusCode::FORBIDDEN,
        403,
        "forbidden"
    )]
    #[case::not_found(
        format_res::not_found("not found"),
        StatusCode::NOT_FOUND,
        404,
        "not found"
    )]
    #[case::route_not_found(
        format_res::route_not_found(),
        StatusCode::NOT_FOUND,
        404,
        "请求的接口路径不存在"
    )]
    #[case::method_not_allowed(
        format_res::method_not_allowed("method err"),
        StatusCode::METHOD_NOT_ALLOWED,
        405,
        "method err"
    )]
    #[case::conflict(
        format_res::conflict("conflict"),
        StatusCode::CONFLICT,
        409,
        "conflict"
    )]
    #[case::validation(
        format_res::validation_error("invalid"),
        StatusCode::UNPROCESSABLE_ENTITY,
        422,
        "invalid"
    )]
    #[case::too_many(
        format_res::too_many_requests("rate limit"),
        StatusCode::TOO_MANY_REQUESTS,
        429,
        "rate limit"
    )]
    #[case::internal(
        format_res::internal_error("internal err"),
        StatusCode::INTERNAL_SERVER_ERROR,
        500,
        "internal err"
    )]
    #[case::unavailable(
        format_res::service_unavailable("down"),
        StatusCode::SERVICE_UNAVAILABLE,
        503,
        "down"
    )]
    #[tokio::test]
    async fn test_format_res_error_helpers(
        #[case] res_result: loco_rs::prelude::Result<Response>,
        #[case] expected_status: StatusCode,
        #[case] expected_code: u32,
        #[case] expected_msg: &str,
    ) {
        let (status, json) = parse_response(res_result.unwrap()).await;
        assert_eq!(status, expected_status);
        assert_eq!(json["success"], false);
        assert_eq!(json["code"], expected_code);
        assert_eq!(json["message"], expected_msg);
        assert!(json["data"].is_null());
    }

    #[tokio::test]
    async fn test_format_res_error_with_details() {
        let details = serde_json::json!({ "field": ["is required"] });
        let (status, json) = parse_response(
            format_res::validation_error_with_details("val error", details.clone()).unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["code"], 422);
        assert_eq!(json["errors"], details);

        let (status, json) = parse_response(
            format_res::bad_request_with_details("bad req", details.clone()).unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], 400);
        assert_eq!(json["errors"], details);
    }

    #[tokio::test]
    async fn test_format_res_internal_error_with_debug() {
        let (status, json) = parse_response(
            format_res::internal_error_with_debug(
                "crash",
                "EyreError",
                vec!["stack line 1".into()],
            )
            .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["code"], 500);
        assert_eq!(json["message"], "crash");
        assert!(json["errors"].is_null());

        if cfg!(debug_assertions) {
            assert_eq!(json["trace"], serde_json::json!(["stack line 1"]));
        } else {
            assert!(json["trace"].is_null());
        }
    }

    // =========================================================================
    // format_res — 600..999 custom / biz error codes
    // =========================================================================

    #[rstest]
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
        assert_eq!(json["success"], false);
        assert_eq!(json["code"], custom_code);
        assert_eq!(json["message"], msg);

        let details = serde_json::json!({ "reason": "blocked" });
        let (status_det, json_det) = parse_response(
            format_res::custom_error_with_details(custom_code, msg, details.clone()).unwrap(),
        )
        .await;
        assert_eq!(status_det, expected_status);
        assert_eq!(json_det["code"], custom_code);
        assert_eq!(json_det["errors"], details);
    }

    #[tokio::test]
    async fn test_format_res_biz_helpers() {
        let (status, json) =
            parse_response(format_res::biz(biz_code::USER_FROZEN, "账号已被冻结").unwrap()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], 601);
        assert_eq!(json["message"], "账号已被冻结");

        let (status, json) = parse_response(
            format_res::biz(biz_code::INSUFFICIENT_BALANCE, "custom balance msg").unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], 602);
        assert_eq!(json["message"], "custom balance msg");

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
        assert_eq!(json["code"], 602);
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
        assert_eq!(json["code"], 601);
    }
}
