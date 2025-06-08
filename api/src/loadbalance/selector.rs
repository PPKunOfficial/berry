use crate::config::model::{Backend, LoadBalanceStrategy, ModelMapping};
use anyhow::Result;
use rand::distributions::{Distribution, WeightedIndex};
use rand::{thread_rng, Rng};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct BackendSelector {
    mapping: ModelMapping,
    round_robin_counter: AtomicUsize,
    metrics: Arc<MetricsCollector>,
}

/// 指标收集器，用于收集后端性能数据
pub struct MetricsCollector {
    latencies: Arc<std::sync::RwLock<HashMap<String, Duration>>>,
    health_status: Arc<std::sync::RwLock<HashMap<String, bool>>>,
    failure_counts: Arc<std::sync::RwLock<HashMap<String, u32>>>,
    last_health_check: Arc<std::sync::RwLock<HashMap<String, Instant>>>,
    // 新增：不健康列表管理
    unhealthy_backends: Arc<std::sync::RwLock<HashMap<String, UnhealthyBackend>>>,
    recovery_attempts: Arc<std::sync::RwLock<HashMap<String, u32>>>,
}

/// 不健康后端信息
#[derive(Debug, Clone)]
pub struct UnhealthyBackend {
    pub backend_key: String,
    pub first_failure_time: Instant,
    pub last_failure_time: Instant,
    pub failure_count: u32,
    pub last_recovery_attempt: Option<Instant>,
    pub recovery_attempts: u32,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            latencies: Arc::new(std::sync::RwLock::new(HashMap::new())),
            health_status: Arc::new(std::sync::RwLock::new(HashMap::new())),
            failure_counts: Arc::new(std::sync::RwLock::new(HashMap::new())),
            last_health_check: Arc::new(std::sync::RwLock::new(HashMap::new())),
            unhealthy_backends: Arc::new(std::sync::RwLock::new(HashMap::new())),
            recovery_attempts: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 记录请求延迟
    pub fn record_latency(&self, backend_key: &str, latency: Duration) {
        if let Ok(mut latencies) = self.latencies.write() {
            latencies.insert(backend_key.to_string(), latency);
        }
    }

    /// 记录请求失败
    pub fn record_failure(&self, backend_key: &str) {
        let now = Instant::now();
        tracing::debug!("Recording failure for backend: {}", backend_key);

        if let Ok(mut failures) = self.failure_counts.write() {
            let count = failures.entry(backend_key.to_string()).or_insert(0);
            *count += 1;
            tracing::debug!("Updated failure count for {}: {}", backend_key, *count);
        }

        // 标记为不健康
        if let Ok(mut health) = self.health_status.write() {
            health.insert(backend_key.to_string(), false);
            tracing::debug!("Marked backend {} as unhealthy", backend_key);
        }

        // 添加到不健康列表
        if let Ok(mut unhealthy) = self.unhealthy_backends.write() {
            match unhealthy.get_mut(backend_key) {
                Some(backend) => {
                    backend.last_failure_time = now;
                    backend.failure_count += 1;
                    tracing::debug!("Updated existing unhealthy backend {}: failure_count={}",
                                   backend_key, backend.failure_count);
                }
                None => {
                    tracing::debug!("Adding new backend {} to unhealthy list", backend_key);
                    unhealthy.insert(backend_key.to_string(), UnhealthyBackend {
                        backend_key: backend_key.to_string(),
                        first_failure_time: now,
                        last_failure_time: now,
                        failure_count: 1,
                        last_recovery_attempt: None,
                        recovery_attempts: 0,
                    });
                }
            }
        }
    }

