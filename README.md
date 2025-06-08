# Berry API - 负载均衡AI网关

Berry API 是一个高性能的AI服务负载均衡网关，支持多种AI服务提供商的智能负载均衡、故障转移和健康检查。

## 📚 文档导航

- **[📋 文档索引](DOCUMENTATION_INDEX.md)** - 所有文档的导航页面
- **[📖 详细使用指南](USAGE_GUIDE.md)** - 高级配置和最佳实践
- **[🔌 API接口参考](API_REFERENCE.md)** - 完整的API文档
- **[⚙️ 配置示例集合](CONFIGURATION_EXAMPLES.md)** - 各种场景的配置示例

## 🚀 特性

### 核心功能
- **多Provider支持**: 支持OpenAI、Azure OpenAI、Anthropic等多种AI服务提供商
- **智能负载均衡**: 支持加权随机、轮询、最低延迟、故障转移等多种负载均衡策略
- **健康检查**: 自动监控后端服务健康状态，实现故障自动切换
- **用户认证**: 基于Token的用户认证和权限管理
- **配置热重载**: 支持运行时配置更新，无需重启服务
- **OpenAI兼容**: 完全兼容OpenAI API格式，无缝替换
- **流式支持**: 完整支持流式和非流式响应

### 负载均衡策略
- **加权随机 (weighted_random)**: 根据权重随机选择后端
- **轮询 (round_robin)**: 依次轮询所有可用后端
- **最低延迟 (least_latency)**: 选择响应时间最短的后端
- **故障转移 (failover)**: 按优先级顺序选择，主要用于备份场景
- **随机 (random)**: 完全随机选择后端
- **权重故障转移 (weighted_failover)**: 🆕 结合权重选择和故障转移，优先从健康的后端中按权重选择，故障时自动切换

### 监控与指标
- **实时健康状态**: 提供详细的服务健康状态信息
- **性能指标**: 记录请求延迟、成功率等关键指标
- **服务发现**: 自动发现和管理可用的模型服务
- **熔断机制**: 自动熔断故障服务，防止级联失败

## 📋 系统架构

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   客户端请求     │───▶│  Berry API网关   │───▶│   AI服务提供商   │
│                │    │                  │    │                │
│ - OpenAI格式    │    │ - 用户认证        │    │ - OpenAI        │
│ - 流式/非流式   │    │ - 负载均衡        │    │ - Azure OpenAI  │
│ - 模型选择      │    │ - 健康检查        │    │ - Anthropic     │
│ - Token认证     │    │ - 故障转移        │    │ - 其他代理服务   │
└─────────────────┘    │ - 指标收集        │    └─────────────────┘
                       │ - 熔断保护        │
                       └──────────────────┘
```

### 核心组件

- **配置管理**: 支持TOML配置文件，包含Provider、模型映射、用户管理等
- **负载均衡器**: 多种策略的智能负载均衡，支持权重、优先级、健康状态
- **健康检查器**: 定期检查后端服务健康状态，支持自动故障转移和恢复
- **认证中间件**: 基于Token的用户认证，支持模型访问权限控制
- **请求转发器**: 高性能的HTTP请求转发，支持流式响应
- **指标收集器**: 实时收集性能指标，支持监控和告警

## 🛠️ 快速开始

### 1. 环境要求
- **Rust**: 1.70+ (推荐使用最新稳定版)
- **操作系统**: Linux, macOS, Windows
- **内存**: 最少512MB，推荐1GB+
- **网络**: 需要访问AI服务提供商的API

### 2. 安装
```bash
# 克隆项目
git clone https://github.com/PPKunOfficial/berry-api.git
cd berry-api

# 编译项目
cargo build --release
```

### 3. 配置文件设置
复制示例配置文件并根据需要修改：
```bash
cp config_example.toml config.toml
```

### 4. 基础配置
编辑 `config.toml` 文件，配置你的AI服务提供商：

```toml
# 全局设置
[settings]
health_check_interval_seconds = 30
request_timeout_seconds = 30
max_retries = 3

