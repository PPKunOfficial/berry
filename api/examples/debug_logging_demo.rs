use berry_api_api::config::model::{Config, Provider, ModelMapping, Backend, LoadBalanceStrategy, GlobalSettings};
use berry_api_api::loadbalance::LoadBalanceService;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, debug};
use tracing_subscriber;

/// 创建测试配置
fn create_demo_config() -> Config {
    let mut providers = HashMap::new();
    
    // 添加一个测试provider（使用httpbin）
    providers.insert("httpbin-provider".to_string(), Provider {
        name: "HTTPBin Test Provider".to_string(),
        base_url: "https://httpbin.org".to_string(),
        api_key: "demo-api-key".to_string(),
        models: vec!["demo-model".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 10,
        max_retries: 2,
    });

    // 添加一个模拟的失败provider
    providers.insert("failing-provider".to_string(), Provider {
        name: "Simulated Failing Provider".to_string(),
        base_url: "https://invalid-url-demo.example.com".to_string(),
        api_key: "invalid-demo-key".to_string(),
        models: vec!["failing-demo-model".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 5,
        max_retries: 1,
    });

    let mut models = HashMap::new();
    models.insert("demo-model".to_string(), ModelMapping {
        name: "demo-model".to_string(),
        backends: vec![
            Backend {
                provider: "httpbin-provider".to_string(),
                model: "demo-model".to_string(),
                weight: 0.8,
                priority: 1,
                enabled: true,
                tags: vec!["demo".to_string()],
            },
            Backend {
                provider: "failing-provider".to_string(),
                model: "failing-demo-model".to_string(),
                weight: 0.2,
                priority: 2,
                enabled: true,
                tags: vec!["demo".to_string()],
            },
        ],
        strategy: LoadBalanceStrategy::WeightedFailover,
        enabled: true,
    });

    Config {
        providers,
        models,
        users: HashMap::new(),
        settings: GlobalSettings {
            health_check_interval_seconds: 5,
            request_timeout_seconds: 5,
            max_retries: 2,
            circuit_breaker_failure_threshold: 3,
            circuit_breaker_timeout_seconds: 30,
            recovery_check_interval_seconds: 10,
            max_internal_retries: 2,
            health_check_timeout_seconds: 5,
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("=== Berry API 健康检查 Debug 日志演示 ===");
    
    // 创建配置和服务
    let config = create_demo_config();
    let service = LoadBalanceService::new(config)?;
    
    info!("创建负载均衡服务成功");
    
    // 启动服务
    info!("启动负载均衡服务...");
    service.start().await?;
    
    // 演示1: 健康检查
    info!("\n=== 演示1: 健康检查过程 ===");
    info!("手动触发健康检查...");
    service.trigger_health_check().await?;
    
    sleep(Duration::from_secs(2)).await;
    
    // 演示2: 指标操作
    info!("\n=== 演示2: 指标记录过程 ===");
    let metrics = service.get_metrics();
    
    info!("模拟backend失败...");
    metrics.record_failure("failing-provider:failing-demo-model");
    
    info!("模拟backend恢复...");
    metrics.record_success("failing-provider:failing-demo-model");
    
    info!("模拟恢复尝试...");
    metrics.record_failure("failing-provider:failing-demo-model"); // 重新失败
    metrics.record_recovery_attempt("failing-provider:failing-demo-model");
    
    // 演示3: 智能backend选择
    info!("\n=== 演示3: 智能backend选择过程 ===");
    info!("尝试选择backend...");
    
    match service.select_backend("demo-model").await {
        Ok(selected) => {
            info!("成功选择backend: {}:{}", selected.backend.provider, selected.backend.model);
        }
        Err(e) => {
            info!("Backend选择失败: {}", e);
        }
    }
    
    // 演示4: 不健康列表管理
    info!("\n=== 演示4: 不健康列表管理 ===");
    let unhealthy = metrics.get_unhealthy_backends();
    info!("当前不健康的backends数量: {}", unhealthy.len());
    
    for backend in &unhealthy {
        debug!("不健康backend: {} (失败次数: {})", 
               backend.backend_key, backend.failure_count);
    }
    
    // 等待一段时间观察恢复检查
    info!("\n=== 演示5: 恢复检查过程 ===");
    info!("等待恢复检查运行...");
    sleep(Duration::from_secs(3)).await;
    
    // 获取服务健康状态
    info!("\n=== 演示6: 服务健康状态 ===");
    let health = service.get_service_health().await;
    info!("服务运行状态: {}", health.is_running);
    info!("健康的providers: {}/{}", health.health_summary.healthy_providers, health.health_summary.total_providers);
    info!("健康的models: {}/{}", health.health_summary.healthy_models, health.health_summary.total_models);
    info!("系统健康状态: {}", if health.is_healthy() { "健康" } else { "不健康" });
    
    // 停止服务
    info!("\n=== 停止服务 ===");
    service.stop().await;
    info!("服务已停止");
    
    info!("\n=== 演示完成 ===");
    info!("提示: 使用 RUST_LOG=debug 可以看到更详细的调试信息");
    info!("      使用 RUST_LOG=info 可以看到关键操作信息");
    info!("      使用 RUST_LOG=warn 只看到警告和错误");
    
    Ok(())
}