    /// 记录请求成功
    pub fn record_success(&self, backend_key: &str) {
        tracing::debug!("Recording success for backend: {}", backend_key);

        // 重置失败计数
        if let Ok(mut failures) = self.failure_counts.write() {
            failures.insert(backend_key.to_string(), 0);
            tracing::debug!("Reset failure count for {} to 0", backend_key);
        }

        // 标记为健康
        if let Ok(mut health) = self.health_status.write() {
            health.insert(backend_key.to_string(), true);
            tracing::debug!("Marked backend {} as healthy", backend_key);
        }

        // 从不健康列表中移除
        if let Ok(mut unhealthy) = self.unhealthy_backends.write() {
            if unhealthy.remove(backend_key).is_some() {
                tracing::debug!("Removed backend {} from unhealthy list", backend_key);
            }
        }

        // 重置恢复尝试计数
        if let Ok(mut recovery) = self.recovery_attempts.write() {
            if recovery.remove(backend_key).is_some() {
                tracing::debug!("Reset recovery attempts for backend {}", backend_key);
            }
        }
    }

    /// 检查后端是否健康
    pub fn is_healthy(&self, provider: &str, model: &str) -> bool {
        let backend_key = format!("{}:{}", provider, model);

        if let Ok(health) = self.health_status.read() {
            health.get(&backend_key).copied().unwrap_or(true) // 默认认为是健康的
        } else {
            true
        }
    }

    /// 获取后端延迟
    pub fn get_latency(&self, provider: &str, model: &str) -> Option<Duration> {
        let backend_key = format!("{}:{}", provider, model);

        if let Ok(latencies) = self.latencies.read() {
            latencies.get(&backend_key).copied()
        } else {
            None
        }
    }

    /// 获取失败计数
    pub fn get_failure_count(&self, provider: &str, model: &str) -> u32 {
        let backend_key = format!("{}:{}", provider, model);

        if let Ok(failures) = self.failure_counts.read() {
            failures.get(&backend_key).copied().unwrap_or(0)
        } else {
            0
        }
    }

    /// 更新健康检查时间
    pub fn update_health_check(&self, backend_key: &str) {
        if let Ok(mut last_check) = self.last_health_check.write() {
            last_check.insert(backend_key.to_string(), Instant::now());
        }
    }