# 用户认证配置
[users.admin]
name = "Administrator"
token = "your-admin-token-here"
allowed_models = []  # 空数组表示允许访问所有模型
enabled = true

# Provider配置
[providers.openai-primary]
name = "OpenAI Primary Account"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-openai-key-here"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true

# 模型映射配置
[models.gpt_4]
name = "gpt-4"
strategy = "weighted_random"
enabled = true

[[models.gpt_4.backends]]
provider = "openai-primary"
model = "gpt-4"
weight = 1.0
priority = 1
enabled = true
```

### 5. 启动服务
```bash
# 开发模式
cargo run

# 生产模式
./target/release/berry-api

# 指定配置文件
CONFIG_PATH="config.toml" cargo run

# 启用调试日志
RUST_LOG=debug cargo run
```

服务默认在 `http://localhost:3000` 启动。

## 📝 详细配置指南

### 1. 全局设置 (settings)
```toml
[settings]
health_check_interval_seconds = 30    # 健康检查间隔（秒）
request_timeout_seconds = 30          # 请求超时时间（秒）
max_retries = 3                       # 最大重试次数
circuit_breaker_failure_threshold = 5 # 熔断器失败阈值
circuit_breaker_timeout_seconds = 60  # 熔断器超时时间（秒）
```

### 2. 用户认证配置 (users)
```toml
# 管理员用户 - 可以访问所有模型
[users.admin]
name = "Administrator"
token = "berry-admin-token-12345"
allowed_models = []  # 空数组表示允许访问所有模型
enabled = true
tags = ["admin", "unlimited"]

# 普通用户 - 只能访问指定模型
[users.user1]
name = "Regular User 1"
token = "berry-user1-token-67890"
allowed_models = ["gpt-3.5-turbo", "fast-chat"]
enabled = true
tags = ["user", "basic"]

# 高级用户 - 可以访问高级模型
[users.premium]
name = "Premium User"
token = "berry-premium-token-abcde"
allowed_models = ["gpt-4", "gpt-4-turbo", "premium", "claude_3"]
enabled = true
tags = ["premium", "advanced"]
```

### 3. Provider配置 (providers)
```toml
# OpenAI 配置
[providers.openai-primary]
name = "OpenAI Primary Account"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-openai-key-here"
models = ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]
enabled = true
timeout_seconds = 30
max_retries = 3

# Azure OpenAI 配置
[providers.azure-openai]
name = "Azure OpenAI Service"
base_url = "https://your-resource.openai.azure.com"
api_key = "your-azure-openai-key-here"
models = ["gpt-4", "gpt-35-turbo"]
enabled = true
timeout_seconds = 30
max_retries = 3
[providers.azure-openai.headers]
"api-version" = "2024-02-01"

# Anthropic Claude 配置
[providers.anthropic]
name = "Anthropic Claude"
base_url = "https://api.anthropic.com"
api_key = "sk-ant-your-anthropic-key-here"
models = ["claude-3-opus-20240229", "claude-3-sonnet-20240229"]
enabled = true
timeout_seconds = 30
max_retries = 3
```

### 4. 模型映射配置 (models)
```toml
# GPT-4 模型 - 使用加权随机负载均衡
[models.gpt_4]
name = "gpt-4"  # 对外暴露的模型名
strategy = "weighted_random"
enabled = true

# 后端配置：多个provider的gpt-4模型
[[models.gpt_4.backends]]
provider = "openai-primary"
model = "gpt-4"
weight = 0.5      # 50% 权重
priority = 1      # 最高优先级
enabled = true
tags = ["premium", "stable"]

[[models.gpt_4.backends]]
provider = "azure-openai"
model = "gpt-4"
weight = 0.3      # 30% 权重
priority = 2
enabled = true
tags = ["enterprise"]

[[models.gpt_4.backends]]
provider = "anthropic"
model = "claude-3-opus-20240229"
weight = 0.2      # 20% 权重
priority = 3
enabled = true
tags = ["alternative"]
```

