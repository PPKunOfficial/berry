use crate::config::model::{Config, Backend};
use super::{LoadBalanceManager, HealthChecker, MetricsCollector};
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, error, debug, warn};

/// 负载均衡服务
/// 整合负载均衡管理器和健康检查器，提供统一的服务接口
pub struct LoadBalanceService {
    manager: Arc<LoadBalanceManager>,
    health_checker: Arc<HealthChecker>,
    metrics: Arc<MetricsCollector>,
    is_running: Arc<RwLock<bool>>,
}

impl LoadBalanceService {
    /// 创建新的负载均衡服务
    pub fn new(config: Config) -> Result<Self> {
        // 验证配置
        config.validate()?;

        let manager = Arc::new(LoadBalanceManager::new(config.clone()));
        let metrics = manager.get_metrics();
        let health_checker = Arc::new(HealthChecker::new(
            manager.get_config(),
            metrics.clone(),
        ));

        Ok(Self {
            manager,
            health_checker,
            metrics,
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// 启动负载均衡服务
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        info!("Starting load balance service");

        // 初始化管理器
        self.manager.initialize().await?;

        // 启动健康检查器
        let health_checker = self.health_checker.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            while *is_running.read().await {
                if let Err(e) = health_checker.check_now().await {
                    error!("Health check failed: {}", e);
                }

                // 等待下一次检查
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        // 启动恢复检查器
        let recovery_checker = self.health_checker.clone();
        let is_running_recovery = self.is_running.clone();

        tokio::spawn(async move {
            while *is_running_recovery.read().await {
                if let Err(e) = recovery_checker.check_recovery().await {
                    error!("Recovery check failed: {}", e);
                }

                // 等待下一次恢复检查（通常比健康检查间隔更长）
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });

        info!("Load balance service started successfully");
        Ok(())
    }

    /// 停止负载均衡服务
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
        info!("Load balance service stopped");
    }

    /// 为指定模型选择后端（带智能重试）
    pub async fn select_backend(&self, model_name: &str) -> Result<SelectedBackend> {
        let start_time = Instant::now();
        let max_retries = self.manager.get_config().settings.max_internal_retries;

        debug!("Selecting backend for model: {} (max retries: {})", model_name, max_retries);

        for attempt in 0..=max_retries {
            debug!("Backend selection attempt {} for model '{}'", attempt + 1, model_name);

            match self.manager.select_backend(model_name).await {
                Ok(backend) => {
                    debug!("Load balancer selected backend: {}:{}", backend.provider, backend.model);

                    // 检查选中的backend是否健康
                    let is_healthy = self.metrics.is_healthy(&backend.provider, &backend.model);
                    debug!("Health check for {}:{}: {}", backend.provider, backend.model,
                           if is_healthy { "HEALTHY" } else { "UNHEALTHY" });

                    if is_healthy {
                        let selection_time = start_time.elapsed();

                        debug!(
                            "Selected healthy backend for model '{}': provider='{}', model='{}', selection_time={}ms",
                            model_name,
                            backend.provider,
                            backend.model,
                            selection_time.as_millis()
                        );

                        // 获取provider配置
                        let config = self.manager.get_config();
                        let provider = config
                            .get_provider(&backend.provider)
                            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", backend.provider))?;

                        debug!("Successfully resolved provider config for: {}", backend.provider);
                        return Ok(SelectedBackend {
                            backend,
                            provider: provider.clone(),
                            selection_time,
                        });
                    } else if attempt < max_retries {
                        debug!("Selected backend {}:{} is unhealthy, retrying... (attempt {}/{})",
                               backend.provider, backend.model, attempt + 1, max_retries + 1);
                        continue;
                    } else {
                        // 最后一次尝试，即使不健康也返回
                        warn!("All retries exhausted, returning unhealthy backend {}:{}",
                              backend.provider, backend.model);
                        debug!("No more retry attempts available, using unhealthy backend as last resort");

                        let selection_time = start_time.elapsed();
                        let config = self.manager.get_config();
                        let provider = config
                            .get_provider(&backend.provider)
                            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", backend.provider))?;

                        return Ok(SelectedBackend {
                            backend,
                            provider: provider.clone(),
                            selection_time,
                        });
                    }
                }
                Err(e) => {
                    debug!("Backend selection failed: {}", e);
                    if attempt < max_retries {
                        debug!("Backend selection failed, retrying... (attempt {}/{}): {}",
                               attempt + 1, max_retries + 1, e);
                        continue;
                    } else {
                        debug!("All backend selection attempts failed");
                        return Err(e);
                    }
                }
            }
        }

        anyhow::bail!("Failed to select backend after {} attempts", max_retries + 1)
    }

    /// 记录请求结果
    pub async fn record_request_result(
        &self,
        provider: &str,
        model: &str,
        result: RequestResult,
    ) {
        match result {
            RequestResult::Success { latency } => {
                self.manager.record_success(provider, model, latency);
                debug!(
                    "Recorded success for {}:{} with latency {}ms",
                    provider,
                    model,
                    latency.as_millis()
                );
            }
            RequestResult::Failure { error } => {
                self.manager.record_failure(provider, model);
                debug!(
                    "Recorded failure for {}:{} with error: {}",
                    provider,
                    model,
                    error
                );
            }
        }
    }

    /// 获取所有可用的模型列表
    pub fn get_available_models(&self) -> Vec<String> {
        self.manager.get_available_models()
    }

    /// 获取服务健康状态
    pub async fn get_service_health(&self) -> ServiceHealth {
        let health_summary = self.health_checker.get_health_summary();
        let model_stats = self.manager.get_health_stats().await;
        let is_running = *self.is_running.read().await;

        ServiceHealth {
            is_running,
            health_summary,
            model_stats,
            total_requests: 0, // TODO: 实现请求计数
            successful_requests: 0, // TODO: 实现成功请求计数
        }
    }

    /// 手动触发健康检查
    pub async fn trigger_health_check(&self) -> Result<()> {
        self.health_checker.check_now().await
    }

    /// 重新加载配置
    pub async fn reload_config(&self, new_config: Config) -> Result<()> {
        info!("Reloading load balance service configuration");
        
        // 验证新配置
        new_config.validate()?;
        
        // 重新加载管理器配置
        self.manager.reload_config(new_config).await?;
        
        info!("Configuration reloaded successfully");
        Ok(())
    }

    /// 获取指标收集器
    pub fn get_metrics(&self) -> Arc<MetricsCollector> {
        self.metrics.clone()
    }

    /// 检查服务是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

/// 选中的后端信息
#[derive(Debug, Clone)]
pub struct SelectedBackend {
    pub backend: Backend,
    pub provider: crate::config::model::Provider,
    pub selection_time: Duration,
}

impl SelectedBackend {
    /// 获取完整的API URL
    pub fn get_api_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.provider.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'))
    }

    /// 获取API密钥
    pub fn get_api_key(&self) -> Result<String> {
        if self.provider.api_key.is_empty() {
            anyhow::bail!("API key is empty for provider: {}", self.provider.name);
        }
        Ok(self.provider.api_key.clone())
    }

    /// 获取请求头
    pub fn get_headers(&self) -> std::collections::HashMap<String, String> {
        self.provider.headers.clone()
    }

    /// 获取超时设置
    pub fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.provider.timeout_seconds)
    }
}

/// 请求结果
#[derive(Debug, Clone)]
pub enum RequestResult {
    Success { latency: Duration },
    Failure { error: String },
}

/// 服务健康状态
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub is_running: bool,
    pub health_summary: super::health_checker::HealthSummary,
    pub model_stats: std::collections::HashMap<String, super::manager::HealthStats>,
    pub total_requests: u64,
    pub successful_requests: u64,
}

impl ServiceHealth {
    /// 检查服务是否健康
    pub fn is_healthy(&self) -> bool {
        self.is_running && self.health_summary.is_system_healthy()
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_requests > 0 {
            self.successful_requests as f64 / self.total_requests as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::{Provider, ModelMapping, LoadBalanceStrategy, GlobalSettings};
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
            settings: GlobalSettings::default(),
        }
    }

    #[tokio::test]
    async fn test_service_creation_and_start() {
        unsafe { std::env::set_var("TEST_API_KEY", "test-key"); }
        
        let config = create_test_config();
        let service = LoadBalanceService::new(config).unwrap();
        
        assert!(!service.is_running().await);
        
        service.start().await.unwrap();
        assert!(service.is_running().await);
        
        service.stop().await;
        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_backend_selection() {
        unsafe { std::env::set_var("TEST_API_KEY", "test-key"); }
        
        let config = create_test_config();
        let service = LoadBalanceService::new(config).unwrap();
        service.start().await.unwrap();
        
        let selected = service.select_backend("test-model").await.unwrap();
        assert_eq!(selected.backend.provider, "test-provider");
        assert_eq!(selected.backend.model, "test-model");
        
        service.stop().await;
    }
}
