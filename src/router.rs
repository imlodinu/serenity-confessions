use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::get, Router};

pub fn build_router() -> Router {
    Router::new().route("/", get(hello_world))
}

pub async fn hello_world() -> impl IntoResponse {
    // let gds = guilds.load::<Guild>(connection).map(Json).expect("Error loading guilds");
    // gds.into_response()
    (StatusCode::OK, "Hello, World!").into_response()
}
