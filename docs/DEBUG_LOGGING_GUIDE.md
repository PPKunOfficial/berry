# Berry API Debug 日志使用指南

## 概述

Berry API 现在支持完整的 debug 日志功能，让您能够详细观察健康检查系统的工作过程。

## 🚀 快速开始

### 1. 运行演示程序（推荐）

```bash
# 查看完整的 debug 日志演示
RUST_LOG=debug cargo run --example debug_logging_demo

# 只查看关键信息
RUST_LOG=info cargo run --example debug_logging_demo

# 只查看警告和错误
RUST_LOG=warn cargo run --example debug_logging_demo
```

### 2. 运行主程序

```bash
# 需要先创建配置文件
cp test_config.toml config.toml

# 启动服务器并查看 debug 日志
RUST_LOG=debug cargo run

# 启动服务器并查看关键信息
RUST_LOG=info cargo run
```

### 3. 运行测试

```bash
# 运行健康检查测试并查看 debug 日志
RUST_LOG=debug cargo test health_check

# 运行所有负载均衡测试
RUST_LOG=debug cargo test loadbalance

# 运行 debug 日志功能测试
RUST_LOG=debug cargo test debug_logging
```

## 📊 日志级别说明

### RUST_LOG=error
只显示错误信息，适用于生产环境
```
ERROR Provider failing-provider models API error: HTTP请求失败
```

### RUST_LOG=warn  
显示警告和错误，适用于生产监控
```
WARN  Provider test-provider health check failed with status: 500
ERROR Provider failing-provider models API error: HTTP请求失败
```

### RUST_LOG=info
显示关键操作信息，适用于日常运维
```
INFO  Starting Berry API server...
INFO  Load balance service started
INFO  Manual health check triggered
INFO  Recovery check passed for openai-primary:gpt-4 (245ms)
```

### RUST_LOG=debug
显示详细调试信息，适用于问题诊断
```
DEBUG Starting health check for 2 enabled providers
DEBUG Scheduling health check for provider: test-provider (Test Provider)
DEBUG API key present for provider test-provider, proceeding with health check
DEBUG Detected test provider (httpbin), using HTTP status check
DEBUG Testing provider test-provider with URL: https://httpbin.org/status/200
DEBUG Sending HTTP request to test provider test-provider
DEBUG Received response with status: 200 OK (245ms)
DEBUG Provider test-provider health check passed, marking 1 models as healthy
DEBUG Marking backend test-provider:test-model as healthy (latency: 245ms)
DEBUG Recording success for backend: test-provider:test-model
DEBUG Reset failure count for test-provider:test-model to 0
DEBUG Marked backend test-provider:test-model as healthy
```

## 🔍 关键日志模式

### 健康检查过程
```
DEBUG Starting health check for N enabled providers
DEBUG Scheduling health check for provider: {provider_name}
DEBUG Starting health check for provider: {provider_id} (base_url: {url})
DEBUG API key present for provider {provider_id}, proceeding with health check
DEBUG Detected test provider (httpbin), using HTTP status check
DEBUG Testing provider {provider_id} with URL: {url}
DEBUG Received response with status: {status} ({latency}ms)
DEBUG Provider {provider_id} health check passed, marking N models as healthy
```

### 失败处理
```
ERROR Provider {provider_id} models API error: {error}
DEBUG Network/API error for provider {provider_id}, marking N models as unhealthy
DEBUG Recording failure for backend: {backend_key}
DEBUG Updated failure count for {backend_key}: {count}
DEBUG Adding new backend {backend_key} to unhealthy list
```

### 恢复检查
```
DEBUG Starting recovery check process (interval: {interval}s)
DEBUG Unhealthy backends: [{backend_list}]
DEBUG Evaluating recovery check for backend: {backend_key}
DEBUG Backend {backend_key} needs recovery check
DEBUG Starting chat-based recovery check for {provider_id}:{model}
DEBUG Sending chat request for recovery check
INFO  Recovery check passed for {provider_id}:{model} ({latency}ms)
DEBUG Marking backend {backend_key} as recovered and healthy
```

### 智能重试
```
DEBUG Backend selection attempt {attempt} for model '{model_name}'
DEBUG Load balancer selected backend: {provider_id}:{model}
DEBUG Health check for {provider_id}:{model}: {HEALTHY|UNHEALTHY}
DEBUG Selected backend {provider_id}:{model} is unhealthy, retrying...
DEBUG Selected healthy backend for model '{model_name}': provider='{provider_id}'
```

