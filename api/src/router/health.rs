use crate::app::AppState;
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::collections::HashMap;

/// 详细健康检查处理器 - 返回具体模型和渠道的健康状态
pub async fn detailed_health_check(State(state): State<AppState>) -> impl IntoResponse {
    let health = state.load_balancer.get_service_health().await;
    let config = &state.config;
    let metrics = state.load_balancer.get_metrics();

    // 获取详细的提供商健康状态
    let mut providers_detail = HashMap::new();
    for (provider_id, provider) in &config.providers {
        if provider.enabled {
            let mut provider_models = HashMap::new();
            let mut provider_healthy = true;

            for model in &provider.models {
                let is_healthy = metrics.is_healthy(provider_id, model);
                let latency = metrics.get_latency(provider_id, model);
                let failure_count = metrics.get_failure_count(provider_id, model);

                provider_models.insert(model.clone(), json!({
                    "healthy": is_healthy,
                    "latency_ms": latency.map(|l| l.as_millis()),
                    "failure_count": failure_count,
                    "backend_key": format!("{}:{}", provider_id, model)
                }));

                if !is_healthy {
                    provider_healthy = false;
                }
            }

            providers_detail.insert(provider_id.clone(), json!({
                "name": provider.name,
                "base_url": provider.base_url,
                "healthy": provider_healthy,
                "enabled": provider.enabled,
                "models": provider_models,
                "total_models": provider.models.len(),
                "healthy_models": provider.models.iter()
                    .filter(|model| metrics.is_healthy(provider_id, model))
                    .count()
            }));
        }
    }

    // 获取详细的模型健康状态
    let mut models_detail = HashMap::new();
    for (model_id, model_mapping) in &config.models {
        if model_mapping.enabled {
            let mut model_backends = Vec::new();
            let mut healthy_backends = 0;

            for backend in &model_mapping.backends {
                if backend.enabled {
                    let is_healthy = metrics.is_healthy(&backend.provider, &backend.model);
                    let latency = metrics.get_latency(&backend.provider, &backend.model);
                    let failure_count = metrics.get_failure_count(&backend.provider, &backend.model);

                    if is_healthy {
                        healthy_backends += 1;
                    }

                    model_backends.push(json!({
                        "provider": backend.provider,
                        "model": backend.model,
                        "weight": backend.weight,
                        "priority": backend.priority,
                        "healthy": is_healthy,
                        "enabled": backend.enabled,
                        "latency_ms": latency.map(|l| l.as_millis()),
                        "failure_count": failure_count,
                        "backend_key": format!("{}:{}", backend.provider, backend.model)
                    }));
                }
            }

            models_detail.insert(model_id.clone(), json!({
                "name": model_mapping.name,
                "strategy": format!("{:?}", model_mapping.strategy),
                "enabled": model_mapping.enabled,
                "backends": model_backends,
                "total_backends": model_mapping.backends.iter().filter(|b| b.enabled).count(),
                "healthy_backends": healthy_backends,
                "health_ratio": if model_mapping.backends.len() > 0 {
                    healthy_backends as f64 / model_mapping.backends.iter().filter(|b| b.enabled).count() as f64
                } else {
                    0.0
                }
            }));
        }
    }

    // 获取不健康的后端列表
    let unhealthy_backends = metrics.get_unhealthy_backends();
    let unhealthy_detail: Vec<_> = unhealthy_backends.iter().map(|backend| {
        json!({
            "backend_key": backend.backend_key,
            "failure_count": backend.failure_count,
            "first_failure": backend.first_failure_time.elapsed().as_secs(),
            "last_failure": backend.last_failure_time.elapsed().as_secs(),
            "recovery_attempts": backend.recovery_attempts,
            "last_recovery_attempt": backend.last_recovery_attempt.map(|t| t.elapsed().as_secs())
        })
    }).collect();

    let status = if health.is_healthy() {
        "healthy"
    } else {
        "unhealthy"
    };
    let status_code = if health.is_healthy() {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "status": status,
            "service_running": health.is_running,
            "summary": {
                "provider_health": {
                    "total": health.health_summary.total_providers,
                    "healthy": health.health_summary.healthy_providers,
                    "ratio": health.health_summary.provider_health_ratio
                },
                "model_health": {
                    "total": health.health_summary.total_models,
                    "healthy": health.health_summary.healthy_models,
                    "ratio": health.health_summary.model_health_ratio
                }
            },
            "providers": providers_detail,
            "models": models_detail,
            "unhealthy_backends": unhealthy_detail,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    )
}

/// 简化健康检查处理器 - 返回原来/health的内容
pub async fn simple_health_check(State(state): State<AppState>) -> impl IntoResponse {
    let health = state.load_balancer.get_service_health().await;

    Json(json!({
        "status": if health.is_healthy() { "ok" } else { "error" },
        "models_available": health.health_summary.has_available_models(),
        "timestamp": chrono::Utc::now().timestamp()
    }))
}