## 🔌 API使用指南

### 1. 认证方式
所有API请求都需要在Header中包含认证Token：
```bash
Authorization: Bearer your-token-here
```

### 2. 聊天完成 (兼容OpenAI)

#### 非流式请求
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer berry-admin-token-12345" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {"role": "user", "content": "Hello, world!"}
    ],
    "stream": false,
    "max_tokens": 1000,
    "temperature": 0.7
  }'
```

#### 流式请求
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer berry-admin-token-12345" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {"role": "user", "content": "写一首关于春天的诗"}
    ],
    "stream": true,
    "max_tokens": 1000
  }'
```

#### Python示例
```python
import openai

# 配置客户端
client = openai.OpenAI(
    api_key="berry-admin-token-12345",
    base_url="http://localhost:3000/v1"
)

# 发送请求
response = client.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "user", "content": "Hello, world!"}
    ],
    stream=False
)

print(response.choices[0].message.content)
```

### 3. 获取可用模型
```bash
curl http://localhost:3000/v1/models \
  -H "Authorization: Bearer berry-admin-token-12345"
```

响应示例：
```json
{
  "object": "list",
  "data": [
    {
      "id": "gpt-4",
      "object": "model",
      "created": 1677610602,
      "owned_by": "berry-api"
    },
    {
      "id": "gpt-3.5-turbo",
      "object": "model",
      "created": 1677610602,
      "owned_by": "berry-api"
    }
  ]
}
```

### 4. 健康检查
```bash
# 基础健康检查
curl http://localhost:3000/health

# OpenAI兼容健康检查
curl http://localhost:3000/v1/health
```

### 5. 服务指标
```bash
curl http://localhost:3000/metrics
```

响应示例：
```json
{
  "providers": {
    "openai-primary": {
      "healthy": true,
      "total_requests": 1250,
      "successful_requests": 1200,
      "failed_requests": 50,
      "average_latency_ms": 850,
      "last_check": "2024-01-15T10:30:00Z"
    }
  },
  "models": {
    "gpt-4": {
      "total_requests": 800,
      "successful_requests": 780,
      "failed_requests": 20
    }
  }
}
```

## 📊 API端点总览

| 端点 | 方法 | 认证 | 描述 |
|------|------|------|------|
| `/` | GET | 否 | 服务首页 |
| `/health` | GET | 否 | 服务健康状态 |
| `/metrics` | GET | 否 | 详细性能指标 |
| `/models` | GET | 是 | 可用模型列表 |
| `/v1/chat/completions` | POST | 是 | 聊天完成（OpenAI兼容） |
| `/v1/models` | GET | 是 | 可用模型列表（OpenAI兼容） |
| `/v1/health` | GET | 否 | OpenAI兼容健康检查 |

## 🔧 负载均衡策略详解

### 策略选择指南

| 策略 | 适用场景 | 优势 | 劣势 |
|------|----------|------|------|
| `weighted_random` | 成本控制、按性能分配 | 灵活的权重分配 | 可能不够均匀 |
| `round_robin` | 简单均衡、相同性能后端 | 完全均匀分配 | 不考虑后端性能差异 |
| `least_latency` | 性能优化、延迟敏感 | 自动选择最快后端 | 需要延迟统计 |
| `failover` | 高可用、主备场景 | 明确的优先级 | 主后端压力大 |
| `random` | 简单场景、测试 | 实现简单 | 无优化策略 |
| `weighted_failover` | 智能负载均衡 | 结合权重和故障转移 | 配置相对复杂 |

### 1. 加权随机 (weighted_random)
根据权重随机选择后端，适合按成本或性能分配流量：
```toml
[models.cost_optimized]
name = "cost-optimized"
strategy = "weighted_random"
enabled = true

[[models.cost_optimized.backends]]
provider = "cheap-provider"
model = "gpt-3.5-turbo"
weight = 0.7  # 70% 流量给便宜的服务
priority = 1
enabled = true

[[models.cost_optimized.backends]]
provider = "premium-provider"
model = "gpt-3.5-turbo"
weight = 0.3  # 30% 流量给高质量服务
priority = 2
enabled = true
```

