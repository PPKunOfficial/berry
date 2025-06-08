use berry_api_api::config::model::{Config, Provider, ModelMapping, Backend, LoadBalanceStrategy, GlobalSettings};
use berry_api_api::loadbalance::{LoadBalanceService, RequestResult};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// 创建测试配置
fn create_auto_recovery_test_config() -> Config {
    let mut providers = HashMap::new();
    
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

    providers.insert("backup-provider".to_string(), Provider {
        name: "Backup Provider".to_string(),
        base_url: "https://httpbin.org".to_string(),
        api_key: "backup-api-key".to_string(),
        models: vec!["backup-model".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 10,
        max_retries: 2,
    });

    let mut models = HashMap::new();
    models.insert("demo-model".to_string(), ModelMapping {
        name: "demo-model".to_string(),
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
                provider: "backup-provider".to_string(),
                model: "backup-model".to_string(),
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
            health_check_interval_seconds: 30,
            request_timeout_seconds: 10,
            max_retries: 2,
            circuit_breaker_failure_threshold: 3,
            circuit_breaker_timeout_seconds: 30,
            recovery_check_interval_seconds: 60,
            max_internal_retries: 2,
            health_check_timeout_seconds: 10,
        },
    }
}

#[tokio::test]
async fn test_auto_recovery_on_successful_request() {
    let config = create_auto_recovery_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    // 等待初始健康检查完成
    sleep(Duration::from_secs(2)).await;
    
    let metrics = service.get_metrics();
    
    // 验证初始状态：所有backend都健康
    assert!(metrics.is_healthy("test-provider", "test-model"));
    assert!(metrics.is_healthy("backup-provider", "backup-model"));
    assert!(!metrics.is_in_unhealthy_list("test-provider:test-model"));
    assert!(!metrics.is_in_unhealthy_list("backup-provider:backup-model"));
    
    println!("✅ Initial state: All backends healthy");
    
    // 模拟一个backend失败
    metrics.record_failure("test-provider:test-model");
    
    // 验证backend被标记为不健康
    assert!(!metrics.is_healthy("test-provider", "test-model"));
    assert!(metrics.is_in_unhealthy_list("test-provider:test-model"));
    
    println!("❌ Backend marked as unhealthy: test-provider:test-model");
    
    // 检查不健康列表
    let unhealthy_before = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy_before.len(), 1);
    assert_eq!(unhealthy_before[0].backend_key, "test-provider:test-model");
    
    println!("📋 Unhealthy list contains: {}", unhealthy_before[0].backend_key);
    
    // 模拟用户请求成功（这应该触发自动恢复）
    service.record_request_result(
        "test-provider",
        "test-model", 
        RequestResult::Success { latency: Duration::from_millis(150) }
    ).await;
    
    println!("🔄 Simulated successful user request to unhealthy backend");
    
    // 验证backend自动恢复为健康
    assert!(metrics.is_healthy("test-provider", "test-model"));
    assert!(!metrics.is_in_unhealthy_list("test-provider:test-model"));
    
    println!("✅ Backend automatically recovered to healthy state");
    
    // 验证不健康列表为空
    let unhealthy_after = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy_after.len(), 0);
    
    println!("📋 Unhealthy list is now empty");
    
    // 停止服务
    service.stop().await;
    
    println!("🎉 Auto-recovery test completed successfully!");
}

#[tokio::test]
async fn test_failed_request_keeps_backend_unhealthy() {
    let config = create_auto_recovery_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    let metrics = service.get_metrics();
    
    // 标记backend为不健康
    metrics.record_failure("test-provider:test-model");
    assert!(!metrics.is_healthy("test-provider", "test-model"));
    assert!(metrics.is_in_unhealthy_list("test-provider:test-model"));
    
    println!("❌ Backend marked as unhealthy: test-provider:test-model");
    
    // 模拟用户请求失败
    service.record_request_result(
        "test-provider",
        "test-model", 
        RequestResult::Failure { error: "Connection timeout".to_string() }
    ).await;
    
    println!("❌ Simulated failed user request to unhealthy backend");
    
    // 验证backend仍然不健康
    assert!(!metrics.is_healthy("test-provider", "test-model"));
    assert!(metrics.is_in_unhealthy_list("test-provider:test-model"));
    
    println!("❌ Backend remains unhealthy after failed request");
    
    // 验证失败计数增加
    let unhealthy = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy.len(), 1);
    assert!(unhealthy[0].failure_count >= 2); // 至少2次失败
    
    println!("📊 Failure count increased: {}", unhealthy[0].failure_count);
    
    // 停止服务
    service.stop().await;
}