    /// 获取所有不健康的后端
    pub fn get_unhealthy_backends(&self) -> Vec<UnhealthyBackend> {
        if let Ok(unhealthy) = self.unhealthy_backends.read() {
            unhealthy.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// 检查后端是否需要恢复检查
    pub fn needs_recovery_check(&self, backend_key: &str, recovery_interval: Duration) -> bool {
        if let Ok(unhealthy) = self.unhealthy_backends.read() {
            if let Some(backend) = unhealthy.get(backend_key) {
                match backend.last_recovery_attempt {
                    Some(last_attempt) => last_attempt.elapsed() >= recovery_interval,
                    None => true, // 从未尝试过恢复
                }
            } else {
                false // 不在不健康列表中
            }
        } else {
            false
        }
    }

    /// 记录恢复尝试
    pub fn record_recovery_attempt(&self, backend_key: &str) {
        let now = Instant::now();
        tracing::debug!("Recording recovery attempt for backend: {}", backend_key);

        if let Ok(mut unhealthy) = self.unhealthy_backends.write() {
            if let Some(backend) = unhealthy.get_mut(backend_key) {
                backend.last_recovery_attempt = Some(now);
                backend.recovery_attempts += 1;
                tracing::debug!("Updated recovery attempt for {}: attempt #{}",
                               backend_key, backend.recovery_attempts);
            } else {
                tracing::warn!("Attempted to record recovery for backend {} not in unhealthy list", backend_key);
            }
        }

        if let Ok(mut recovery) = self.recovery_attempts.write() {
            let count = recovery.entry(backend_key.to_string()).or_insert(0);
            *count += 1;
            tracing::debug!("Updated global recovery count for {}: {}", backend_key, *count);
        }
    }

    /// 检查后端是否在不健康列表中
    pub fn is_in_unhealthy_list(&self, backend_key: &str) -> bool {
        if let Ok(unhealthy) = self.unhealthy_backends.read() {
            unhealthy.contains_key(backend_key)
        } else {
            false
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl BackendSelector {
    pub fn new(mapping: ModelMapping, metrics: Arc<MetricsCollector>) -> Self {
        Self {
            mapping,
            round_robin_counter: AtomicUsize::new(0),
            metrics,
        }
    }

    /// 获取模型映射的引用
    pub fn get_mapping(&self) -> &ModelMapping {
        &self.mapping
    }

    /// 获取模型名称
    pub fn get_model_name(&self) -> &str {
        &self.mapping.name
    }

    pub fn select(&self) -> Result<Backend> {
        let enabled_backends: Vec<Backend> = self.mapping.backends
            .iter()
            .filter(|b| b.enabled)
            .cloned()
            .collect();

        if enabled_backends.is_empty() {
            anyhow::bail!("No enabled backends for model {}", self.mapping.name);
        }

        match self.mapping.strategy {
            LoadBalanceStrategy::WeightedRandom => {
                self.select_weighted_random(&enabled_backends)
            }
            LoadBalanceStrategy::RoundRobin => {
                self.select_round_robin(&enabled_backends)
            }
            LoadBalanceStrategy::LeastLatency => {
                self.select_least_latency(&enabled_backends)
            }
            LoadBalanceStrategy::Failover => {
                self.select_failover(&enabled_backends)
            }
            LoadBalanceStrategy::Random => {
                self.select_random(&enabled_backends)
            }
            LoadBalanceStrategy::WeightedFailover => {
                self.select_weighted_failover(&enabled_backends)
            }
        }
    }

    fn select_weighted_random(&self, backends: &[Backend]) -> Result<Backend> {
        let weights: Vec<f64> = backends.iter().map(|b| b.weight).collect();
        let dist = WeightedIndex::new(&weights)?;
        let mut rng = thread_rng();
        Ok(backends[dist.sample(&mut rng)].clone())
    }

    fn select_round_robin(&self, backends: &[Backend]) -> Result<Backend> {
        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) % backends.len();
        Ok(backends[index].clone())
    }

    fn select_least_latency(&self, backends: &[Backend]) -> Result<Backend> {
        // 根据metrics选择延迟最低的后端
        let mut best_backend = &backends[0];
        let mut best_latency = self.metrics.get_latency(&best_backend.provider, &best_backend.model)
            .unwrap_or(Duration::from_secs(999)); // 默认很高的延迟

        for backend in backends.iter().skip(1) {
            let latency = self.metrics.get_latency(&backend.provider, &backend.model)
                .unwrap_or(Duration::from_secs(999));

            if latency < best_latency {
                best_backend = backend;
                best_latency = latency;
            }
        }

        Ok(best_backend.clone())
    }

    fn select_failover(&self, backends: &[Backend]) -> Result<Backend> {
        // 按优先级排序，选择第一个可用的
        let mut sorted = backends.to_vec();
        sorted.sort_by_key(|b| b.priority);

        for backend in &sorted {
            if self.metrics.is_healthy(&backend.provider, &backend.model) {
                return Ok(backend.clone());
            }
        }

        // 如果都不健康，返回优先级最高的
        Ok(sorted[0].clone())
    }

    fn select_random(&self, backends: &[Backend]) -> Result<Backend> {
        let mut rng = thread_rng();
        let index = rng.gen_range(0..backends.len());
        Ok(backends[index].clone())
    }

    fn select_weighted_failover(&self, backends: &[Backend]) -> Result<Backend> {
        // 首先过滤出健康的后端
        let healthy_backends: Vec<Backend> = backends
            .iter()
            .filter(|b| self.metrics.is_healthy(&b.provider, &b.model))
            .cloned()
            .collect();

        // 如果有健康的后端，使用权重随机选择
        if !healthy_backends.is_empty() {
            return self.select_weighted_random(&healthy_backends);
        }

        // 如果没有健康的后端，仍然使用权重选择
        // 这样可以在所有后端都不健康时，仍然根据权重分配流量
        tracing::warn!("No healthy backends available for weighted failover, using weights on all backends");
        self.select_weighted_random(backends)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::{ModelMapping, LoadBalanceStrategy};

    fn create_test_backends() -> Vec<Backend> {
        vec![
            Backend {
                provider: "provider1".to_string(),
                model: "model1".to_string(),
                weight: 0.6,
                priority: 1,
                enabled: true,
                tags: vec![],
            },
            Backend {
                provider: "provider2".to_string(),
                model: "model2".to_string(),
                weight: 0.3,
                priority: 2,
                enabled: true,
                tags: vec![],
            },
            Backend {
                provider: "provider3".to_string(),
                model: "model3".to_string(),
                weight: 0.1,
                priority: 3,
                enabled: true,
                tags: vec![],
            },
        ]
    }

    fn create_test_mapping() -> ModelMapping {
        ModelMapping {
            name: "test-model".to_string(),
            backends: create_test_backends(),
            strategy: LoadBalanceStrategy::WeightedFailover,
            enabled: true,
        }
    }

    #[test]
    fn test_weighted_failover_all_healthy() {
        let metrics = Arc::new(MetricsCollector::new());
        let mapping = create_test_mapping();
        let selector = BackendSelector::new(mapping, metrics.clone());

        // 标记所有后端为健康
        metrics.record_success("provider1:model1");
        metrics.record_success("provider2:model2");
        metrics.record_success("provider3:model3");

        // 多次选择，验证权重分布
        let mut selections = std::collections::HashMap::new();
        for _ in 0..1000 {
            let backend = selector.select().unwrap();
            let key = format!("{}:{}", backend.provider, backend.model);
            *selections.entry(key).or_insert(0) += 1;
        }

        // 验证选择分布大致符合权重比例
        assert!(selections.contains_key("provider1:model1"));
        assert!(selections.contains_key("provider2:model2"));
        assert!(selections.contains_key("provider3:model3"));

        // provider1应该被选择最多（权重0.6）
        let provider1_count = selections.get("provider1:model1").unwrap_or(&0);
        let provider2_count = selections.get("provider2:model2").unwrap_or(&0);
        let provider3_count = selections.get("provider3:model3").unwrap_or(&0);

        assert!(provider1_count > provider2_count);
        assert!(provider2_count > provider3_count);
    }

    #[test]
    fn test_weighted_failover_partial_failure() {
        let metrics = Arc::new(MetricsCollector::new());
        let mapping = create_test_mapping();
        let selector = BackendSelector::new(mapping, metrics.clone());

        // 标记provider1为不健康，其他为健康
        metrics.record_failure("provider1:model1");
        metrics.record_success("provider2:model2");
        metrics.record_success("provider3:model3");

        // 多次选择，验证只选择健康的后端
        let mut selections = std::collections::HashMap::new();
        for _ in 0..100 {
            let backend = selector.select().unwrap();
            let key = format!("{}:{}", backend.provider, backend.model);
            *selections.entry(key).or_insert(0) += 1;
        }

        // 不应该选择不健康的provider1
        assert!(!selections.contains_key("provider1:model1"));
        // 应该选择健康的provider2和provider3
        assert!(selections.contains_key("provider2:model2"));
        assert!(selections.contains_key("provider3:model3"));
    }

    #[test]
    fn test_weighted_failover_all_failed() {
        let metrics = Arc::new(MetricsCollector::new());
        let mapping = create_test_mapping();
        let selector = BackendSelector::new(mapping, metrics.clone());

        // 标记所有后端为不健康
        metrics.record_failure("provider1:model1");
        metrics.record_failure("provider2:model2");
        metrics.record_failure("provider3:model3");

        // 应该选择优先级最高的后端（priority=1）
        let backend = selector.select().unwrap();
        assert_eq!(backend.provider, "provider1");
        assert_eq!(backend.model, "model1");
        assert_eq!(backend.priority, 1);
    }
}