### 2. 轮询 (round_robin)
依次轮询所有可用后端，适合性能相近的后端：
```toml
[models.balanced]
name = "balanced"
strategy = "round_robin"
enabled = true

[[models.balanced.backends]]
provider = "provider-a"
model = "gpt-4"
weight = 1.0  # 轮询中权重无效
priority = 1
enabled = true

[[models.balanced.backends]]
provider = "provider-b"
model = "gpt-4"
weight = 1.0
priority = 2
enabled = true
```

### 3. 最低延迟 (least_latency)
自动选择响应时间最短的后端：
```toml
[models.fast_response]
name = "fast-response"
strategy = "least_latency"
enabled = true

[[models.fast_response.backends]]
provider = "fast-provider"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 1
enabled = true

[[models.fast_response.backends]]
provider = "slow-provider"
model = "gpt-3.5-turbo"
weight = 1.0
priority = 2
enabled = true
```

### 4. 故障转移 (failover)
按优先级顺序选择，主要用于主备场景：
```toml
[models.high_availability]
name = "high-availability"
strategy = "failover"
enabled = true

[[models.high_availability.backends]]
provider = "primary-provider"
model = "gpt-4"
weight = 1.0
priority = 1  # 最高优先级，优先使用
enabled = true

[[models.high_availability.backends]]
provider = "backup-provider"
model = "gpt-4"
weight = 1.0
priority = 2  # 备用，主服务故障时使用
enabled = true

[[models.high_availability.backends]]
provider = "emergency-provider"
model = "gpt-4"
weight = 1.0
priority = 3  # 应急，前两个都故障时使用
enabled = true
```

### 5. 权重故障转移 (weighted_failover) 🆕
结合权重选择和故障转移的智能策略：

**工作原理**：
1. **正常情况**: 从所有健康的后端中按权重随机选择
2. **故障情况**: 自动屏蔽不健康的后端，只在健康的后端中选择
3. **全部故障**: 如果所有后端都不健康，仍按权重选择（而非优先级）
4. **自动恢复**: 后端恢复健康后自动重新加入负载均衡

```toml
[models.smart_model]
name = "smart-model"
strategy = "weighted_failover"
enabled = true

[[models.smart_model.backends]]
provider = "openai-main"
model = "gpt-4"
weight = 0.6    # 60%权重 - 主要服务
priority = 1    # 最高优先级
enabled = true

[[models.smart_model.backends]]
provider = "openai-backup"
model = "gpt-4"
weight = 0.3    # 30%权重 - 备用服务
priority = 2    # 中等优先级
enabled = true

[[models.smart_model.backends]]
provider = "azure"
model = "gpt-4"
weight = 0.1    # 10%权重 - 应急服务
priority = 3    # 最低优先级
enabled = true
```

### 6. 随机 (random)
完全随机选择，适合简单场景：
```toml
[models.simple_random]
name = "simple-random"
strategy = "random"
enabled = true

[[models.simple_random.backends]]
provider = "provider-a"
model = "gpt-3.5-turbo"
weight = 1.0  # 随机策略中权重无效
priority = 1
enabled = true
```

## 🏥 健康检查与故障处理

### 健康检查配置
```toml
[settings]
health_check_interval_seconds = 30    # 检查间隔（秒）
circuit_breaker_failure_threshold = 5 # 熔断阈值
circuit_breaker_timeout_seconds = 60  # 熔断恢复时间（秒）
```

### 健康检查机制
1. **定期检查**: 每30秒自动检查所有Provider的健康状态
2. **模型列表验证**: 通过调用 `/v1/models` 端点验证服务可用性
3. **聊天请求测试**: 发送简单的聊天请求验证模型功能
4. **自动标记**: 根据检查结果自动标记Provider为健康/不健康

