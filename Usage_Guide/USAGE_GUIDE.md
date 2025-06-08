# Berry API 详细使用指南

本文档提供Berry API的详细使用指南，包括高级配置、最佳实践和实际应用场景。

## 📋 目录

- [快速开始](#快速开始)
- [配置详解](#配置详解)
- [使用场景](#使用场景)
- [最佳实践](#最佳实践)
- [故障排除](#故障排除)
- [性能调优](#性能调优)

## 🚀 快速开始

### 最小配置示例

创建一个最简单的配置文件 `minimal_config.toml`：

```toml
[settings]
health_check_interval_seconds = 30

[users.admin]
name = "Admin"
token = "admin-token-123"
allowed_models = []
enabled = true

[providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-key-here"
models = ["gpt-3.5-turbo"]
enabled = true

[models.chat]
name = "chat"
strategy = "random"
enabled = true

[[models.chat.backends]]
provider = "openai"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true
```

启动服务：
```bash
CONFIG_PATH="minimal_config.toml" cargo run
```

测试请求：
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer admin-token-123" \
  -d '{
    "model": "chat",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## ⚙️ 配置详解

### 用户权限管理

#### 1. 管理员用户
```toml
[users.admin]
name = "System Administrator"
token = "admin-super-secret-token"
allowed_models = []  # 空数组 = 访问所有模型
enabled = true
tags = ["admin", "unlimited"]
```

#### 2. 受限用户
```toml
[users.basic_user]
name = "Basic User"
token = "user-basic-token-456"
allowed_models = ["gpt-3.5-turbo", "fast-chat"]  # 只能访问指定模型
enabled = true
tags = ["basic", "limited"]

[users.premium_user]
name = "Premium User"
token = "user-premium-token-789"
allowed_models = ["gpt-4", "gpt-4-turbo", "claude-3"]
enabled = true
tags = ["premium", "advanced"]
```

#### 3. 临时禁用用户
```toml
[users.suspended_user]
name = "Suspended User"
token = "user-suspended-token"
allowed_models = ["gpt-3.5-turbo"]
enabled = false  # 禁用用户
tags = ["suspended"]
```

### Provider高级配置

#### 1. OpenAI配置
```toml
[providers.openai_primary]
name = "OpenAI Primary"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-primary-key"
models = ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo", "gpt-4o"]
enabled = true
timeout_seconds = 30
max_retries = 3
```

#### 2. Azure OpenAI配置
```toml
[providers.azure_openai]
name = "Azure OpenAI"
base_url = "https://your-resource.openai.azure.com"
api_key = "your-azure-key"
models = ["gpt-4", "gpt-35-turbo"]
enabled = true
timeout_seconds = 45
max_retries = 2

[providers.azure_openai.headers]
"api-version" = "2024-02-01"
"Content-Type" = "application/json"
```

#### 3. Anthropic配置
```toml
[providers.anthropic]
name = "Anthropic Claude"
base_url = "https://api.anthropic.com"
api_key = "sk-ant-your-key"
models = ["claude-3-opus-20240229", "claude-3-sonnet-20240229", "claude-3-haiku-20240307"]
enabled = true
timeout_seconds = 60
max_retries = 2
```

#### 4. 自定义代理服务
```toml
[providers.custom_proxy]
name = "Custom Proxy Service"
base_url = "https://your-proxy.example.com/v1"
api_key = "proxy-api-key"
models = ["gpt-4", "gpt-3.5-turbo", "claude-3"]
enabled = true
timeout_seconds = 20
max_retries = 3

[providers.custom_proxy.headers]
"X-Custom-Header" = "custom-value"
"User-Agent" = "Berry-API/1.0"
```

### 模型映射高级配置

#### 1. 多Provider负载均衡
```toml
[models.gpt_4_balanced]
name = "gpt-4-balanced"
strategy = "weighted_random"
enabled = true

# 主要Provider - 70%流量
[[models.gpt_4_balanced.backends]]
provider = "openai_primary"
model = "gpt-4"
weight = 0.7
priority = 1
enabled = true
tags = ["primary", "stable"]

# Azure备用 - 20%流量
[[models.gpt_4_balanced.backends]]
provider = "azure_openai"
model = "gpt-4"
weight = 0.2
priority = 2
enabled = true
tags = ["azure", "backup"]

# 代理服务 - 10%流量
[[models.gpt_4_balanced.backends]]
provider = "custom_proxy"
model = "gpt-4"
weight = 0.1
priority = 3
enabled = true
tags = ["proxy", "fallback"]
```

#### 2. 高可用故障转移
```toml
[models.gpt_4_ha]
name = "gpt-4-ha"
strategy = "failover"
enabled = true

# 主服务
[[models.gpt_4_ha.backends]]
provider = "openai_primary"
model = "gpt-4"
weight = 1.0
priority = 1  # 最高优先级
enabled = true

# 第一备用
[[models.gpt_4_ha.backends]]
provider = "azure_openai"
model = "gpt-4"
weight = 1.0
priority = 2
enabled = true

# 第二备用
[[models.gpt_4_ha.backends]]
provider = "custom_proxy"
model = "gpt-4"
weight = 1.0
priority = 3
enabled = true
```

#### 3. 性能优化配置
```toml
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
provider = "custom_proxy"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 2
enabled = true
```

## 🎯 使用场景

### 场景1：企业级多租户部署

适用于需要为不同用户群体提供不同服务级别的企业：

```toml
# 基础用户 - 只能使用经济型模型
[users.basic]
name = "Basic Tier User"
token = "basic-tier-token"
allowed_models = ["economy-chat", "basic-assistant"]
enabled = true

# 高级用户 - 可以使用高级模型
[users.premium]
name = "Premium Tier User"
token = "premium-tier-token"
allowed_models = ["premium-chat", "advanced-assistant", "code-helper"]
enabled = true

# 经济型聊天模型
[models.economy_chat]
name = "economy-chat"
strategy = "weighted_random"
enabled = true

[[models.economy_chat.backends]]
provider = "cheap_provider"
model = "gpt-3.5-turbo"
weight = 0.8
priority = 1
enabled = true

[[models.economy_chat.backends]]
provider = "backup_provider"
model = "gpt-3.5-turbo"
weight = 0.2
priority = 2
enabled = true

# 高级聊天模型
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
provider = "anthropic"
model = "claude-3-opus-20240229"
weight = 1.0
priority = 2
enabled = true
```

### 场景2：成本优化部署

适用于需要控制成本但保证服务可用性的场景：

```toml
[models.cost_optimized]
name = "cost-optimized"
strategy = "weighted_failover"
enabled = true

# 便宜的代理服务 - 80%流量
[[models.cost_optimized.backends]]
provider = "cheap_proxy"
model = "gpt-3.5-turbo"
weight = 0.8
priority = 1
enabled = true

# 官方服务作为备用 - 20%流量
[[models.cost_optimized.backends]]
provider = "openai_primary"
model = "gpt-3.5-turbo"
weight = 0.2
priority = 2
enabled = true
```

### 场景3：地理分布式部署

适用于全球用户的低延迟服务：

```toml
[models.global_service]
name = "global-service"
strategy = "least_latency"
enabled = true

# 美国东部
[[models.global_service.backends]]
provider = "us_east_provider"
model = "gpt-4"
weight = 1.0
priority = 1
enabled = true

# 欧洲
[[models.global_service.backends]]
provider = "eu_provider"
model = "gpt-4"
weight = 1.0
priority = 2
enabled = true

# 亚太
[[models.global_service.backends]]
provider = "apac_provider"
model = "gpt-4"
weight = 1.0
priority = 3
enabled = true
```

## 🏆 最佳实践

### 1. 安全配置

#### Token管理
- 使用强随机Token（至少32字符）
- 定期轮换Token
- 为不同环境使用不同Token
- 避免在日志中记录Token

```bash
# 生成安全Token的示例
openssl rand -hex 32
```

#### API密钥保护
- 使用环境变量或安全的配置管理
- 定期轮换API密钥
- 监控API密钥使用情况
- 设置适当的权限和限制

#### 网络安全
```toml
# 推荐的安全配置
[settings]
request_timeout_seconds = 30  # 避免长时间连接
max_retries = 3              # 限制重试次数
circuit_breaker_failure_threshold = 5  # 快速熔断
```

### 2. 性能优化

#### 连接池配置
```toml
[settings]
# 根据并发需求调整
health_check_interval_seconds = 30
request_timeout_seconds = 30
max_retries = 3
```

#### 负载均衡策略选择
- **高并发场景**: 使用 `round_robin` 或 `least_latency`
- **成本敏感**: 使用 `weighted_random`
- **高可用要求**: 使用 `failover` 或 `weighted_failover`
- **简单场景**: 使用 `random`

#### 权重调优
```toml
# 根据Provider性能调整权重
[[models.optimized.backends]]
provider = "fast_provider"
model = "gpt-4"
weight = 0.6  # 高性能Provider更高权重
priority = 1
enabled = true

[[models.optimized.backends]]
provider = "slow_provider"
model = "gpt-4"
weight = 0.4  # 低性能Provider较低权重
priority = 2
enabled = true
```

### 3. 监控和告警

#### 关键指标监控
- Provider健康状态
- 请求成功率
- 平均响应时间
- 错误率分布

#### 日志配置
```bash
# 生产环境推荐日志级别
RUST_LOG=info cargo run

# 调试时使用
RUST_LOG=debug cargo run

# 特定模块调试
RUST_LOG=berry_api_api::loadbalance=debug cargo run
```

#### 健康检查监控
```bash
# 监控健康检查状态
curl http://localhost:3000/health | jq '.providers'

# 获取详细指标
curl http://localhost:3000/metrics | jq '.providers'
```

### 4. 容错设计

#### 多Provider配置
```toml
# 为每个模型至少配置2个Provider
[models.reliable_service]
name = "reliable-service"
strategy = "weighted_failover"
enabled = true

[[models.reliable_service.backends]]
provider = "primary_provider"
model = "gpt-4"
weight = 0.7
priority = 1
enabled = true

[[models.reliable_service.backends]]
provider = "backup_provider"
model = "gpt-4"
weight = 0.3
priority = 2
enabled = true
```

#### 熔断配置
```toml
[settings]
# 快速检测故障
circuit_breaker_failure_threshold = 3
# 适当的恢复时间
circuit_breaker_timeout_seconds = 60
```

## 🔧 故障排除

### 常见问题诊断

#### 1. 服务无法启动
```bash
# 检查配置文件语法
cargo run -- --check-config

# 查看详细错误信息
RUST_LOG=debug cargo run 2>&1 | grep ERROR

# 检查端口占用
netstat -tlnp | grep 3000
```

#### 2. Provider连接失败
```bash
# 测试网络连接
curl -v https://api.openai.com/v1/models

# 验证API密钥
curl -H "Authorization: Bearer sk-your-key" https://api.openai.com/v1/models

# 检查DNS解析
nslookup api.openai.com
```

#### 3. 认证问题
```bash
# 测试Token认证
curl -H "Authorization: Bearer your-token" http://localhost:3000/v1/models

# 检查用户配置
grep -A 5 "users.your_user" config.toml
```

#### 4. 负载均衡异常
```bash
# 查看负载均衡决策日志
grep "selected backend" logs/berry-api.log

# 检查Provider健康状态
curl http://localhost:3000/health | jq '.providers'

# 验证权重配置
grep -A 10 "models.your_model" config.toml
```

### 调试技巧

#### 1. 启用详细日志
```bash
# 全局调试
RUST_LOG=debug cargo run

# 模块级调试
RUST_LOG=berry_api_api::loadbalance=debug,berry_api_api::auth=debug cargo run

# 追踪级日志（最详细）
RUST_LOG=trace cargo run
```

#### 2. 配置验证
```bash
# 验证TOML语法
toml-cli check config.toml

# 验证配置逻辑
cargo run -- --validate-config
```

#### 3. 网络诊断
```bash
# 测试Provider连接
curl -w "@curl-format.txt" -o /dev/null -s "https://api.openai.com/v1/models"

# 监控网络延迟
ping -c 10 api.openai.com
```

## ⚡ 性能调优

### 1. 系统级优化

#### 操作系统配置
```bash
# 增加文件描述符限制
ulimit -n 65536

# 优化TCP参数
echo 'net.core.somaxconn = 65536' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 65536' >> /etc/sysctl.conf
sysctl -p
```

#### 内存配置
```bash
# 设置合适的堆大小
export RUST_MIN_STACK=8388608

# 启用内存优化
export MALLOC_ARENA_MAX=2
```

### 2. 应用级优化

#### 连接复用
```toml
[settings]
# 保持连接活跃
request_timeout_seconds = 30
# 合理的重试策略
max_retries = 3
# 适当的健康检查频率
health_check_interval_seconds = 30
```

#### 并发控制
```bash
# 使用多线程运行时
export TOKIO_WORKER_THREADS=8

# 启用工作窃取
export TOKIO_ENABLE_WORK_STEALING=1
```

### 3. 监控和分析

#### 性能指标收集
```bash
# 使用系统监控工具
htop
iotop
nethogs

# 应用性能分析
cargo flamegraph --bin berry-api
```

#### 压力测试
```bash
# 使用wrk进行压力测试
wrk -t12 -c400 -d30s --script=load-test.lua http://localhost:3000/v1/chat/completions

# 使用ab进行简单测试
ab -n 1000 -c 10 http://localhost:3000/health
```

---

这份详细指南涵盖了Berry API的高级使用方法，帮助您充分发挥系统的潜力。如有问题，请参考主README文档或提交Issue。
