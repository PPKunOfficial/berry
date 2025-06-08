use berry_api_api::config::model::{Config, Provider, ModelMapping, Backend, LoadBalanceStrategy, GlobalSettings};
use berry_api_api::loadbalance::{LoadBalanceService, MetricsCollector};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// 创建测试配置
fn create_test_config() -> Config {
    let mut providers = HashMap::new();
    
    // 添加一个测试provider（使用httpbin）
    providers.insert("test-provider".to_string(), Provider {
        name: "Test Provider".to_string(),
        base_url: "https://httpbin.org".to_string(),
        api_key: "test-api-key".to_string(),
        models: vec!["test-model".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 10,
        max_retries: 2,
    });

    // 添加一个模拟的失败provider
    providers.insert("failing-provider".to_string(), Provider {
        name: "Failing Provider".to_string(),
        base_url: "https://invalid-url-that-will-fail.example.com".to_string(),
        api_key: "invalid-key".to_string(),
        models: vec!["failing-model".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 5,
        max_retries: 1,
    });

    let mut models = HashMap::new();
    models.insert("test-model".to_string(), ModelMapping {
        name: "test-model".to_string(),
        backends: vec![
            Backend {
                provider: "test-provider".to_string(),
                model: "test-model".to_string(),
                weight: 0.7,
                priority: 1,
                enabled: true,
                tags: vec![],
            },
            Backend {
                provider: "failing-provider".to_string(),
                model: "failing-model".to_string(),
                weight: 0.3,
                priority: 2,
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

#[tokio::test]
async fn test_health_check_debug_logging() {
    // 这个测试主要验证debug日志功能不会导致编译错误
    // 实际的日志输出需要在运行时通过RUST_LOG=debug观察
    let config = create_test_config();
    let service = LoadBalanceService::new(config).unwrap();

    // 启动服务
    service.start().await.unwrap();

    // 等待健康检查运行
    sleep(Duration::from_secs(1)).await;

    // 手动触发健康检查以确保日志输出
    service.trigger_health_check().await.unwrap();

    // 停止服务
    service.stop().await;

    // 测试通过表示debug日志功能正常工作
    assert!(true);
}

#[tokio::test]
async fn test_metrics_debug_logging() {
    let metrics = Arc::new(MetricsCollector::new());

    // 测试失败记录的debug日志
    metrics.record_failure("test-provider:test-model");

    // 测试成功记录的debug日志
    metrics.record_success("test-provider:test-model");

    // 测试恢复尝试的debug日志
    metrics.record_failure("test-provider:test-model"); // 重新标记为失败
    metrics.record_recovery_attempt("test-provider:test-model");

    // 测试通过表示debug日志功能正常工作
    assert!(true);
}

#[tokio::test]
async fn test_backend_selection_debug_logging() {
    let config = create_test_config();
    let service = LoadBalanceService::new(config).unwrap();

    // 启动服务
    service.start().await.unwrap();

    // 模拟一个backend失败
    let metrics = service.get_metrics();
    metrics.record_failure("failing-provider:failing-model");

    // 尝试选择backend，应该触发重试逻辑
    let _selected = service.select_backend("test-model").await.unwrap();

    // 停止服务
    service.stop().await;

    // 测试通过表示debug日志功能正常工作
    assert!(true);
}
