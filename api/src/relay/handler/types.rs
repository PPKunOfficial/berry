use axum::response::sse::Event;
use serde_json::{Value, json};
use crate::relay::client::ClientError;

// 创建错误事件
pub fn create_error_event(error: &ClientError) -> Event {
    Event::default().data(
        json!({
            "error": {
                "message": error.to_string()
            }
        })
        .to_string(),
    )
}

// 创建错误 JSON 响应
pub fn create_error_json(error: &ClientError) -> Value {
    json!({
        "error": {
            "message": error.to_string()
        }
    })
}

// 创建网络错误 JSON 响应
pub fn create_network_error_json(message: &str, details: Option<String>) -> Value {
    json!({
        "error": {
            "message": message,
            "status": Option::<u16>::None,
            "details": details,
        }
    })
}

// 创建上游错误 JSON 响应
pub fn create_upstream_error_json(message: &str, status: Option<u16>, details: Option<String>) -> Value {
    json!({
        "error": {
            "message": message,
            "status": status,
            "details": details,
        }
    })
}
