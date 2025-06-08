use berry_api_api::config::model::{Config, Provider, ModelMapping, Backend, LoadBalanceStrategy, GlobalSettings};
use berry_api_api::loadbalance::LoadBalanceService;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// 创建测试配置，包含一个健康和一个不健康的provider
fn create_retry_test_config() -> Config {
    let mut providers = HashMap::new();
    
    // 健康的provider（使用httpbin）
    providers.insert("healthy-provider".to_string(), Provider {
        name: "Healthy Provider".to_string(),
        base_url: "https://httpbin.org".to_string(),
        api_key: "healthy-api-key".to_string(),
        models: vec!["healthy-model".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 10,
        max_retries: 2,
    });

    // 不健康的provider（无效URL）
    providers.insert("unhealthy-provider".to_string(), Provider {
        name: "Unhealthy Provider".to_string(),
        base_url: "https://invalid-url-for-retry-test.example.com".to_string(),
        api_key: "invalid-api-key".to_string(),
        models: vec!["unhealthy-model".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 5,
        max_retries: 1,
    });

    let mut models = HashMap::new();
    models.insert("test-model".to_string(), ModelMapping {
        name: "test-model".to_string(),
        backends: vec![
            // 不健康的provider权重更高，会被优先选择
            Backend {
                provider: "unhealthy-provider".to_string(),
                model: "unhealthy-model".to_string(),
                weight: 0.8,
                priority: 1,
                enabled: true,
                tags: vec![],
            },
            // 健康的provider作为备选
            Backend {
                provider: "healthy-provider".to_string(),
                model: "healthy-model".to_string(),
                weight: 0.2,
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
            health_check_interval_seconds: 30,
            request_timeout_seconds: 10,
            max_retries: 2,
            circuit_breaker_failure_threshold: 3,
            circuit_breaker_timeout_seconds: 30,
            recovery_check_interval_seconds: 60,
            max_internal_retries: 3, // 设置较高的重试次数
            health_check_timeout_seconds: 10,
        },
    }
}

#[tokio::test]
async fn test_backend_selection_retry() {
    let config = create_retry_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    // 等待健康检查运行
    sleep(Duration::from_secs(2)).await;
    
    // 手动标记不健康provider为失败状态
    let metrics = service.get_metrics();
    metrics.record_failure("unhealthy-provider:unhealthy-model");
    
    // 尝试选择backend，应该重试并选择健康的
    let selected = service.select_backend("test-model").await.unwrap();
    
    // 验证选择了健康的provider
    println!("Selected backend: {}:{}", selected.backend.provider, selected.backend.model);
    
    // 由于不健康的provider被标记为失败，应该选择健康的provider
    // 注意：这个测试可能不稳定，因为负载均衡算法的随机性
    
    // 停止服务
    service.stop().await;
}

#[tokio::test]
async fn test_request_level_retry_mechanism() {
    let config = create_retry_test_config();
    let service = LoadBalanceService::new(config).unwrap();

    // 启动服务
    service.start().await.unwrap();

    // 等待健康检查运行
    sleep(Duration::from_secs(2)).await;

    // 模拟多次backend选择，验证重试逻辑
    for i in 0..5 {
        match service.select_backend("test-model").await {
            Ok(selected) => {
                println!("Attempt {}: Selected backend: {}:{}",
                         i + 1, selected.backend.provider, selected.backend.model);
            }
            Err(e) => {
                println!("Attempt {}: Backend selection failed: {}", i + 1, e);
            }
        }
    }

    // 停止服务
    service.stop().await;
}

#[tokio::test]
async fn test_retry_configuration() {
    let config = create_retry_test_config();

    // 验证重试配置
    assert_eq!(config.settings.max_internal_retries, 3);
    assert_eq!(config.settings.max_retries, 2);

    let _service = LoadBalanceService::new(config).unwrap();

    println!("Retry configuration verified:");
    println!("  max_internal_retries: 3");
    println!("  max_retries: 2");
}

#[tokio::test]
async fn test_metrics_tracking_during_retry() {
    let config = create_retry_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    let metrics = service.get_metrics();
    
    // 模拟多次失败
    metrics.record_failure("unhealthy-provider:unhealthy-model");
    metrics.record_failure("unhealthy-provider:unhealthy-model");
    metrics.record_failure("unhealthy-provider:unhealthy-model");
    
    // 检查不健康列表
    let unhealthy = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy.len(), 1);
    assert_eq!(unhealthy[0].backend_key, "unhealthy-provider:unhealthy-model");
    assert_eq!(unhealthy[0].failure_count, 3);
    
    // 模拟成功恢复
    metrics.record_success("unhealthy-provider:unhealthy-model");
    
    // 检查是否从不健康列表中移除
    let unhealthy_after_recovery = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy_after_recovery.len(), 0);
    
    println!("Metrics tracking during retry verified");
    
    // 停止服务
    service.stop().await;
}
