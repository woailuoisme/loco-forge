use loco_rs::testing::prelude::*;
use locos::app::App;
use serde_json::Value;

#[tokio::test]
#[serial_test::serial]
async fn can_get_liveness() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/livez").await;
        assert_eq!(res.status_code(), 200);
        // 纯文本契约：探针只回 "ok"，无 JSON 外壳
        assert_eq!(res.text(), "ok");
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn can_get_readiness() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/readyz").await;
        assert_eq!(res.status_code(), 200);

        let body: Value = serde_json::from_str(&res.text()).expect("Failed to parse JSON");
        // 裸 ReadinessData：200 与 503 共用同一根结构，无信封
        assert_eq!(body["status"], "up");
        assert!(body.get("success").is_none());
        let db_kind = body["checks"]["db"]["kind"].as_str().unwrap();
        assert!(db_kind == "pg" || db_kind == "sqlite");
        assert_eq!(body["checks"]["cache"]["kind"], "memory");
        assert_eq!(body["checks"]["storage"]["kind"], "memory");
    })
    .await;
}
