use loco_rs::testing::prelude::*;
use locos::app::App;

#[tokio::test]
#[serial_test::serial]
async fn can_get_file_not_found() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/v1/files/uploads/non_existent.txt").await;
        res.assert_status(axum::http::StatusCode::NOT_FOUND);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn file_upload_requires_authentication() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.post("/api/v1/files/upload").await;
        res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    })
    .await;
}
