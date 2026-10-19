use loco_rs::testing::prelude::*;
use locos::app::App;
use serde_json::Value;

#[tokio::test]
#[serial_test::serial]
async fn can_get_home() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/").await;
        assert_eq!(res.status_code(), 200);

        let body: Value = serde_json::from_str(&res.text()).expect("Failed to parse JSON");
        // 裸 HomeResource，无响应信封
        assert_eq!(body["status"], "ok");
        assert_eq!(body["name"], "locos");
        assert!(body["version"].is_string());
        assert!(body.get("success").is_none());
        assert!(body.get("docs").is_none());
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn cannot_get_non_existent_route_returns_json_404() {
    request::<App, _, _>(|request, _ctx| async move {
        // 任意未定义路径返回 404 与统一 error 信封 {error: {code, message}}
        let res = request.get("/1").await;
        assert_eq!(res.status_code(), 404);

        let body: Value = serde_json::from_str(&res.text()).expect("Failed to parse JSON");
        assert!(body.get("success").is_none());
        assert_eq!(body["code"], "NOT_FOUND");
        assert!(body["message"].is_string());

        // 未定义的 API 版本路径同样返回统一 404 信封
        let api_res = request.get("/v1/non_existent").await;
        assert_eq!(api_res.status_code(), 404);

        let api_body: Value = serde_json::from_str(&api_res.text()).expect("Failed to parse JSON");
        assert_eq!(api_body["code"], "NOT_FOUND");
    })
    .await;
}
