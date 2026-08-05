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
        assert_eq!(body["status"], "ok");
        assert_eq!(body["name"], "loco_starter");
        assert_eq!(body["docs"]["scalar"], "/scalar");
        assert_eq!(body["docs"]["redoc"], "/redoc");
        assert_eq!(body["docs"]["openapi"], "/api-docs/openapi.json");
    })
    .await;
}
