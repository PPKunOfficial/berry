use crate::config::model::{Config, Backend, ModelMapping};
use super::{BackendSelector, MetricsCollector};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 负载均衡管理器
/// 负责管理所有模型的负载均衡选择器和指标收集
pub struct LoadBalanceManager {
    config: Arc<Config>,
    selectors: Arc<RwLock<HashMap<String, BackendSelector>>>,
    metrics: Arc<MetricsCollector>,
}

impl LoadBalanceManager {
    /// 创建新的负载均衡管理器
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let metrics = Arc::new(MetricsCollector::new());
        let selectors = Arc::new(RwLock::new(HashMap::new()));

        Self {
            config,
            selectors,
            metrics,
        }
    }

    /// 初始化所有模型的选择器
    pub async fn initialize(&self) -> Result<()> {
        let mut selectors = self.selectors.write().await;
        
        for (model_id, model_mapping) in &self.config.models {
            if model_mapping.enabled {
                let selector = BackendSelector::new(
                    model_mapping.clone(),
                    self.metrics.clone(),
                );
                selectors.insert(model_id.clone(), selector);
            }
        }

        tracing::info!("Initialized {} model selectors", selectors.len());
        Ok(())
    }

    /// 为指定模型选择后端
    pub async fn select_backend(&self, model_name: &str) -> Result<Backend> {
        // 首先尝试通过模型ID查找
        if let Some(selector) = self.selectors.read().await.get(model_name) {
            return selector.select();
        }

        // 如果没找到，尝试通过模型的真实名称查找
        for (_, selector) in self.selectors.read().await.iter() {
            if selector.get_model_name() == model_name {
                return selector.select();
            }
        }

        anyhow::bail!("Model '{}' not found or not enabled", model_name)
    }

    /// 获取指定模型的配置
    pub fn get_model_config(&self, model_name: &str) -> Option<&ModelMapping> {
        self.config.get_model(model_name)
    }

    /// 获取所有可用的模型列表
    pub fn get_available_models(&self) -> Vec<String> {
        self.config.get_available_models()
    }

    /// 记录请求成功
    pub fn record_success(&self, provider: &str, model: &str, latency: std::time::Duration) {
        let backend_key = format!("{}:{}", provider, model);
        self.metrics.record_latency(&backend_key, latency);
        self.metrics.record_success(&backend_key);
    }

    /// 记录请求失败
    pub fn record_failure(&self, provider: &str, model: &str) {
        let backend_key = format!("{}:{}", provider, model);
        self.metrics.record_failure(&backend_key);
    }

    /// 获取指标收集器的引用
    pub fn get_metrics(&self) -> Arc<MetricsCollector> {
        self.metrics.clone()
    }

    /// 重新加载配置
    pub async fn reload_config(&self, new_config: Config) -> Result<()> {
        // 验证新配置
        new_config.validate()?;

        // 更新配置
        let _old_config = std::mem::replace(
            &mut *Arc::get_mut(&mut self.config.clone()).unwrap(),
            new_config
        );

        // 重新初始化选择器
        self.initialize().await?;

        tracing::info!("Configuration reloaded successfully");
        Ok(())
    }

    /// 获取模型的健康状态统计
    pub async fn get_health_stats(&self) -> HashMap<String, HealthStats> {
        let mut stats = HashMap::new();
        
        for (model_id, selector) in self.selectors.read().await.iter() {
            let mut healthy_backends = 0;
            let mut total_backends = 0;
            let mut total_latency = std::time::Duration::ZERO;
            let mut latency_count = 0;

            for backend in &selector.get_mapping().backends {
                if backend.enabled {
                    total_backends += 1;
                    
                    if self.metrics.is_healthy(&backend.provider, &backend.model) {
                        healthy_backends += 1;
                    }

                    if let Some(latency) = self.metrics.get_latency(&backend.provider, &backend.model) {
                        total_latency += latency;
                        latency_count += 1;
                    }
                }
            }

            let avg_latency = if latency_count > 0 {
                Some(total_latency / latency_count as u32)
            } else {
                None
            };

            stats.insert(model_id.clone(), HealthStats {
                healthy_backends,
                total_backends,
                health_ratio: if total_backends > 0 {
                    healthy_backends as f64 / total_backends as f64
                } else {
                    0.0
                },
                average_latency: avg_latency,
            });
        }

        stats
    }

    /// 获取配置的引用
    pub fn get_config(&self) -> Arc<Config> {
        self.config.clone()
    }
}

/// 健康状态统计
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStats {
    pub healthy_backends: usize,
    pub total_backends: usize,
    pub health_ratio: f64,
    pub average_latency: Option<std::time::Duration>,
}

impl HealthStats {
    /// 检查模型是否健康
    pub fn is_healthy(&self) -> bool {
        self.health_ratio > 0.0
    }

    /// 检查模型是否完全健康
    pub fn is_fully_healthy(&self) -> bool {
        self.health_ratio >= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::{Provider, LoadBalanceStrategy};
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut providers = HashMap::new();
        providers.insert("test-provider".to_string(), Provider {
            name: "Test Provider".to_string(),
            base_url: "https://api.test.com".to_string(),
            api_key: "test-api-key".to_string(),
            models: vec!["test-model".to_string()],
            headers: HashMap::new(),
            enabled: true,
            timeout_seconds: 30,
            max_retries: 3,
        });

        let mut models = HashMap::new();
        models.insert("test-model".to_string(), ModelMapping {
            name: "test-model".to_string(),
            backends: vec![Backend {
                provider: "test-provider".to_string(),
                model: "test-model".to_string(),
                weight: 1.0,
                priority: 1,
                enabled: true,
                tags: vec![],
            }],
            strategy: LoadBalanceStrategy::WeightedRandom,
            enabled: true,
        });

        Config {
            providers,
            models,
            users: HashMap::new(),
            settings: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_manager_initialization() {
        let config = create_test_config();
        let manager = LoadBalanceManager::new(config);
        
        assert!(manager.initialize().await.is_ok());
        
        let models = manager.get_available_models();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0], "test-model");
    }

    #[tokio::test]
    async fn test_backend_selection() {
        let config = create_test_config();
        let manager = LoadBalanceManager::new(config);
        manager.initialize().await.unwrap();
        
        let backend = manager.select_backend("test-model").await;
        assert!(backend.is_ok());
        
        let backend = backend.unwrap();
        assert_eq!(backend.provider, "test-provider");
        assert_eq!(backend.model, "test-model");
    }
}
