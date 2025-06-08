# 健康检查系统升级说明

## 概述

本次升级大幅增强了Berry API的健康检查系统，实现了您要求的高级功能：

1. **真实API验证**：使用model list和chat请求检查provider可用性
2. **智能不健康列表管理**：自动跟踪和恢复失败的provider
3. **内部重试机制**：避免直接向用户返回错误
4. **恢复检查机制**：定期检查不健康provider是否已恢复

## 主要改进

### 1. 真实API健康检查

**之前**：只检查基础HTTP连接
**现在**：
- 对真实AI服务使用 `/v1/models` API检查可用性
- 对测试服务保持原有的HTTP状态检查
- 验证API密钥有效性和服务响应

### 2. 不健康列表管理

新增 `UnhealthyBackend` 结构跟踪失败信息：
```rust
pub struct UnhealthyBackend {
    pub backend_key: String,
    pub first_failure_time: Instant,
    pub last_failure_time: Instant,
    pub failure_count: u32,
    pub last_recovery_attempt: Option<Instant>,
    pub recovery_attempts: u32,
}
```

**功能**：
- 自动记录失败时间和次数
- 跟踪恢复尝试历史
- 智能判断是否需要恢复检查

### 3. 恢复检查机制

**工作原理**：
1. 定期扫描不健康列表
2. 使用简单的chat请求测试provider恢复状态
3. 成功响应后自动将provider标记为健康
4. 失败则继续保持不健康状态

**恢复检查请求示例**：
```json
{
    "model": "gpt-3.5-turbo",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 1,
    "stream": false
}
```

### 4. 内部重试机制

**智能后端选择**：
- 优先选择健康的backend
- 失败时自动重试其他可用backend
- 最多重试 `max_internal_retries` 次
- 避免直接向用户返回错误

**重试流程**：
1. 选择backend → 检查健康状态
2. 如果不健康 → 重新选择
3. 发送请求失败 → 标记为不健康并重试
4. 所有重试失败 → 返回错误给用户

## 新增配置选项

在 `[settings]` 部分添加了三个新配置：

```toml
[settings]
# 原有配置...
health_check_interval_seconds = 60
request_timeout_seconds = 10
max_retries = 2

# 新增配置
recovery_check_interval_seconds = 120  # 恢复检查间隔（秒）
max_internal_retries = 2               # 内部重试次数
health_check_timeout_seconds = 10      # 健康检查超时时间（秒）
```

### 配置说明

- **recovery_check_interval_seconds**: 不健康provider的恢复检查间隔，建议120-300秒
- **max_internal_retries**: 请求失败时的内部重试次数，建议2-3次
- **health_check_timeout_seconds**: 健康检查请求的超时时间，建议5-15秒

## 使用场景

### 场景1：Provider临时故障
1. 用户请求 → provider返回错误
2. 系统自动标记为不健康
3. 后续请求自动路由到其他健康provider
4. 恢复检查定期测试故障provider
5. 恢复后自动重新启用

### 场景2：API密钥失效
1. 健康检查发现API密钥无效
2. 自动标记相关models为不健康
3. 请求自动路由到其他provider
4. 管理员更新密钥后自动恢复

### 场景3：网络波动
1. 短暂网络问题导致请求失败
2. 内部重试机制尝试其他backend
3. 成功后用户无感知
4. 避免因临时问题影响用户体验

## API变化

### 新增方法

**MetricsCollector**:
- `get_unhealthy_backends()` - 获取不健康backend列表
- `needs_recovery_check()` - 检查是否需要恢复检查
- `record_recovery_attempt()` - 记录恢复尝试
- `is_in_unhealthy_list()` - 检查是否在不健康列表中

**HealthChecker**:
- `check_recovery()` - 执行恢复检查
- `check_recovery_with_chat()` - 使用chat请求检查恢复

**LoadBalanceService**:
- 增强的 `select_backend()` - 带智能重试的backend选择

## 监控和调试

### 日志级别

- **INFO**: 健康检查结果、恢复成功
- **WARN**: 健康检查失败、重试警告
- **ERROR**: 严重错误、所有重试失败
- **DEBUG**: 详细的选择和重试过程

### 关键日志示例

```
INFO  Recovery check passed for openai-primary:gpt-4 (245ms)
WARN  Selected backend openai-backup:gpt-4 is unhealthy, retrying... (attempt 1/3)
ERROR All retry attempts failed for model 'gpt-4': No available backends
```

## 性能影响

### 优化措施
- 恢复检查使用最小token数量（max_tokens: 1）
- 并发执行健康检查
- 智能间隔避免过度检查
- 缓存健康状态减少重复检查

### 资源消耗
- 额外内存：每个不健康backend约200字节
- 网络开销：恢复检查每次约1KB
- CPU开销：可忽略不计

## 向后兼容性

- 所有原有配置保持兼容
- 新配置有合理默认值
- 原有API接口不变
- 渐进式升级，无需重启

## 测试验证

新增了完整的集成测试：
- `test_health_check_basic_functionality` - 基础功能测试
- `test_unhealthy_backend_management` - 不健康列表管理测试
- `test_recovery_check_timing` - 恢复检查时机测试
- `test_smart_backend_selection` - 智能选择测试

运行测试：
```bash
cargo test health_check
```

## 总结

本次升级实现了您要求的所有功能：

✅ **真实API检查**：使用model list验证provider可用性
✅ **不健康列表管理**：自动跟踪和管理失败的provider
✅ **恢复检查机制**：使用chat请求定期检查恢复状态
✅ **内部重试机制**：避免直接向用户报错
✅ **智能故障转移**：自动路由到健康的backend

系统现在能够：
- 主动发现和隔离故障provider
- 自动恢复已修复的provider
- 提供无缝的故障转移体验
- 最大化系统可用性和用户体验
