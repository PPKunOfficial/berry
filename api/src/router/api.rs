use axum::{
    Router,
    routing::get,
};

pub fn set_api_router() -> Router {
    Router::new().nest("/user", set_user_router())
}
fn set_user_router() -> Router {
    Router::new().route("/", get(|| async { "User router" }))
}