#[tokio::test]
async fn test_multiple_backends_auto_recovery() {
    let config = create_auto_recovery_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    let metrics = service.get_metrics();
    
    // 标记两个backend为不健康
    metrics.record_failure("test-provider:test-model");
    metrics.record_failure("backup-provider:backup-model");
    
    assert!(!metrics.is_healthy("test-provider", "test-model"));
    assert!(!metrics.is_healthy("backup-provider", "backup-model"));
    
    println!("❌ Both backends marked as unhealthy");
    
    // 验证不健康列表包含两个backend
    let unhealthy_before = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy_before.len(), 2);
    
    println!("📋 Unhealthy list contains {} backends", unhealthy_before.len());
    
    // 模拟第一个backend的成功请求
    service.record_request_result(
        "test-provider",
        "test-model", 
        RequestResult::Success { latency: Duration::from_millis(100) }
    ).await;
    
    println!("✅ First backend request succeeded");
    
    // 验证第一个backend恢复，第二个仍然不健康
    assert!(metrics.is_healthy("test-provider", "test-model"));
    assert!(!metrics.is_healthy("backup-provider", "backup-model"));
    
    let unhealthy_middle = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy_middle.len(), 1);
    assert_eq!(unhealthy_middle[0].backend_key, "backup-provider:backup-model");
    
    println!("📊 One backend recovered, one still unhealthy");
    
    // 模拟第二个backend的成功请求
    service.record_request_result(
        "backup-provider",
        "backup-model", 
        RequestResult::Success { latency: Duration::from_millis(200) }
    ).await;
    
    println!("✅ Second backend request succeeded");
    
    // 验证两个backend都恢复
    assert!(metrics.is_healthy("test-provider", "test-model"));
    assert!(metrics.is_healthy("backup-provider", "backup-model"));
    
    let unhealthy_after = metrics.get_unhealthy_backends();
    assert_eq!(unhealthy_after.len(), 0);
    
    println!("🎉 Both backends recovered, unhealthy list empty");
    
    // 停止服务
    service.stop().await;
}

#[tokio::test]
async fn test_backend_selection_after_auto_recovery() {
    let config = create_auto_recovery_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    let metrics = service.get_metrics();
    
    // 标记高权重的backend为不健康
    metrics.record_failure("test-provider:test-model");
    
    // 多次选择，应该只选择健康的backup-provider
    let mut selections_before = HashMap::new();
    for _ in 0..50 {
        if let Ok(backend) = service.select_backend("demo-model").await {
            let key = format!("{}:{}", backend.backend.provider, backend.backend.model);
            *selections_before.entry(key).or_insert(0) += 1;
        }
    }
    
    println!("Selections before recovery: {:?}", selections_before);
    assert_eq!(selections_before.get("test-provider:test-model").unwrap_or(&0), &0);
    assert!(*selections_before.get("backup-provider:backup-model").unwrap_or(&0) > 0);
    
    // 模拟test-provider恢复
    service.record_request_result(
        "test-provider",
        "test-model", 
        RequestResult::Success { latency: Duration::from_millis(100) }
    ).await;
    
    println!("✅ test-provider recovered");
    
    // 再次选择，现在应该根据权重分配
    let mut selections_after = HashMap::new();
    for _ in 0..100 {
        if let Ok(backend) = service.select_backend("demo-model").await {
            let key = format!("{}:{}", backend.backend.provider, backend.backend.model);
            *selections_after.entry(key).or_insert(0) += 1;
        }
    }
    
    println!("Selections after recovery: {:?}", selections_after);
    
    // 验证test-provider（权重0.7）被选择更多
    let test_count = selections_after.get("test-provider:test-model").unwrap_or(&0);
    let backup_count = selections_after.get("backup-provider:backup-model").unwrap_or(&0);
    
    assert!(*test_count > *backup_count, "test-provider should be selected more due to higher weight");
    
    println!("🎯 Weight-based selection working after recovery");
    
    // 停止服务
    service.stop().await;
}
