use axum::response::sse::Event;
use axum::{response::IntoResponse, http::StatusCode, Json};
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

/// 错误类型枚举，用于确定HTTP状态码
#[derive(Debug, Clone)]
pub enum ErrorType {
    /// 客户端错误 - 400 Bad Request
    BadRequest,
    /// 认证错误 - 401 Unauthorized
    Unauthorized,
    /// 权限错误 - 403 Forbidden
    Forbidden,
    /// 资源未找到 - 404 Not Found
    NotFound,
    /// 请求超时 - 408 Request Timeout
    RequestTimeout,
    /// 请求过多 - 429 Too Many Requests
    TooManyRequests,
    /// 服务器内部错误 - 500 Internal Server Error
    InternalServerError,
    /// 服务不可用 - 503 Service Unavailable
    ServiceUnavailable,
    /// 网关超时 - 504 Gateway Timeout
    GatewayTimeout,
}

impl ErrorType {
    /// 获取对应的HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorType::BadRequest => StatusCode::BAD_REQUEST,
            ErrorType::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorType::Forbidden => StatusCode::FORBIDDEN,
            ErrorType::NotFound => StatusCode::NOT_FOUND,
            ErrorType::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
            ErrorType::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ErrorType::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ErrorType::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }

    /// 根据错误消息内容推断错误类型
    pub fn from_error_message(message: &str) -> Self {
        let message_lower = message.to_lowercase();

        if message_lower.contains("unauthorized") || message_lower.contains("invalid token") || message_lower.contains("authentication") {
            ErrorType::Unauthorized
        } else if message_lower.contains("forbidden") || message_lower.contains("permission") || message_lower.contains("access denied") {
            ErrorType::Forbidden
        } else if message_lower.contains("not found") || message_lower.contains("model") && message_lower.contains("not") {
            ErrorType::NotFound
        } else if message_lower.contains("timeout") || message_lower.contains("timed out") {
            ErrorType::GatewayTimeout
        } else if message_lower.contains("too many requests") || message_lower.contains("rate limit") {
            ErrorType::TooManyRequests
        } else if message_lower.contains("service unavailable") || message_lower.contains("no available backends") || message_lower.contains("all") && message_lower.contains("unhealthy") {
            ErrorType::ServiceUnavailable
        } else if message_lower.contains("bad request") || message_lower.contains("invalid") && !message_lower.contains("token") {
            ErrorType::BadRequest
        } else {
            ErrorType::InternalServerError
        }
    }
}

/// 创建带有正确HTTP状态码的错误响应
pub fn create_error_response(error_type: ErrorType, message: &str, details: Option<String>) -> impl IntoResponse {
    let status_code = error_type.status_code();
    let error_json = json!({
        "error": {
            "message": message,
            "type": format!("{:?}", error_type),
            "status": status_code.as_u16(),
            "details": details,
        }
    });

    (status_code, Json(error_json))
}

/// 创建带有正确HTTP状态码的错误响应（从ClientError）
pub fn create_client_error_response(error: &ClientError) -> impl IntoResponse {
    let message = error.to_string();
    let error_type = ErrorType::from_error_message(&message);

    let status_code = error_type.status_code();
    let error_json = json!({
        "error": {
            "message": message,
            "type": format!("{:?}", error_type),
            "status": status_code.as_u16(),
            "details": Option::<String>::None,
        }
    });

    (status_code, Json(error_json))
}

/// 创建服务不可用错误响应
pub fn create_service_unavailable_response(message: &str, details: Option<String>) -> impl IntoResponse {
    create_error_response(ErrorType::ServiceUnavailable, message, details)
}

/// 创建内部服务器错误响应
pub fn create_internal_error_response(message: &str, details: Option<String>) -> impl IntoResponse {
    create_error_response(ErrorType::InternalServerError, message, details)
}

/// 创建网关超时错误响应
pub fn create_gateway_timeout_response(message: &str, details: Option<String>) -> impl IntoResponse {
    create_error_response(ErrorType::GatewayTimeout, message, details)
}
