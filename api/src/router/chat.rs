use crate::app::AppState;
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use axum_extra::TypedHeader;
use serde_json::{Value, json};

/// V1 API: 聊天完成
pub async fn chat_completions(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<headers::Authorization<headers::authorization::Bearer>>,
    TypedHeader(content_type): TypedHeader<headers::ContentType>,
    Json(body): Json<Value>,
) -> axum::response::Response {
    // 认证检查
    let token = authorization.token();
    let user = match state.config.validate_user_token(token) {
        Some(user) if user.enabled => user,
        _ => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": {
                        "type": "invalid_token",
                        "message": "The provided API key is invalid",
                        "code": 401
                    }
                })),
            )
                .into_response();
        }
    };

    // 检查模型访问权限
    if let Some(model_name) = body.get("model").and_then(|m| m.as_str()) {
        if !state.config.user_can_access_model(user, model_name) {
            return (
                axum::http::StatusCode::FORBIDDEN,
                Json(json!({
                    "error": {
                        "type": "model_access_denied",
                        "message": format!("Access denied for model: {}", model_name),
                        "code": 403
                    }
                })),
            )
                .into_response();
        }
    }

    // 继续处理请求
    state
        .handler
        .clone()
        .handle_completions(
            TypedHeader(authorization),
            TypedHeader(content_type),
            Json(body),
        )
        .await
}
