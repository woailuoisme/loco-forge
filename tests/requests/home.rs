use loco_rs::testing::prelude::*;
use locos::app::App;
use serde_json::Value;

#[tokio::test]
#[serial_test::serial]
async fn can_get_home() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/").await;
        assert_eq!(res.status_code(), 200);

        let text = res.text();
        let body: Value = serde_json::from_str(&text).expect("Failed to parse JSON");
        assert_eq!(body["success"], true);
        assert_eq!(body["code"], 200);
        assert_eq!(body["data"]["status"], "ok");
        assert_eq!(body["data"]["name"], "locos");
        assert_eq!(body["data"]["docs"]["docs"], "/docs");
        assert_eq!(body["data"]["docs"]["openapi"], "/api-docs/openapi.json");
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn cannot_get_non_existent_route_returns_json_404() {
    request::<App, _, _>(|request, _ctx| async move {
        // 验证访问任意未定义路径 (如 /1) 返回 HTTP 404 与标准 ApiResponse JSON
        let res = request.get("/1").await;
        assert_eq!(res.status_code(), 404);

        let body: Value = serde_json::from_str(&res.text()).expect("Failed to parse JSON");
        assert_eq!(body["success"], false);
        assert_eq!(body["code"], 404003);

        // 验证访问 API 未定义路径 (如 /api/non_existent) 同样返回标准 Route 404 JSON
        let api_res = request.get("/api/non_existent").await;
        assert_eq!(api_res.status_code(), 404);

        let api_body: Value = serde_json::from_str(&api_res.text()).expect("Failed to parse JSON");
        assert_eq!(api_body["success"], false);
        assert_eq!(api_body["code"], 404003);
    })
    .await;
}
