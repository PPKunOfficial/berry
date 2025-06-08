# Berry API 配置示例集合

本文档提供了各种场景下的Berry API配置示例，帮助您快速配置适合您需求的负载均衡方案。

## 📋 目录

- [基础配置](#基础配置)
- [企业级配置](#企业级配置)
- [高可用配置](#高可用配置)
- [成本优化配置](#成本优化配置)
- [开发测试配置](#开发测试配置)
- [多地域配置](#多地域配置)

## 🚀 基础配置

### 单Provider简单配置

适用于刚开始使用或简单场景：

```toml
# config_simple.toml
[settings]
health_check_interval_seconds = 30
request_timeout_seconds = 30
max_retries = 3

[users.admin]
name = "Administrator"
token = "admin-token-123456"
allowed_models = []
enabled = true

[providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-openai-key-here"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 30
max_retries = 3

[models.gpt_4]
name = "gpt-4"
strategy = "random"
enabled = true

[[models.gpt_4.backends]]
provider = "openai"
model = "gpt-4"
weight = 1.0
priority = 1
enabled = true

[models.gpt_3_5_turbo]
name = "gpt-3.5-turbo"
strategy = "random"
enabled = true

[[models.gpt_3_5_turbo.backends]]
provider = "openai"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true
```

### 双Provider负载均衡

适用于需要基本负载均衡的场景：

```toml
# config_dual_provider.toml
[settings]
health_check_interval_seconds = 30
request_timeout_seconds = 30
max_retries = 3
circuit_breaker_failure_threshold = 5
circuit_breaker_timeout_seconds = 60

[users.admin]
name = "Administrator"
token = "admin-secure-token-789"
allowed_models = []
enabled = true

[providers.openai_primary]
name = "OpenAI Primary"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-primary-key"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 30
max_retries = 3

[providers.openai_backup]
name = "OpenAI Backup"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-backup-key"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 30
max_retries = 3

[models.gpt_4]
name = "gpt-4"
strategy = "weighted_random"
enabled = true

[[models.gpt_4.backends]]
provider = "openai_primary"
model = "gpt-4"
weight = 0.7
priority = 1
enabled = true

[[models.gpt_4.backends]]
provider = "openai_backup"
model = "gpt-4"
weight = 0.3
priority = 2
enabled = true

[models.gpt_3_5_turbo]
name = "gpt-3.5-turbo"
strategy = "round_robin"
enabled = true

[[models.gpt_3_5_turbo.backends]]
provider = "openai_primary"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.gpt_3_5_turbo.backends]]
provider = "openai_backup"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 2
enabled = true
```

## 🏢 企业级配置

### 多租户权限管理

适用于需要为不同用户群体提供不同服务的企业：

```toml
# config_enterprise.toml
[settings]
health_check_interval_seconds = 30
request_timeout_seconds = 30
max_retries = 3
circuit_breaker_failure_threshold = 5
circuit_breaker_timeout_seconds = 60

# ===== 用户权限配置 =====

# 系统管理员 - 完全访问权限
[users.admin]
name = "System Administrator"
token = "admin-enterprise-token-super-secure"
allowed_models = []  # 空数组表示访问所有模型
enabled = true
tags = ["admin", "unlimited"]

# 基础用户 - 只能使用经济型模型
[users.basic_tier]
name = "Basic Tier User"
token = "basic-user-token-12345"
allowed_models = ["economy-chat", "basic-assistant"]
enabled = true
tags = ["basic", "limited"]

# 标准用户 - 可以使用中级模型
[users.standard_tier]
name = "Standard Tier User"
token = "standard-user-token-67890"
allowed_models = ["standard-chat", "gpt-3.5-turbo", "fast-response"]
enabled = true
tags = ["standard", "moderate"]

# 高级用户 - 可以使用高级模型
[users.premium_tier]
name = "Premium Tier User"
token = "premium-user-token-abcdef"
allowed_models = ["premium-chat", "gpt-4", "claude-3", "advanced-assistant"]
enabled = true
tags = ["premium", "advanced"]

# 企业用户 - 可以使用所有模型
[users.enterprise_tier]
name = "Enterprise Tier User"
token = "enterprise-user-token-xyz789"
allowed_models = []  # 访问所有模型
enabled = true
tags = ["enterprise", "unlimited"]

# ===== Provider配置 =====

[providers.openai_primary]
name = "OpenAI Primary Account"
base_url = "https://api.openai.com/v1"
api_key = "sk-openai-primary-key"
models = ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 30
max_retries = 3

[providers.openai_secondary]
name = "OpenAI Secondary Account"
base_url = "https://api.openai.com/v1"
api_key = "sk-openai-secondary-key"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 30
max_retries = 3

[providers.azure_openai]
name = "Azure OpenAI Enterprise"
base_url = "https://your-enterprise.openai.azure.com"
api_key = "azure-enterprise-key"
models = ["gpt-4", "gpt-35-turbo"]
enabled = true
timeout_seconds = 45
max_retries = 2
[providers.azure_openai.headers]
"api-version" = "2024-02-01"

[providers.anthropic]
name = "Anthropic Claude"
base_url = "https://api.anthropic.com"
api_key = "sk-ant-anthropic-key"
models = ["claude-3-opus-20240229", "claude-3-sonnet-20240229"]
enabled = true
timeout_seconds = 60
max_retries = 2

[providers.budget_proxy]
name = "Budget Proxy Service"
base_url = "https://budget-proxy.example.com/v1"
api_key = "budget-proxy-key"
models = ["gpt-3.5-turbo", "gpt-4"]
enabled = true
timeout_seconds = 20
max_retries = 3

# ===== 模型映射配置 =====

# 经济型聊天 - 基础用户使用
[models.economy_chat]
name = "economy-chat"
strategy = "weighted_random"
enabled = true

[[models.economy_chat.backends]]
provider = "budget_proxy"
model = "gpt-3.5-turbo"
weight = 0.8
priority = 1
enabled = true

[[models.economy_chat.backends]]
provider = "openai_secondary"
model = "gpt-3.5-turbo"
weight = 0.2
priority = 2
enabled = true

# 标准聊天 - 标准用户使用
[models.standard_chat]
name = "standard-chat"
strategy = "round_robin"
enabled = true

[[models.standard_chat.backends]]
provider = "openai_primary"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.standard_chat.backends]]
provider = "openai_secondary"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 2
enabled = true

# 高级聊天 - 高级用户使用
[models.premium_chat]
name = "premium-chat"
strategy = "least_latency"
enabled = true

[[models.premium_chat.backends]]
provider = "openai_primary"
model = "gpt-4"
weight = 1.0
priority = 1
enabled = true

[[models.premium_chat.backends]]
provider = "azure_openai"
model = "gpt-4"
weight = 1.0
priority = 2
enabled = true

# Claude-3 高级模型
[models.claude_3]
name = "claude-3"
strategy = "failover"
enabled = true

[[models.claude_3.backends]]
provider = "anthropic"
model = "claude-3-opus-20240229"
weight = 1.0
priority = 1
enabled = true

[[models.claude_3.backends]]
provider = "anthropic"
model = "claude-3-sonnet-20240229"
weight = 1.0
priority = 2
enabled = true

# 基础助手
[models.basic_assistant]
name = "basic-assistant"
strategy = "random"
enabled = true

[[models.basic_assistant.backends]]
provider = "budget_proxy"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

# 高级助手
[models.advanced_assistant]
name = "advanced-assistant"
strategy = "weighted_failover"
enabled = true

[[models.advanced_assistant.backends]]
provider = "openai_primary"
model = "gpt-4-turbo"
weight = 0.6
priority = 1
enabled = true

[[models.advanced_assistant.backends]]
provider = "anthropic"
model = "claude-3-opus-20240229"
weight = 0.4
priority = 2
enabled = true

# 快速响应模型
[models.fast_response]
name = "fast-response"
strategy = "least_latency"
enabled = true

[[models.fast_response.backends]]
provider = "openai_primary"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.fast_response.backends]]
provider = "budget_proxy"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 2
enabled = true

# 标准GPT-4访问
[models.gpt_4]
name = "gpt-4"
strategy = "weighted_random"
enabled = true

[[models.gpt_4.backends]]
provider = "openai_primary"
model = "gpt-4"
weight = 0.5
priority = 1
enabled = true

[[models.gpt_4.backends]]
provider = "azure_openai"
model = "gpt-4"
weight = 0.3
priority = 2
enabled = true

[[models.gpt_4.backends]]
provider = "openai_secondary"
model = "gpt-4"
weight = 0.2
priority = 3
enabled = true

# 标准GPT-3.5访问
[models.gpt_3_5_turbo]
name = "gpt-3.5-turbo"
strategy = "round_robin"
enabled = true

[[models.gpt_3_5_turbo.backends]]
provider = "openai_primary"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.gpt_3_5_turbo.backends]]
provider = "openai_secondary"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 2
enabled = true
```

## 🏥 高可用配置

### 故障转移优先配置

适用于对可用性要求极高的生产环境：

```toml
# config_high_availability.toml
[settings]
health_check_interval_seconds = 15  # 更频繁的健康检查
request_timeout_seconds = 30
max_retries = 5  # 更多重试次数
circuit_breaker_failure_threshold = 3  # 更快的熔断
circuit_breaker_timeout_seconds = 30   # 更快的恢复

[users.admin]
name = "HA Administrator"
token = "ha-admin-token-ultra-secure"
allowed_models = []
enabled = true

# 主Provider - 最高优先级
[providers.primary_openai]
name = "Primary OpenAI"
base_url = "https://api.openai.com/v1"
api_key = "sk-primary-key"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 25
max_retries = 3

# 备用Provider 1 - Azure
[providers.backup_azure]
name = "Backup Azure OpenAI"
base_url = "https://backup.openai.azure.com"
api_key = "azure-backup-key"
models = ["gpt-4", "gpt-35-turbo"]
enabled = true
timeout_seconds = 30
max_retries = 3
[providers.backup_azure.headers]
"api-version" = "2024-02-01"

# 备用Provider 2 - 第二个OpenAI账户
[providers.backup_openai]
name = "Backup OpenAI"
base_url = "https://api.openai.com/v1"
api_key = "sk-backup-key"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 25
max_retries = 3

# 应急Provider - 代理服务
[providers.emergency_proxy]
name = "Emergency Proxy"
base_url = "https://emergency-proxy.example.com/v1"
api_key = "emergency-proxy-key"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 20
max_retries = 2

# 高可用GPT-4配置
[models.gpt_4_ha]
name = "gpt-4"
strategy = "failover"
enabled = true

[[models.gpt_4_ha.backends]]
provider = "primary_openai"
model = "gpt-4"
weight = 1.0
priority = 1  # 最高优先级
enabled = true

[[models.gpt_4_ha.backends]]
provider = "backup_azure"
model = "gpt-4"
weight = 1.0
priority = 2  # 第二优先级
enabled = true

[[models.gpt_4_ha.backends]]
provider = "backup_openai"
model = "gpt-4"
weight = 1.0
priority = 3  # 第三优先级
enabled = true

[[models.gpt_4_ha.backends]]
provider = "emergency_proxy"
model = "gpt-4"
weight = 1.0
priority = 4  # 应急使用
enabled = true

# 高可用GPT-3.5配置
[models.gpt_3_5_turbo_ha]
name = "gpt-3.5-turbo"
strategy = "failover"
enabled = true

[[models.gpt_3_5_turbo_ha.backends]]
provider = "primary_openai"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.gpt_3_5_turbo_ha.backends]]
provider = "backup_azure"
model = "gpt-35-turbo"
weight = 1.0
priority = 2
enabled = true

[[models.gpt_3_5_turbo_ha.backends]]
provider = "backup_openai"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 3
enabled = true

[[models.gpt_3_5_turbo_ha.backends]]
provider = "emergency_proxy"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 4
enabled = true
```

## 💰 成本优化配置

### 成本敏感型配置

适用于需要控制成本但保证基本可用性的场景：

```toml
# config_cost_optimized.toml
[settings]
health_check_interval_seconds = 60  # 降低检查频率节省资源
request_timeout_seconds = 45        # 稍长的超时时间
max_retries = 2                     # 减少重试次数
circuit_breaker_failure_threshold = 10  # 更宽松的熔断条件
circuit_breaker_timeout_seconds = 120   # 更长的恢复时间

[users.cost_user]
name = "Cost Conscious User"
token = "cost-user-token-123"
allowed_models = ["economy", "budget-chat", "cheap-assistant"]
enabled = true

# 便宜的代理服务 - 主要使用
[providers.cheap_proxy]
name = "Cheap Proxy Service"
base_url = "https://cheap-proxy.example.com/v1"
api_key = "cheap-proxy-key"
models = ["gpt-3.5-turbo", "gpt-4"]
enabled = true
timeout_seconds = 30
max_retries = 2

# 中等价格的代理 - 备用
[providers.medium_proxy]
name = "Medium Price Proxy"
base_url = "https://medium-proxy.example.com/v1"
api_key = "medium-proxy-key"
models = ["gpt-3.5-turbo", "gpt-4"]
enabled = true
timeout_seconds = 25
max_retries = 2

# 官方服务 - 应急使用
[providers.official_backup]
name = "Official Backup"
base_url = "https://api.openai.com/v1"
api_key = "sk-official-backup-key"
models = ["gpt-3.5-turbo", "gpt-4"]
enabled = true
timeout_seconds = 30
max_retries = 1

# 经济型模型 - 主要推荐
[models.economy]
name = "economy"
strategy = "weighted_random"
enabled = true

[[models.economy.backends]]
provider = "cheap_proxy"
model = "gpt-3.5-turbo"
weight = 0.8  # 80%使用便宜服务
priority = 1
enabled = true

[[models.economy.backends]]
provider = "medium_proxy"
model = "gpt-3.5-turbo"
weight = 0.15  # 15%使用中等价格
priority = 2
enabled = true

[[models.economy.backends]]
provider = "official_backup"
model = "gpt-3.5-turbo"
weight = 0.05  # 5%使用官方服务
priority = 3
enabled = true

# 预算聊天模型
[models.budget_chat]
name = "budget-chat"
strategy = "weighted_failover"
enabled = true

[[models.budget_chat.backends]]
provider = "cheap_proxy"
model = "gpt-3.5-turbo"
weight = 0.9
priority = 1
enabled = true

[[models.budget_chat.backends]]
provider = "medium_proxy"
model = "gpt-3.5-turbo"
weight = 0.1
priority = 2
enabled = true

# 便宜的助手
[models.cheap_assistant]
name = "cheap-assistant"
strategy = "failover"
enabled = true

[[models.cheap_assistant.backends]]
provider = "cheap_proxy"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.cheap_assistant.backends]]
provider = "medium_proxy"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 2
enabled = true

[[models.cheap_assistant.backends]]
provider = "official_backup"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 3
enabled = true
```

## 🧪 开发测试配置

### 开发环境配置

适用于开发和测试环境：

```toml
# config_development.toml
[settings]
health_check_interval_seconds = 60
request_timeout_seconds = 60  # 开发时允许更长超时
max_retries = 1               # 开发时快速失败
circuit_breaker_failure_threshold = 20  # 宽松的熔断条件
circuit_breaker_timeout_seconds = 30

# 开发者用户
[users.developer]
name = "Developer"
token = "dev-token-123"
allowed_models = []  # 开发者可以访问所有模型
enabled = true
tags = ["developer"]

# 测试用户
[users.tester]
name = "Tester"
token = "test-token-456"
allowed_models = ["test-model", "debug-chat"]
enabled = true
tags = ["tester"]

# 临时禁用的用户（用于测试）
[users.disabled_user]
name = "Disabled Test User"
token = "disabled-token-789"
allowed_models = ["test-model"]
enabled = false
tags = ["disabled", "test"]

# 开发用的OpenAI账户
[providers.dev_openai]
name = "Development OpenAI"
base_url = "https://api.openai.com/v1"
api_key = "sk-dev-key"
models = ["gpt-3.5-turbo", "gpt-4"]
enabled = true
timeout_seconds = 60
max_retries = 1

# 测试用的模拟服务
[providers.mock_service]
name = "Mock Service"
base_url = "http://localhost:8080/v1"  # 本地模拟服务
api_key = "mock-key"
models = ["mock-gpt-3.5", "mock-gpt-4"]
enabled = true
timeout_seconds = 10
max_retries = 1

# 测试模型
[models.test_model]
name = "test-model"
strategy = "random"
enabled = true

[[models.test_model.backends]]
provider = "mock_service"
model = "mock-gpt-3.5"
weight = 1.0
priority = 1
enabled = true

# 调试聊天模型
[models.debug_chat]
name = "debug-chat"
strategy = "failover"
enabled = true

[[models.debug_chat.backends]]
provider = "mock_service"
model = "mock-gpt-3.5"
weight = 1.0
priority = 1
enabled = true

[[models.debug_chat.backends]]
provider = "dev_openai"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 2
enabled = true

# 开发用GPT-3.5
[models.gpt_3_5_turbo]
name = "gpt-3.5-turbo"
strategy = "round_robin"
enabled = true

[[models.gpt_3_5_turbo.backends]]
provider = "dev_openai"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.gpt_3_5_turbo.backends]]
provider = "mock_service"
model = "mock-gpt-3.5"
weight = 1.0
priority = 2
enabled = true

# 开发用GPT-4
[models.gpt_4]
name = "gpt-4"
strategy = "failover"
enabled = true

[[models.gpt_4.backends]]
provider = "dev_openai"
model = "gpt-4"
weight = 1.0
priority = 1
enabled = true

[[models.gpt_4.backends]]
provider = "mock_service"
model = "mock-gpt-4"
weight = 1.0
priority = 2
enabled = true
```

## 🌍 多地域配置

### 全球分布式配置

适用于需要为全球用户提供低延迟服务的场景：

```toml
# config_global.toml
[settings]
health_check_interval_seconds = 30
request_timeout_seconds = 30
max_retries = 3
circuit_breaker_failure_threshold = 5
circuit_breaker_timeout_seconds = 60

[users.global_admin]
name = "Global Administrator"
token = "global-admin-token-secure"
allowed_models = []
enabled = true

# 美国东部Provider
[providers.us_east]
name = "US East OpenAI"
base_url = "https://api.openai.com/v1"
api_key = "sk-us-east-key"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 25
max_retries = 3

# 欧洲Provider
[providers.eu_west]
name = "EU West Azure"
base_url = "https://eu-west.openai.azure.com"
api_key = "eu-west-azure-key"
models = ["gpt-4", "gpt-35-turbo"]
enabled = true
timeout_seconds = 30
max_retries = 3
[providers.eu_west.headers]
"api-version" = "2024-02-01"

# 亚太Provider
[providers.apac]
name = "APAC Proxy Service"
base_url = "https://apac-proxy.example.com/v1"
api_key = "apac-proxy-key"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 35
max_retries = 3

# 全球GPT-4服务 - 使用最低延迟策略
[models.global_gpt_4]
name = "gpt-4"
strategy = "least_latency"
enabled = true

[[models.global_gpt_4.backends]]
provider = "us_east"
model = "gpt-4"
weight = 1.0
priority = 1
enabled = true
tags = ["us", "americas"]

[[models.global_gpt_4.backends]]
provider = "eu_west"
model = "gpt-4"
weight = 1.0
priority = 2
enabled = true
tags = ["eu", "europe"]

[[models.global_gpt_4.backends]]
provider = "apac"
model = "gpt-4"
weight = 1.0
priority = 3
enabled = true
tags = ["apac", "asia"]

# 全球GPT-3.5服务
[models.global_gpt_3_5]
name = "gpt-3.5-turbo"
strategy = "least_latency"
enabled = true

[[models.global_gpt_3_5.backends]]
provider = "us_east"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.global_gpt_3_5.backends]]
provider = "eu_west"
model = "gpt-35-turbo"
weight = 1.0
priority = 2
enabled = true

[[models.global_gpt_3_5.backends]]
provider = "apac"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 3
enabled = true

# 地域优化的快速响应模型
[models.fast_global]
name = "fast-global"
strategy = "least_latency"
enabled = true

[[models.fast_global.backends]]
provider = "us_east"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.fast_global.backends]]
provider = "eu_west"
model = "gpt-35-turbo"
weight = 1.0
priority = 2
enabled = true

[[models.fast_global.backends]]
provider = "apac"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 3
enabled = true
```

---

这些配置示例涵盖了各种常见的使用场景。您可以根据自己的需求选择合适的配置作为起点，然后根据实际情况进行调整。

记住在生产环境中：
1. 使用强随机Token
2. 定期轮换API密钥
3. 监控服务健康状态
4. 根据实际性能调整权重和超时设置
