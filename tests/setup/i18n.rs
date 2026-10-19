use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::app::App;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_i18n_middleware_with_query_param() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/v1/sessions?lang=zh-CN").await;
        // 未带 Token 的情况统一下发 401 响应
        assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_i18n_middleware_with_header() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .get("/_health")
            .add_header(
                axum::http::header::ACCEPT_LANGUAGE,
                "zh-CN,zh;q=0.9,en;q=0.8",
            )
            .await;
        assert_eq!(res.status_code(), StatusCode::OK);
    })
    .await;
}
