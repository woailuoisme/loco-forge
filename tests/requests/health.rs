use loco_rs::testing::prelude::*;
use locos::app::App;
use serde_json::Value;

#[tokio::test]
#[serial_test::serial]
async fn can_get_health_liveness() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/health").await;
        assert_eq!(res.status_code(), 200);

        let text = res.text();
        let body: Value = serde_json::from_str(&text).expect("Failed to parse JSON");
        assert_eq!(body["success"], true);
        assert_eq!(body["code"], 200);
        assert_eq!(body["message"], "ok");
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn can_get_health_readiness() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/health/ready").await;
        assert_eq!(res.status_code(), 200);

        let text = res.text();
        let body: Value = serde_json::from_str(&text).expect("Failed to parse JSON");
        assert_eq!(body["success"], true);
        assert_eq!(body["code"], 200);
        assert_eq!(body["data"]["db"], "ok");
    })
    .await;
}
