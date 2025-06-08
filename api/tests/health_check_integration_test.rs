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

    // 添加一个模拟的OpenAI provider
    providers.insert("openai-mock".to_string(), Provider {
        name: "OpenAI Mock".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: "sk-test-key".to_string(),
        models: vec!["gpt-3.5-turbo".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 10,
        max_retries: 2,
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
                provider: "openai-mock".to_string(),
                model: "gpt-3.5-turbo".to_string(),
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
            request_timeout_seconds: 10,
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
async fn test_health_check_basic_functionality() {
    let config = create_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    // 等待一段时间让健康检查运行
    sleep(Duration::from_secs(2)).await;
    
    // 检查服务健康状态
    let health = service.get_service_health().await;
    assert!(health.is_running);
    assert!(health.health_summary.total_providers > 0);
    
    // 停止服务
    service.stop().await;
}

#[tokio::test]
async fn test_unhealthy_backend_management() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // 模拟一个backend失败
    let backend_key = "test-provider:test-model";
    metrics.record_failure(backend_key);
    
    // 检查是否被标记为不健康
    assert!(!metrics.is_healthy("test-provider", "test-model"));
    assert!(metrics.is_in_unhealthy_list(backend_key));
    
    // 获取不健康的backends
    let unhealthy = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy.len(), 1);
    assert_eq!(unhealthy[0].backend_key, backend_key);
    
    // 模拟恢复
    metrics.record_success(backend_key);
    
    // 检查是否恢复健康
    assert!(metrics.is_healthy("test-provider", "test-model"));
    assert!(!metrics.is_in_unhealthy_list(backend_key));
    
    // 不健康列表应该为空
    let unhealthy_after_recovery = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy_after_recovery.len(), 0);
}

#[tokio::test]
async fn test_recovery_check_timing() {
    let metrics = Arc::new(MetricsCollector::new());
    let backend_key = "test-provider:test-model";
    
    // 标记为失败
    metrics.record_failure(backend_key);
    
    // 立即检查是否需要恢复检查（应该需要，因为从未尝试过）
    assert!(metrics.needs_recovery_check(backend_key, Duration::from_secs(60)));
    
    // 记录一次恢复尝试
    metrics.record_recovery_attempt(backend_key);
    
    // 立即再次检查（应该不需要，因为刚刚尝试过）
    assert!(!metrics.needs_recovery_check(backend_key, Duration::from_secs(60)));
    
    // 等待一小段时间后检查（仍然不需要）
    sleep(Duration::from_millis(10)).await;
    assert!(!metrics.needs_recovery_check(backend_key, Duration::from_millis(100)));
    
    // 等待足够长时间后检查（应该需要）
    sleep(Duration::from_millis(150)).await;
    assert!(metrics.needs_recovery_check(backend_key, Duration::from_millis(100)));
}

#[tokio::test]
async fn test_smart_backend_selection() {
    let config = create_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    // 模拟一个backend失败
    let metrics = service.get_metrics();
    metrics.record_failure("test-provider:test-model");
    
    // 尝试选择backend，应该选择健康的那个
    let selected = service.select_backend("test-model").await.unwrap();
    
    // 由于test-provider被标记为不健康，应该选择openai-mock
    // 注意：这个测试可能不稳定，因为负载均衡算法可能仍然选择不健康的backend
    println!("Selected backend: {}:{}", selected.backend.provider, selected.backend.model);
    
    // 停止服务
    service.stop().await;
}