### 故障转移流程
当某个Provider出现故障时：

1. **故障检测**
   - API请求失败
   - 健康检查失败
   - 响应超时

2. **自动处理**
   - 立即标记为不健康
   - 将流量切换到其他健康的Provider
   - 记录故障指标

3. **恢复检测**
   - 定期重试故障的Provider
   - 健康检查通过后自动恢复
   - 用户请求成功也会触发恢复

4. **流量恢复**
   - 恢复后自动重新加入负载均衡
   - 按配置的权重分配流量

### 熔断机制
```
正常状态 ──失败次数达到阈值──▶ 熔断状态
    ▲                           │
    │                           │
    └──超时后自动尝试恢复────────┘
```

- **触发条件**: 连续失败次数达到 `circuit_breaker_failure_threshold`
- **熔断期间**: 不会向该Provider发送请求
- **自动恢复**: 超过 `circuit_breaker_timeout_seconds` 后自动尝试恢复

### 故障处理最佳实践

1. **多Provider配置**: 为每个模型配置多个Provider
2. **合理的权重分配**: 主Provider权重高，备用Provider权重低
3. **适当的超时设置**: 避免过长的等待时间
4. **监控告警**: 定期检查健康状态和指标

## 🧪 测试与调试

### 1. 单元测试
```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test loadbalance
cargo test config
cargo test auth

# 运行集成测试
cargo test --test integration

# 显示测试输出
cargo test -- --nocapture
```

### 2. 功能测试
```bash
# 测试基本功能
./test_auth.sh

# 调试演示
./debug_demo.sh

# 健康检查演示
cargo run --example initial_health_check_demo
```

### 3. 调试日志
启用详细日志进行调试：
```bash
# 启用调试日志
RUST_LOG=debug cargo run

# 只显示特定模块的日志
RUST_LOG=berry_api_api=debug cargo run

# 显示所有日志级别
RUST_LOG=trace cargo run
```

### 4. 配置验证
```bash
# 验证配置文件语法
cargo run -- --check-config

# 使用测试配置
CONFIG_PATH="test_config.toml" cargo run
```

### 5. 性能测试
```bash
# 使用 wrk 进行压力测试
wrk -t12 -c400 -d30s --script=test.lua http://localhost:3000/v1/chat/completions

# 使用 curl 测试延迟
time curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer berry-admin-token-12345" \
  -d '{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}'
```

## 📈 性能优化与部署

### 性能调优建议

1. **连接池优化**
   ```toml
   [settings]
   request_timeout_seconds = 30      # 根据网络情况调整
   max_retries = 3                   # 避免过多重试
   health_check_interval_seconds = 30 # 平衡检查频率和性能
   ```

2. **权重分配策略**
   - 根据Provider的实际性能和成本调整权重
   - 高性能Provider分配更高权重
   - 备用Provider保持较低权重

3. **超时设置**
   - 设置合理的请求超时时间
   - 避免过长的等待导致用户体验差
   - 考虑不同Provider的响应特性

4. **熔断参数**
   ```toml
   circuit_breaker_failure_threshold = 5  # 根据容错需求调整
   circuit_breaker_timeout_seconds = 60   # 平衡恢复速度和稳定性
   ```

### 监控与告警

1. **关键指标监控**
   - Provider健康状态
   - 请求成功率
   - 平均响应时间
   - 错误率统计

2. **日志分析**
   ```bash
   # 查看错误日志
   grep "ERROR" logs/berry-api.log

   # 监控健康检查
   grep "health_check" logs/berry-api.log

   # 分析性能指标
   grep "latency" logs/berry-api.log
   ```

### 生产部署

1. **Docker部署**
   ```dockerfile
   FROM rust:1.70 as builder
   WORKDIR /app
   COPY . .
   RUN cargo build --release

   FROM debian:bookworm-slim
   RUN apt-get update && apt-get install -y ca-certificates
   COPY --from=builder /app/target/release/berry-api /usr/local/bin/
   COPY config.toml /etc/berry-api/
   EXPOSE 3000
   CMD ["berry-api"]
   ```

