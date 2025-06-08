use crate::app::AppState;
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde_json::json;

/// 指标处理器
pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let health = state.load_balancer.get_service_health().await;

    Json(json!({
        "service": {
            "running": health.is_running,
            "total_requests": health.total_requests,
            "successful_requests": health.successful_requests,
            "success_rate": health.success_rate()
        },
        "providers": {
            "total": health.health_summary.total_providers,
            "healthy": health.health_summary.healthy_providers,
            "health_ratio": health.health_summary.provider_health_ratio
        },
        "models": {
            "total": health.health_summary.total_models,
            "healthy": health.health_summary.healthy_models,
            "health_ratio": health.health_summary.model_health_ratio,
            "details": health.model_stats
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
