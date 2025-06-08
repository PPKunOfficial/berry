use crate::config::model::{Config, Provider};
use crate::relay::client::openai::OpenAIClient;
use super::MetricsCollector;
use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// 健康检查器
/// 定期检查所有provider的健康状态
pub struct HealthChecker {
    config: Arc<Config>,
    metrics: Arc<MetricsCollector>,
    client: Client,
    check_interval: Duration,
    initial_check_done: Arc<std::sync::RwLock<bool>>,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(config: Arc<Config>, metrics: Arc<MetricsCollector>) -> Self {
        let check_interval = Duration::from_secs(config.settings.health_check_interval_seconds);
        let timeout = Duration::from_secs(config.settings.request_timeout_seconds);
        
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            metrics,
            client,
            check_interval,
            initial_check_done: Arc::new(std::sync::RwLock::new(false)),
        }
    }

    /// 启动健康检查循环
    pub async fn start(&self) {
        info!("Starting health checker with interval: {:?}", self.check_interval);
        
        let mut interval = interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_all_providers().await {
                error!("Health check failed: {}", e);
            }
        }
    }

    /// 检查所有provider的健康状态
    async fn check_all_providers(&self) -> Result<()> {
        let enabled_providers: Vec<_> = self.config.providers.iter()
            .filter(|(_, provider)| provider.enabled)
            .collect();

        debug!("Starting health check for {} enabled providers", enabled_providers.len());

        // 检查是否是初始检查
        let is_initial_check = {
            let initial_done = self.initial_check_done.read().unwrap();
            !*initial_done
        };

        if is_initial_check {
            info!("Performing initial health check - marking all enabled providers as healthy");
        } else {
            debug!("Performing routine health check - only checking currently healthy providers");
        }

        let mut tasks = Vec::new();

        for (provider_id, provider) in enabled_providers {
            debug!("Scheduling health check for provider: {} ({})", provider_id, provider.name);

            let provider_id_clone = provider_id.clone();
            let provider_clone = provider.clone();
            let client = self.client.clone();
            let metrics = self.metrics.clone();
            let is_initial = is_initial_check;

            let task = tokio::spawn(async move {
                debug!("Starting health check task for provider: {}", provider_id_clone);
                Self::check_provider_health(&provider_id_clone, &provider_clone, &client, &metrics, is_initial).await;
                debug!("Completed health check task for provider: {}", provider_id_clone);
            });

            tasks.push((provider_id.clone(), task));
        }

        // 等待所有健康检查完成
        debug!("Waiting for {} health check tasks to complete", tasks.len());
        for (provider_id, task) in tasks {
            if let Err(e) = task.await {
                error!("Health check task failed for provider {}: {}", provider_id, e);
            } else {
                debug!("Health check task completed successfully for provider: {}", provider_id);
            }
        }

        // 标记初始检查已完成
        if is_initial_check {
            let mut initial_done = self.initial_check_done.write().unwrap();
            *initial_done = true;
            info!("Initial health check completed - subsequent checks will require chat validation for recovery");
        }

        debug!("Completed health check for all providers");
        Ok(())
    }

    /// 检查单个provider的健康状态
    async fn check_provider_health(
        provider_id: &str,
        provider: &Provider,
        client: &Client,
        metrics: &MetricsCollector,
        is_initial_check: bool,
    ) {
        let start_time = Instant::now();
        debug!("Starting health check for provider: {} (base_url: {})", provider_id, provider.base_url);

        // 直接使用配置中的API密钥
        let api_key = &provider.api_key;

        if api_key.is_empty() {
            warn!("API key is empty for provider {}", provider_id);
            debug!("Marking {} models as unhealthy due to empty API key", provider.models.len());
            // 标记所有模型为不健康
            for model in &provider.models {
                let backend_key = format!("{}:{}", provider_id, model);
                debug!("Marking backend {} as unhealthy (empty API key)", backend_key);
                metrics.record_failure(&backend_key);
            }
            return;
        }

        debug!("API key present for provider {}, proceeding with health check", provider_id);

        // 使用真实的API检查
        if provider.base_url.contains("httpbin.org") {
            debug!("Detected test provider (httpbin), using HTTP status check for {}", provider_id);
            // 对于测试服务，使用httpbin的状态端点
            Self::check_test_provider(provider_id, provider, client, metrics, start_time, is_initial_check).await;
        } else {
            debug!("Detected real AI provider, using models API check for {}", provider_id);
            // 对于真实的AI服务，使用model list API检查
            Self::check_real_provider(provider_id, provider, metrics, start_time, is_initial_check).await;
        }

        let total_time = start_time.elapsed();
        debug!("Completed health check for provider {} in {}ms", provider_id, total_time.as_millis());
    }

    /// 检查测试provider（httpbin等）
    async fn check_test_provider(
        provider_id: &str,
        provider: &Provider,
        client: &Client,
        metrics: &MetricsCollector,
        start_time: Instant,
        is_initial_check: bool,
    ) {
        let health_check_url = format!("{}/status/200", provider.base_url);
        debug!("Testing provider {} with URL: {}", provider_id, health_check_url);

        let mut request = client.get(&health_check_url);

        // 添加自定义头部
        if !provider.headers.is_empty() {
            debug!("Adding {} custom headers for provider {}", provider.headers.len(), provider_id);
            for (key, value) in &provider.headers {
                debug!("Adding header: {} = {}", key, value);
                request = request.header(key, value);
            }
        }

        debug!("Sending HTTP request to test provider {}", provider_id);
        // 发送请求
        match request.send().await {
            Ok(response) => {
                let latency = start_time.elapsed();
                let status = response.status();
                debug!("Received response from provider {} with status: {} ({}ms)", provider_id, status, latency.as_millis());

                if status.is_success() {
                    if is_initial_check {
                        debug!("Provider {} initial health check passed, marking {} models as healthy", provider_id, provider.models.len());

                        // 初始检查：标记所有模型为健康
                        for model in &provider.models {
                            let backend_key = format!("{}:{}", provider_id, model);
                            debug!("Initial check: Marking backend {} as healthy (latency: {}ms)", backend_key, latency.as_millis());
                            metrics.record_latency(&backend_key, latency);
                            metrics.record_success(&backend_key);
                            metrics.update_health_check(&backend_key);
                        }
                    } else {
                        debug!("Provider {} routine health check passed, but not marking as healthy (requires chat validation)", provider_id);
                        // 后续检查：成功但不自动标记为健康，只更新延迟
                        for model in &provider.models {
                            let backend_key = format!("{}:{}", provider_id, model);

                            // 检查当前是否在不健康列表中
                            if metrics.is_in_unhealthy_list(&backend_key) {
                                debug!("Routine check: Backend {} is in unhealthy list, not auto-recovering (requires chat validation)", backend_key);
                                // 只更新延迟和检查时间，不改变健康状态
                                metrics.record_latency(&backend_key, latency);
                                metrics.update_health_check(&backend_key);
                            } else {
                                debug!("Routine check: Backend {} is healthy, maintaining status", backend_key);
                                // 对于已经健康的backend，正常更新
                                metrics.record_latency(&backend_key, latency);
                                metrics.update_health_check(&backend_key);
                                // 注意：不调用 record_success 避免重复标记
                            }
                        }
                    }
                } else {
                    warn!("Provider {} health check failed with status: {}", provider_id, status);
                    debug!("Marking {} models as unhealthy for provider {}", provider.models.len(), provider_id);

                    // 无论初始还是后续检查，失败都标记为不健康
                    for model in &provider.models {
                        let backend_key = format!("{}:{}", provider_id, model);
                        debug!("Marking backend {} as unhealthy (HTTP {})", backend_key, status);
                        metrics.record_failure(&backend_key);
                    }
                }
            }
            Err(e) => {
                error!("Provider {} health check error: {}", provider_id, e);
                debug!("Network error for provider {}, marking {} models as unhealthy", provider_id, provider.models.len());

                // 标记所有模型为不健康
                for model in &provider.models {
                    let backend_key = format!("{}:{}", provider_id, model);
                    debug!("Marking backend {} as unhealthy (network error: {})", backend_key, e);
                    metrics.record_failure(&backend_key);
                }
            }
        }
    }

    /// 检查真实的AI provider
    async fn check_real_provider(
        provider_id: &str,
        provider: &Provider,
        metrics: &MetricsCollector,
        start_time: Instant,
        is_initial_check: bool,
    ) {
        debug!("Checking real AI provider {} using models API", provider_id);
        let openai_client = OpenAIClient::with_base_url(provider.base_url.clone());

        debug!("Sending models API request to provider {} (base_url: {})", provider_id, provider.base_url);
        // 使用models API检查provider健康状态
        match openai_client.models(&provider.api_key).await {
            Ok(response) => {
                let latency = start_time.elapsed();
                debug!("Received models API response from provider {} ({}ms)", provider_id, latency.as_millis());

                if response.is_success {
                    if is_initial_check {
                        debug!("Provider {} initial models API check passed, marking {} models as healthy", provider_id, provider.models.len());

                        // 初始检查：标记所有模型为健康
                        for model in &provider.models {
                            let backend_key = format!("{}:{}", provider_id, model);
                            debug!("Initial check: Marking backend {} as healthy (models API success, latency: {}ms)", backend_key, latency.as_millis());
                            metrics.record_latency(&backend_key, latency);
                            metrics.record_success(&backend_key);
                            metrics.update_health_check(&backend_key);
                        }
                    } else {
                        debug!("Provider {} routine models API check passed, but not marking as healthy (requires chat validation)", provider_id);
                        // 后续检查：成功但不自动标记为健康，只更新延迟
                        for model in &provider.models {
                            let backend_key = format!("{}:{}", provider_id, model);

                            // 检查当前是否在不健康列表中
                            if metrics.is_in_unhealthy_list(&backend_key) {
                                debug!("Routine check: Backend {} is in unhealthy list, not auto-recovering (requires chat validation)", backend_key);
                                // 只更新延迟和检查时间，不改变健康状态
                                metrics.record_latency(&backend_key, latency);
                                metrics.update_health_check(&backend_key);
                            } else {
                                debug!("Routine check: Backend {} is healthy, maintaining status", backend_key);
                                // 对于已经健康的backend，正常更新
                                metrics.record_latency(&backend_key, latency);
                                metrics.update_health_check(&backend_key);
                                // 注意：不调用 record_success 避免重复标记
                            }
                        }
                    }
                } else {
                    warn!("Provider {} models API check failed: {}", provider_id, response.body);
                    debug!("Models API failed for provider {}, marking {} models as unhealthy", provider_id, provider.models.len());

                    // 无论初始还是后续检查，失败都标记为不健康
                    for model in &provider.models {
                        let backend_key = format!("{}:{}", provider_id, model);
                        debug!("Marking backend {} as unhealthy (models API failed)", backend_key);
                        metrics.record_failure(&backend_key);
                    }
                }
            }
            Err(e) => {
                error!("Provider {} models API error: {}", provider_id, e);
                debug!("Network/API error for provider {}, marking {} models as unhealthy", provider_id, provider.models.len());

                // 标记所有模型为不健康
                for model in &provider.models {
                    let backend_key = format!("{}:{}", provider_id, model);
                    debug!("Marking backend {} as unhealthy (API error: {})", backend_key, e);
                    metrics.record_failure(&backend_key);
                }
            }
        }
    }

    /// 手动触发健康检查
    pub async fn check_now(&self) -> Result<()> {
        info!("Manual health check triggered");
        self.check_all_providers().await
    }

    /// 检查特定provider的健康状态
    pub async fn check_provider(&self, provider_id: &str) -> Result<()> {
        if let Some(provider) = self.config.providers.get(provider_id) {
            if provider.enabled {
                Self::check_provider_health(
                    provider_id,
                    provider,
                    &self.client,
                    &self.metrics,
                    false, // 手动触发的检查不是初始检查
                ).await;
                Ok(())
            } else {
                anyhow::bail!("Provider '{}' is disabled", provider_id);
            }
        } else {
            anyhow::bail!("Provider '{}' not found", provider_id);
        }
    }

    /// 检查不健康的provider是否可以恢复
    pub async fn check_recovery(&self) -> Result<()> {
        let recovery_interval = Duration::from_secs(self.config.settings.recovery_check_interval_seconds);
        let unhealthy_backends = self.metrics.get_unhealthy_backends();

        debug!("Starting recovery check process (interval: {}s)", recovery_interval.as_secs());

        if unhealthy_backends.is_empty() {
            debug!("No unhealthy backends to check for recovery");
            return Ok(());
        }

        info!("Checking recovery for {} unhealthy backends", unhealthy_backends.len());
        debug!("Unhealthy backends: {:?}", unhealthy_backends.iter().map(|b| &b.backend_key).collect::<Vec<_>>());

        for unhealthy_backend in unhealthy_backends {
            debug!("Evaluating recovery check for backend: {} (failed {} times, last failure: {:?})",
                   unhealthy_backend.backend_key,
                   unhealthy_backend.failure_count,
                   unhealthy_backend.last_failure_time.elapsed());

            if self.metrics.needs_recovery_check(&unhealthy_backend.backend_key, recovery_interval) {
                debug!("Backend {} needs recovery check", unhealthy_backend.backend_key);

                // 解析backend_key获取provider_id和model
                let parts: Vec<&str> = unhealthy_backend.backend_key.split(':').collect();
                if parts.len() != 2 {
                    warn!("Invalid backend key format: {}", unhealthy_backend.backend_key);
                    continue;
                }

                let provider_id = parts[0];
                let model_name = parts[1];

                debug!("Parsed backend key: provider={}, model={}", provider_id, model_name);

                if let Some(provider) = self.config.providers.get(provider_id) {
                    if provider.enabled {
                        info!("Attempting recovery check for {}:{}", provider_id, model_name);
                        debug!("Recording recovery attempt for backend: {}", unhealthy_backend.backend_key);
                        self.metrics.record_recovery_attempt(&unhealthy_backend.backend_key);

                        // 使用chat请求进行恢复检查
                        self.check_recovery_with_chat(provider_id, provider, model_name).await;
                    } else {
                        debug!("Provider {} is disabled, skipping recovery check", provider_id);
                    }
                } else {
                    warn!("Provider {} not found in config, cannot perform recovery check", provider_id);
                }
            } else {
                debug!("Backend {} does not need recovery check yet (last attempt: {:?})",
                       unhealthy_backend.backend_key,
                       unhealthy_backend.last_recovery_attempt.map(|t| t.elapsed()));
            }
        }

        debug!("Completed recovery check process");
        Ok(())
    }

    /// 使用chat请求检查provider恢复状态
    async fn check_recovery_with_chat(
        &self,
        provider_id: &str,
        provider: &Provider,
        model_name: &str,
    ) {
        let start_time = Instant::now();
        debug!("Starting chat-based recovery check for {}:{}", provider_id, model_name);

        let openai_client = OpenAIClient::with_base_url(provider.base_url.clone());
        debug!("Created OpenAI client for recovery check (base_url: {})", provider.base_url);

        // 构建简单的chat请求
        let test_body = json!({
            "model": model_name,
            "messages": [
                {
                    "role": "user",
                    "content": "Hello"
                }
            ],
            "max_tokens": 1,
            "stream": false
        });
        debug!("Built test chat request for recovery check: {}", test_body);

        // 构建请求头
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", provider.api_key).parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        debug!("Added basic headers for recovery check (Authorization, Content-Type)");

        // 添加自定义头部
        if !provider.headers.is_empty() {
            debug!("Adding {} custom headers for recovery check", provider.headers.len());
            for (key, value) in &provider.headers {
                if let (Ok(header_name), Ok(header_value)) = (
                    key.parse::<reqwest::header::HeaderName>(),
                    value.parse::<reqwest::header::HeaderValue>()
                ) {
                    debug!("Adding custom header for recovery: {} = {}", key, value);
                    headers.insert(header_name, header_value);
                } else {
                    warn!("Failed to parse custom header for recovery check: {} = {}", key, value);
                }
            }
        }

        debug!("Sending chat request for recovery check to {}:{}", provider_id, model_name);
        match openai_client.chat_completions(headers, &test_body).await {
            Ok(response) => {
                let latency = start_time.elapsed();
                let backend_key = format!("{}:{}", provider_id, model_name);
                let status = response.status();

                debug!("Received chat response for recovery check: status={}, latency={}ms", status, latency.as_millis());

                if status.is_success() {
                    info!("Recovery check passed for {}:{} ({}ms)", provider_id, model_name, latency.as_millis());
                    debug!("Marking backend {} as recovered and healthy", backend_key);

                    // 恢复成功，标记为健康
                    self.metrics.record_latency(&backend_key, latency);
                    self.metrics.record_success(&backend_key);
                    self.metrics.update_health_check(&backend_key);

                    debug!("Successfully restored backend {} to healthy state", backend_key);
                } else {
                    warn!("Recovery check failed for {}:{} with status: {}", provider_id, model_name, status);
                    debug!("Backend {} remains unhealthy after recovery attempt", backend_key);
                    // 保持不健康状态
                }
            }
            Err(e) => {
                error!("Recovery check error for {}:{}: {}", provider_id, model_name, e);
                debug!("Network/API error during recovery check for {}:{}: {}", provider_id, model_name, e);
                // 保持不健康状态
            }
        }

        let total_time = start_time.elapsed();
        debug!("Completed recovery check for {}:{} in {}ms", provider_id, model_name, total_time.as_millis());
    }

    /// 获取健康检查统计信息
    pub fn get_health_summary(&self) -> HealthSummary {
        let mut total_providers = 0;
        let mut healthy_providers = 0;
        let mut total_models = 0;
        let mut healthy_models = 0;

        for (provider_id, provider) in &self.config.providers {
            if provider.enabled {
                total_providers += 1;
                let mut provider_healthy = true;

                for model in &provider.models {
                    total_models += 1;

                    if self.metrics.is_healthy(provider_id, model) {
                        healthy_models += 1;
                    } else {
                        provider_healthy = false;
                    }
                }

                if provider_healthy {
                    healthy_providers += 1;
                }
            }
        }

        HealthSummary {
            total_providers,
            healthy_providers,
            total_models,
            healthy_models,
            provider_health_ratio: if total_providers > 0 {
                healthy_providers as f64 / total_providers as f64
            } else {
                0.0
            },
            model_health_ratio: if total_models > 0 {
                healthy_models as f64 / total_models as f64
            } else {
                0.0
            },
        }
    }
}

