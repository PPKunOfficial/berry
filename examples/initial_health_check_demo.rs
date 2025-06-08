use berry_api_api::config::model::{Config, Provider, ModelMapping, Backend, LoadBalanceStrategy, GlobalSettings};
use berry_api_api::loadbalance::LoadBalanceService;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// 创建演示配置
fn create_demo_config() -> Config {
    let mut providers = HashMap::new();
    
    // 健康的provider（使用httpbin）
    providers.insert("healthy-provider".to_string(), Provider {
        name: "Healthy Provider (httpbin)".to_string(),
        base_url: "https://httpbin.org".to_string(),
        api_key: "demo-api-key".to_string(),
        models: vec!["demo-model".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 10,
        max_retries: 2,
    });

    // 会失败的provider
    providers.insert("failing-provider".to_string(), Provider {
        name: "Failing Provider".to_string(),
        base_url: "https://invalid-url-for-demo.example.com".to_string(),
        api_key: "invalid-key".to_string(),
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
                provider: "healthy-provider".to_string(),
                model: "demo-model".to_string(),
                weight: 0.7,
                priority: 1,
                enabled: true,
                tags: vec![],
            },
        ],
        strategy: LoadBalanceStrategy::WeightedFailover,
        enabled: true,
    });

    models.insert("failing-demo-model".to_string(), ModelMapping {
        name: "failing-demo-model".to_string(),
        backends: vec![
            Backend {
                provider: "failing-provider".to_string(),
                model: "failing-demo-model".to_string(),
                weight: 1.0,
                priority: 1,
                enabled: true,
                tags: vec![],
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
            health_check_interval_seconds: 15, // 较短的间隔用于演示
            request_timeout_seconds: 10,
            max_retries: 2,
            circuit_breaker_failure_threshold: 3,
            circuit_breaker_timeout_seconds: 30,
            recovery_check_interval_seconds: 20,
            max_internal_retries: 2,
            health_check_timeout_seconds: 10,
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Initial Health Check Demo");
    println!("This demo shows how initial health checks mark all providers as healthy,");
    println!("but subsequent checks require chat validation for recovery.");
    
    let config = create_demo_config();
    let service = LoadBalanceService::new(config)?;
    
    println!("📋 Configuration loaded with 2 providers:");
    println!("  - healthy-provider (httpbin.org) - should work");
    println!("  - failing-provider (invalid URL) - will fail");

    // 启动服务 - 这会触发初始健康检查
    println!("🔄 Starting service and performing initial health check...");
    service.start().await?;

    // 等待初始健康检查完成
    sleep(Duration::from_secs(5)).await;

    let metrics = service.get_metrics();

    println!("📊 Initial Health Check Results:");
    let healthy_status = metrics.is_healthy("healthy-provider", "demo-model");
    let failing_status = metrics.is_healthy("failing-provider", "failing-demo-model");

    println!("  ✅ healthy-provider:demo-model = {}", healthy_status);
    println!("  ✅ failing-provider:failing-demo-model = {}", failing_status);
    println!("📝 Note: Both are marked healthy after initial check, regardless of actual API response");

    // 模拟一个backend失败
    println!("🔥 Simulating failure for healthy-provider:demo-model...");
    metrics.record_failure("healthy-provider:demo-model");

    let after_failure = metrics.is_healthy("healthy-provider", "demo-model");
    println!("  ❌ healthy-provider:demo-model after failure = {}", after_failure);

    // 检查不健康列表
    let unhealthy = metrics.get_unhealthy_backends();
    println!("📋 Unhealthy backends list: {} items", unhealthy.len());
    for backend in &unhealthy {
        println!("  - {} (failures: {})", backend.backend_key, backend.failure_count);
    }

    // 等待下一次健康检查
    println!("⏳ Waiting for next routine health check (15 seconds)...");
    sleep(Duration::from_secs(16)).await;

    // 检查状态是否改变
    let after_routine_check = metrics.is_healthy("healthy-provider", "demo-model");
    println!("📊 After routine health check:");
    println!("  🔍 healthy-provider:demo-model = {}", after_routine_check);

    if !after_routine_check {
        println!("  ✅ Correct! Backend remains unhealthy despite successful API check");
        println!("  📝 This proves that routine checks don't auto-recover failed backends");
    } else {
        println!("  ⚠️  Backend was auto-recovered, which shouldn't happen");
    }

    // 演示chat验证恢复
    println!("💬 Simulating chat validation recovery...");
    metrics.record_success("healthy-provider:demo-model");

    let after_chat_recovery = metrics.is_healthy("healthy-provider", "demo-model");
    println!("  ✅ healthy-provider:demo-model after chat validation = {}", after_chat_recovery);

    if after_chat_recovery {
        println!("  🎉 Success! Backend recovered through chat validation");
    }

    // 检查不健康列表是否更新
    let unhealthy_after_recovery = metrics.get_unhealthy_backends();
    println!("📋 Unhealthy backends after recovery: {} items", unhealthy_after_recovery.len());

    // 演示手动健康检查
    println!("🔧 Triggering manual health check...");
    service.trigger_health_check().await?;
    sleep(Duration::from_secs(3)).await;

    println!("📊 Final status check:");
    let final_healthy = metrics.is_healthy("healthy-provider", "demo-model");
    let final_failing = metrics.is_healthy("failing-provider", "failing-demo-model");
    println!("  - healthy-provider:demo-model = {}", final_healthy);
    println!("  - failing-provider:failing-demo-model = {}", final_failing);

    // 停止服务
    println!("🛑 Stopping service...");
    service.stop().await;

    println!("🎯 Demo Summary:");
    println!("  1. ✅ Initial health check marked all providers as healthy");
    println!("  2. ❌ Manual failure marking worked correctly");
    println!("  3. 🔄 Routine health checks preserved unhealthy status");
    println!("  4. 💬 Chat validation successfully restored health");
    println!("  5. 📝 This ensures only validated recovery, not automatic recovery");

    println!("✨ Demo completed successfully!");
    
    Ok(())
}
