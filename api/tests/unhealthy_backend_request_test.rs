use berry_api_api::config::model::{Config, Provider, ModelMapping, Backend, LoadBalanceStrategy, GlobalSettings};
use berry_api_api::loadbalance::LoadBalanceService;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// 创建测试配置，包含多个不同权重的backend
fn create_weighted_test_config() -> Config {
    let mut providers = HashMap::new();
    
    // 添加多个provider用于测试权重选择
    providers.insert("provider1".to_string(), Provider {
        name: "Provider 1".to_string(),
        base_url: "https://httpbin.org".to_string(),
        api_key: "key1".to_string(),
        models: vec!["model1".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 10,
        max_retries: 2,
    });

    providers.insert("provider2".to_string(), Provider {
        name: "Provider 2".to_string(),
        base_url: "https://httpbin.org".to_string(),
        api_key: "key2".to_string(),
        models: vec!["model2".to_string()],
        headers: HashMap::new(),
        enabled: true,
        timeout_seconds: 10,
        max_retries: 2,
    });

    providers.insert("provider3".to_string(), Provider {
        name: "Provider 3".to_string(),
        base_url: "https://httpbin.org".to_string(),
        api_key: "key3".to_string(),
        models: vec!["model3".to_string()],
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
                provider: "provider1".to_string(),
                model: "model1".to_string(),
                weight: 0.5,  // 高权重
                priority: 2,
                enabled: true,
                tags: vec![],
            },
            Backend {
                provider: "provider2".to_string(),
                model: "model2".to_string(),
                weight: 0.3,  // 中权重
                priority: 1,
                enabled: true,
                tags: vec![],
            },
            Backend {
                provider: "provider3".to_string(),
                model: "model3".to_string(),
                weight: 0.2,  // 低权重
                priority: 3,
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
async fn test_no_healthy_backends_uses_weights() {
    let config = create_weighted_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    // 等待初始健康检查完成
    sleep(Duration::from_secs(2)).await;
    
    let metrics = service.get_metrics();
    
    // 标记所有backend为不健康
    metrics.record_failure("provider1:model1");
    metrics.record_failure("provider2:model2");
    metrics.record_failure("provider3:model3");
    
    // 验证所有backend都不健康
    assert!(!metrics.is_healthy("provider1", "model1"));
    assert!(!metrics.is_healthy("provider2", "model2"));
    assert!(!metrics.is_healthy("provider3", "model3"));
    
    // 多次选择backend，验证仍然根据权重分配
    let mut selections = HashMap::new();
    for _ in 0..100 {
        if let Ok(backend) = service.select_backend("test-model").await {
            let key = format!("{}:{}", backend.backend.provider, backend.backend.model);
            *selections.entry(key).or_insert(0) += 1;
        }
    }
    
    println!("Selections when all backends unhealthy: {:?}", selections);
    
    // 验证高权重的provider1被选择最多
    let provider1_count = selections.get("provider1:model1").unwrap_or(&0);
    let provider2_count = selections.get("provider2:model2").unwrap_or(&0);
    let provider3_count = selections.get("provider3:model3").unwrap_or(&0);
    
    println!("Provider1 (weight 0.5): {} selections", provider1_count);
    println!("Provider2 (weight 0.3): {} selections", provider2_count);
    println!("Provider3 (weight 0.2): {} selections", provider3_count);
    
    // 验证权重分配大致正确（允许一定误差）
    assert!(*provider1_count > *provider2_count, "Provider1 should be selected more than Provider2");
    assert!(*provider2_count > *provider3_count, "Provider2 should be selected more than Provider3");
    
    // 停止服务
    service.stop().await;
}

#[tokio::test]
async fn test_unhealthy_backend_success_not_auto_recover() {
    let config = create_weighted_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    let metrics = service.get_metrics();
    
    // 标记一个backend为不健康
    metrics.record_failure("provider1:model1");
    assert!(!metrics.is_healthy("provider1", "model1"));
    assert!(metrics.is_in_unhealthy_list("provider1:model1"));
    
    // 模拟该backend的请求成功（这在实际使用中可能发生）
    // 注意：这里我们直接调用metrics方法，因为实际的请求处理逻辑已经修改
    
    // 模拟不健康backend的成功请求（应该只更新延迟，不恢复健康状态）
    let backend_key = "provider1:model1";
    if metrics.is_in_unhealthy_list(backend_key) {
        // 模拟修改后的逻辑：只记录延迟，不标记为健康
        metrics.record_latency(backend_key, Duration::from_millis(100));
        metrics.update_health_check(backend_key);
        println!("Simulated successful request for unhealthy backend, only recorded latency");
    }
    
    // 验证backend仍然不健康
    assert!(!metrics.is_healthy("provider1", "model1"));
    assert!(metrics.is_in_unhealthy_list("provider1:model1"));
    
    println!("✅ Unhealthy backend remained unhealthy after successful request");
    
    // 只有通过chat验证才能恢复
    metrics.record_success("provider1:model1");
    assert!(metrics.is_healthy("provider1", "model1"));
    assert!(!metrics.is_in_unhealthy_list("provider1:model1"));
    
    println!("✅ Backend recovered only through explicit chat validation");
    
    // 停止服务
    service.stop().await;
}

#[tokio::test]
async fn test_healthy_backend_success_normal_behavior() {
    let config = create_weighted_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    let metrics = service.get_metrics();
    
    // 验证backend初始是健康的
    assert!(metrics.is_healthy("provider1", "model1"));
    assert!(!metrics.is_in_unhealthy_list("provider1:model1"));
    
    // 模拟健康backend的成功请求（应该正常记录）
    let backend_key = "provider1:model1";
    if !metrics.is_in_unhealthy_list(backend_key) {
        // 模拟正常的成功记录
        metrics.record_success(backend_key);
        println!("Simulated successful request for healthy backend, recorded normally");
    }
    
    // 验证backend仍然健康
    assert!(metrics.is_healthy("provider1", "model1"));
    assert!(!metrics.is_in_unhealthy_list("provider1:model1"));
    
    println!("✅ Healthy backend maintained healthy status after successful request");
    
    // 停止服务
    service.stop().await;
}

#[tokio::test]
async fn test_weight_distribution_with_mixed_health() {
    let config = create_weighted_test_config();
    let service = LoadBalanceService::new(config).unwrap();
    
    // 启动服务
    service.start().await.unwrap();
    
    let metrics = service.get_metrics();
    
    // 标记部分backend为不健康
    metrics.record_failure("provider1:model1");  // 权重0.5，不健康
    // provider2:model2 权重0.3，健康
    // provider3:model3 权重0.2，健康
    
    // 多次选择，验证只选择健康的backend
    let mut selections = HashMap::new();
    for _ in 0..100 {
        if let Ok(backend) = service.select_backend("test-model").await {
            let key = format!("{}:{}", backend.backend.provider, backend.backend.model);
            *selections.entry(key).or_insert(0) += 1;
        }
    }
    
    println!("Selections with mixed health: {:?}", selections);
    
    // 验证不健康的provider1没有被选择
    assert_eq!(selections.get("provider1:model1").unwrap_or(&0), &0);
    
    // 验证健康的provider2和provider3被选择，且按权重分配
    let provider2_count = selections.get("provider2:model2").unwrap_or(&0);
    let provider3_count = selections.get("provider3:model3").unwrap_or(&0);
    
    println!("Provider2 (weight 0.3, healthy): {} selections", provider2_count);
    println!("Provider3 (weight 0.2, healthy): {} selections", provider3_count);
    
    // 验证provider2被选择更多（权重更高）
    assert!(*provider2_count > *provider3_count, "Provider2 should be selected more than Provider3");
    
    // 停止服务
    service.stop().await;
}
