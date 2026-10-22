use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct StatusBody {
    status: &'static str,
}

pub async fn live() -> Json<StatusBody> {
    Json(StatusBody { status: "ok" })
}

pub async fn ready() -> Json<StatusBody> {
    Json(StatusBody { status: "ready" })
}
