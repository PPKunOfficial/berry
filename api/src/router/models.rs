use crate::app::AppState;
use axum::{
    extract::State,
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use serde_json::json;

/// 列出可用模型（无认证，返回所有可用模型）
pub async fn list_models(State(state): State<AppState>) -> impl IntoResponse {
    let all_models = state.load_balancer.get_available_models();
    state.handler.handle_models_for_user(all_models).await
}

/// V1 API: 列出可用模型（需要认证）
pub async fn list_models_v1(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<headers::Authorization<headers::authorization::Bearer>>,
) -> impl IntoResponse {
    // 认证检查
    let token = authorization.token();
    let user = match state.config.validate_user_token(token) {
        Some(user) if user.enabled => user,
        _ => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(json!({
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

    // 获取用户可访问的模型列表
    let user_models = state.config.get_user_available_models(user);

    // 使用handler的方法来格式化响应
    state
        .handler
        .handle_models_for_user(user_models)
        .await
        .into_response()
}