2. **Systemd服务**
   ```ini
   [Unit]
   Description=Berry API Load Balancer
   After=network.target

   [Service]
   Type=simple
   User=berry-api
   WorkingDirectory=/opt/berry-api
   Environment=CONFIG_PATH=/etc/berry-api/config.toml
   Environment=RUST_LOG=info
   ExecStart=/usr/local/bin/berry-api
   Restart=always
   RestartSec=5

   [Install]
   WantedBy=multi-user.target
   ```

3. **负载均衡部署**
   - 使用Nginx或HAProxy进行前端负载均衡
   - 部署多个Berry API实例
   - 配置健康检查和故障转移

4. **安全配置**
   - 使用HTTPS加密传输
   - 定期轮换API密钥
   - 限制网络访问权限
   - 启用访问日志审计

### 扩展性

1. **水平扩展**
   - 支持多实例部署
   - 无状态设计，易于扩展
   - 配置文件共享

2. **动态配置**
   - 支持运行时配置更新
   - 热重载Provider配置
   - 动态添加新模型

3. **插件化架构**
   - 可扩展的认证机制
   - 自定义负载均衡策略
   - 可插拔的监控组件

## 🔧 故障排除

### 常见问题

1. **服务启动失败**
   ```bash
   # 检查配置文件语法
   cargo run -- --check-config

   # 检查端口占用
   lsof -i :3000

   # 查看详细错误信息
   RUST_LOG=debug cargo run
   ```

2. **Provider连接失败**
   - 检查API密钥是否正确
   - 验证网络连接
   - 确认base_url格式正确
   - 检查防火墙设置

3. **认证失败**
   - 确认Token配置正确
   - 检查用户是否启用
   - 验证模型访问权限

4. **负载均衡不工作**
   - 检查Provider健康状态
   - 验证权重配置
   - 查看负载均衡策略设置

### 日志分析

```bash
# 查看服务启动日志
grep "Starting Berry API" logs/berry-api.log

# 检查健康检查状态
grep "health_check" logs/berry-api.log

# 查看认证失败
grep "Authentication failed" logs/berry-api.log

# 监控负载均衡决策
grep "selected backend" logs/berry-api.log
```

## 🤝 贡献指南

### 开发环境设置
```bash
# 克隆项目
git clone https://github.com/PPKunOfficial/berry-api.git
cd berry-api

# 安装依赖
cargo build

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 提交规范
- 使用清晰的commit message
- 添加相应的测试用例
- 更新相关文档
- 确保所有测试通过

### 贡献类型
- 🐛 Bug修复
- ✨ 新功能
- 📚 文档改进
- 🎨 代码优化
- 🧪 测试增强

欢迎提交Issue和Pull Request！

## 📄 许可证

本项目采用 GNU GENERAL PUBLIC LICENSE Version 3 许可证。

详细信息请查看 [LICENSE](LICENSE) 文件。

## 🔗 相关资源

### 官方文档
- [OpenAI API文档](https://platform.openai.com/docs/api-reference)
- [Azure OpenAI文档](https://docs.microsoft.com/en-us/azure/cognitive-services/openai/)
- [Anthropic API文档](https://docs.anthropic.com/claude/reference/)

### 技术栈
- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [Tokio](https://tokio.rs/) - 异步运行时
- [Axum](https://github.com/tokio-rs/axum) - Web框架
- [Serde](https://serde.rs/) - 序列化框架
- [TOML](https://toml.io/) - 配置文件格式

### 社区
- [GitHub Issues](https://github.com/PPKunOfficial/berry-api/issues) - 问题反馈
- [GitHub Discussions](https://github.com/PPKunOfficial/berry-api/discussions) - 讨论交流

---

**Berry API** - 让AI服务负载均衡变得简单高效！ 🚀