/// 健康检查摘要
#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub total_providers: usize,
    pub healthy_providers: usize,
    pub total_models: usize,
    pub healthy_models: usize,
    pub provider_health_ratio: f64,
    pub model_health_ratio: f64,
}

impl HealthSummary {
    /// 检查整体系统是否健康
    pub fn is_system_healthy(&self) -> bool {
        self.model_health_ratio > 0.5 // 至少50%的模型健康
    }

    /// 检查是否有任何可用的模型
    pub fn has_available_models(&self) -> bool {
        self.healthy_models > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::{GlobalSettings, ModelMapping, Backend, LoadBalanceStrategy};
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut providers = HashMap::new();
        providers.insert("test-provider".to_string(), Provider {
            name: "Test Provider".to_string(),
            base_url: "https://httpbin.org".to_string(), // 使用httpbin进行测试
            api_key: "test-api-key".to_string(),
            models: vec!["test-model".to_string()],
            headers: HashMap::new(),
            enabled: true,
            timeout_seconds: 5,
            max_retries: 1,
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
            settings: GlobalSettings {
                health_check_interval_seconds: 10,
                request_timeout_seconds: 5,
                max_retries: 1,
                circuit_breaker_failure_threshold: 3,
                circuit_breaker_timeout_seconds: 30,
                recovery_check_interval_seconds: 120,
                max_internal_retries: 2,
                health_check_timeout_seconds: 10,
            },
        }
    }

    #[tokio::test]
    async fn test_health_checker_creation() {
        let config = Arc::new(create_test_config());
        let metrics = Arc::new(MetricsCollector::new());
        
        let checker = HealthChecker::new(config, metrics);
        let summary = checker.get_health_summary();
        
        assert_eq!(summary.total_providers, 1);
        assert_eq!(summary.total_models, 1);
    }
}
