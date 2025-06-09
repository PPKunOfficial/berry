# 错误处理改进说明

## 概述

本次更新大幅改进了Berry API的错误处理机制，现在当多次内部重试后仍然不可用时，系统会返回详细的错误信息和正确的HTTP状态码，而不是简单的200状态码加错误消息。

## 主要改进

### 1. 正确的HTTP状态码

现在系统会根据错误类型返回相应的HTTP状态码：

- **400 Bad Request**: 请求格式错误（如缺少model字段）
- **401 Unauthorized**: 认证失败（如无效token）
- **403 Forbidden**: 权限不足
- **404 Not Found**: 模型未找到
- **408 Request Timeout**: 请求超时
- **429 Too Many Requests**: 请求过多
- **500 Internal Server Error**: 服务器内部错误（如配置错误）
- **503 Service Unavailable**: 服务不可用（如所有后端不健康）
- **504 Gateway Timeout**: 网关超时

### 2. 详细的错误信息

错误响应现在包含更多有用信息：

```json
{
  "error": {
    "message": "Service temporarily unavailable for model 'gpt-4'",
    "type": "ServiceUnavailable",
    "status": 503,
    "details": "All backends are currently unhealthy or unavailable. Backend selection failed after 3 internal retries for model 'gpt-4': No healthy backends available. Total backends: 3, Enabled: 3, Healthy: 0."
  }
}
```

### 3. 智能错误分类

系统会根据错误消息内容自动分类错误类型：

- 包含"unauthorized"、"invalid token"、"authentication" → 401 Unauthorized
- 包含"forbidden"、"permission"、"access denied" → 403 Forbidden  
- 包含"not found"、"model not" → 404 Not Found
- 包含"timeout"、"timed out" → 504 Gateway Timeout
- 包含"too many requests"、"rate limit" → 429 Too Many Requests
- 包含"service unavailable"、"no available backends"、"unhealthy" → 503 Service Unavailable
- 包含"bad request"、"invalid"（非token相关） → 400 Bad Request
- 其他情况 → 500 Internal Server Error

## 错误场景示例

### 场景1：所有后端不健康

**请求**:
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer your-token" \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}'
```

**响应** (HTTP 503):
```json
{
  "error": {
    "message": "Service temporarily unavailable for model 'gpt-4'",
    "type": "ServiceUnavailable", 
    "status": 503,
    "details": "All backends are currently unhealthy or unavailable. Details: Backend selection failed after 3 internal retries..."
  }
}
```

### 场景2：缺少模型字段

**请求**:
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer your-token" \
  -H "Content-Type: application/json" \
  -d '{"messages": [{"role": "user", "content": "Hello"}]}'
```

**响应** (HTTP 400):
```json
{
  "error": {
    "message": "Missing model field in request",
    "type": "BadRequest",
    "status": 400,
    "details": "The 'model' field is required in the request body"
  }
}
```

### 场景3：配置错误

**响应** (HTTP 500):
```json
{
  "error": {
    "message": "Configuration error for model 'gpt-4'",
    "type": "InternalServerError",
    "status": 500,
    "details": "Please contact system administrator to check backend configuration"
  }
}
```

### 场景4：请求超时

**响应** (HTTP 504):
```json
{
  "error": {
    "message": "Request timeout for model 'gpt-4'",
    "type": "GatewayTimeout",
    "status": 504,
    "details": "Request processing timed out after multiple attempts..."
  }
}
```

## 内部重试机制

系统在返回错误给用户之前会进行多次内部重试：

1. **后端选择重试**: 如果选择的后端不健康，会重新选择其他后端
2. **请求重试**: 如果请求失败，会尝试其他可用后端
3. **最大重试次数**: 由配置文件中的`max_internal_retries`控制（默认2次）

只有在所有重试都失败后，才会向用户返回详细的错误信息。

## 日志记录

系统会记录详细的错误日志，便于问题诊断：

```
ERROR berry_api_api::relay::handler::loadbalanced: All retry attempts failed for model 'gpt-4': Backend selection failed after 3 internal retries...
```

## 配置建议

为了获得最佳的错误处理体验，建议在配置文件中设置：

```toml
[settings]
max_internal_retries = 3  # 增加重试次数
request_timeout_seconds = 30  # 合理的超时时间
health_check_interval_seconds = 30  # 频繁的健康检查
```

## 客户端处理建议

客户端应该根据HTTP状态码采取不同的处理策略：

- **4xx错误**: 检查请求格式和认证信息，不要重试
- **503错误**: 服务暂时不可用，可以稍后重试
- **504错误**: 请求超时，可以重试但建议增加超时时间
- **500错误**: 服务器内部错误，联系管理员

这些改进使得Berry API的错误处理更加标准化和用户友好，便于客户端正确处理各种错误情况。