## 🛠️ 故障排查

### 问题：看不到 debug 日志
**解决方案：**
```bash
# 确保使用正确的环境变量
RUST_LOG=debug cargo run --example debug_logging_demo

# 检查是否有其他日志配置覆盖
unset RUST_LOG
export RUST_LOG=debug
cargo run --example debug_logging_demo
```

### 问题：日志太多难以阅读
**解决方案：**
```bash
# 只显示我们的模块日志
RUST_LOG=berry_api_api=debug cargo run --example debug_logging_demo

# 过滤特定组件
RUST_LOG=berry_api_api::loadbalance=debug cargo run --example debug_logging_demo

# 使用 grep 过滤关键信息
RUST_LOG=debug cargo run --example debug_logging_demo 2>&1 | grep "health_checker"
```

### 问题：配置文件找不到
**解决方案：**
```bash
# 复制示例配置
cp test_config.toml config.toml

# 或者指定配置文件路径
CONFIG_PATH=test_config.toml RUST_LOG=debug cargo run
```

## 📝 日志分析示例

### 分析健康检查性能
```bash
# 查看所有健康检查的响应时间
RUST_LOG=debug cargo run --example debug_logging_demo 2>&1 | grep "latency:"

# 输出示例：
# DEBUG Marking backend httpbin-provider:demo-model as healthy (latency: 1099ms)
# DEBUG Marking backend test-provider:test-model as healthy (latency: 245ms)
```

### 监控失败模式
```bash
# 查看所有失败记录
RUST_LOG=debug cargo run --example debug_logging_demo 2>&1 | grep "Recording failure"

# 输出示例：
# DEBUG Recording failure for backend: failing-provider:failing-demo-model
# DEBUG Updated failure count for failing-provider:failing-demo-model: 1
```

### 跟踪恢复过程
```bash
# 查看恢复检查过程
RUST_LOG=debug cargo run --example debug_logging_demo 2>&1 | grep "recovery"

# 输出示例：
# DEBUG Starting recovery check process (interval: 10s)
# DEBUG Recording recovery attempt for backend: failing-provider:failing-demo-model
# DEBUG Updated recovery attempt for failing-provider:failing-demo-model: attempt #1
```

## 🎯 生产环境建议

### 推荐的日志级别
- **开发环境**: `RUST_LOG=debug`
- **测试环境**: `RUST_LOG=info`  
- **生产环境**: `RUST_LOG=warn`
- **故障排查**: `RUST_LOG=debug`

### 日志轮转配置
```bash
# 使用 systemd 服务时的日志配置
[Service]
Environment=RUST_LOG=info
StandardOutput=journal
StandardError=journal

# 查看服务日志
journalctl -u berry-api -f
```

### 性能监控
```bash
# 监控健康检查性能
RUST_LOG=info cargo run 2>&1 | grep -E "(health check|Recovery check)" | while read line; do
    echo "$(date): $line"
done
```

## 🔧 自定义日志配置

### 模块级别控制
```bash
# 只显示健康检查相关日志
RUST_LOG=berry_api_api::loadbalance::health_checker=debug

# 显示多个模块的日志
RUST_LOG=berry_api_api::loadbalance=debug,berry_api_api::relay=info

# 排除某些模块的详细日志
RUST_LOG=debug,hyper=warn,reqwest=warn
```

### 输出格式控制
程序会自动包含文件名和行号信息，便于定位问题：
```
2025-06-07T05:22:23.339298Z DEBUG berry_api_api::loadbalance::health_checker: api/src/loadbalance/health_checker.rs:62: Starting health check for 2 enabled providers
```

## 📚 更多资源

- 查看 `debug_demo.sh` 脚本了解更多使用示例
- 运行 `cargo run --example debug_logging_demo` 查看完整演示
- 查看 `HEALTH_CHECK_UPGRADE.md` 了解系统架构
- 运行测试：`cargo test health_check` 验证功能

---

**提示**: 在生产环境中，建议使用 `RUST_LOG=info` 或 `RUST_LOG=warn` 以避免日志过多影响性能。只在需要调试问题时才使用 `RUST_LOG=debug`。
