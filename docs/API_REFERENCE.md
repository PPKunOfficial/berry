# Berry API 接口参考文档

本文档详细描述了Berry API的所有HTTP接口，包括请求格式、响应格式和错误处理。

## 📋 目录

- [认证](#认证)
- [聊天完成接口](#聊天完成接口)
- [模型列表接口](#模型列表接口)
- [健康检查接口](#健康检查接口)
- [指标接口](#指标接口)
- [错误处理](#错误处理)

## 🔐 认证

所有需要认证的API都使用Bearer Token认证方式。

### 请求头格式
```
Authorization: Bearer <your-token>
```

### 认证流程
1. 在配置文件中配置用户Token
2. 客户端在请求头中包含Token
3. 服务器验证Token有效性和权限
4. 返回相应结果或错误

## 💬 聊天完成接口

### POST /v1/chat/completions

与OpenAI Chat Completions API完全兼容的聊天完成接口。

#### 请求参数

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| model | string | 是 | 模型名称 |
| messages | array | 是 | 消息数组 |
| stream | boolean | 否 | 是否流式响应，默认false |
| max_tokens | integer | 否 | 最大token数 |
| temperature | number | 否 | 温度参数，0-2 |
| top_p | number | 否 | Top-p参数，0-1 |
| n | integer | 否 | 生成的响应数量 |
| stop | string/array | 否 | 停止序列 |
| presence_penalty | number | 否 | 存在惩罚，-2到2 |
| frequency_penalty | number | 否 | 频率惩罚，-2到2 |
| user | string | 否 | 用户标识 |

#### 消息格式

```json
{
  "role": "user|assistant|system",
  "content": "消息内容"
}
```

#### 请求示例

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user",
        "content": "Hello, how are you?"
      }
    ],
    "max_tokens": 1000,
    "temperature": 0.7,
    "stream": false
  }'
```

#### 非流式响应

```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-4",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm doing well, thank you for asking. How can I help you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 18,
    "total_tokens": 38
  }
}
```

#### 流式响应

流式响应使用Server-Sent Events (SSE) 格式：

```
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4","choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4","choices":[{"index":0,"delta":{"content":"!"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

## 📋 模型列表接口

### GET /v1/models

获取当前用户可访问的模型列表。

#### 请求示例

```bash
curl http://localhost:3000/v1/models \
  -H "Authorization: Bearer your-token"
```

#### 响应格式

```json
{
  "object": "list",
  "data": [
    {
      "id": "gpt-4",
      "object": "model",
      "created": 1677610602,
      "owned_by": "berry-api",
      "permission": [],
      "root": "gpt-4",
      "parent": null
    },
    {
      "id": "gpt-3.5-turbo",
      "object": "model",
      "created": 1677610602,
      "owned_by": "berry-api",
      "permission": [],
      "root": "gpt-3.5-turbo",
      "parent": null
    }
  ]
}
```

### GET /models

获取所有可用模型列表（需要认证）。

#### 响应格式

```json
{
  "models": [
    {
      "name": "gpt-4",
      "enabled": true,
      "strategy": "weighted_random",
      "backends": [
        {
          "provider": "openai-primary",
          "model": "gpt-4",
          "weight": 0.7,
          "priority": 1,
          "enabled": true,
          "healthy": true
        }
      ]
    }
  ]
}
```

## 🏥 健康检查接口

### GET /health

获取服务整体健康状态，无需认证。

#### 响应格式

```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "providers": {
    "openai-primary": {
      "healthy": true,
      "last_check": "2024-01-15T10:29:30Z",
      "error": null
    },
    "azure-openai": {
      "healthy": false,
      "last_check": "2024-01-15T10:29:30Z",
      "error": "Connection timeout"
    }
  },
  "models": {
    "gpt-4": {
      "available": true,
      "healthy_backends": 2,
      "total_backends": 3
    }
  }
}
```

### GET /v1/health

OpenAI兼容的健康检查接口，无需认证。

#### 响应格式

```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## 📊 指标接口

### GET /metrics

获取详细的性能指标和统计信息，无需认证。

#### 响应格式

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "uptime_seconds": 86400,
  "providers": {
    "openai-primary": {
      "healthy": true,
      "total_requests": 1250,
      "successful_requests": 1200,
      "failed_requests": 50,
      "success_rate": 0.96,
      "average_latency_ms": 850,
      "last_success": "2024-01-15T10:29:45Z",
      "last_failure": "2024-01-15T09:15:30Z",
      "circuit_breaker_state": "closed"
    }
  },
  "models": {
    "gpt-4": {
      "total_requests": 800,
      "successful_requests": 780,
      "failed_requests": 20,
      "success_rate": 0.975,
      "average_latency_ms": 900
    }
  },
  "load_balancer": {
    "total_selections": 1250,
    "strategy_usage": {
      "weighted_random": 800,
      "failover": 300,
      "least_latency": 150
    }
  },
  "authentication": {
    "total_requests": 1300,
    "successful_authentications": 1250,
    "failed_authentications": 50,
    "success_rate": 0.962
  }
}
```

## ❌ 错误处理

### 错误响应格式

所有错误都遵循统一的响应格式：

```json
{
  "error": {
    "type": "authentication_error",
    "code": "invalid_token",
    "message": "The provided token is invalid or expired",
    "details": {
      "timestamp": "2024-01-15T10:30:00Z",
      "request_id": "req_123456"
    }
  }
}
```

### 常见错误类型

#### 1. 认证错误 (401)

```json
{
  "error": {
    "type": "authentication_error",
    "code": "missing_token",
    "message": "Authorization header is required"
  }
}
```

```json
{
  "error": {
    "type": "authentication_error",
    "code": "invalid_token",
    "message": "The provided token is invalid"
  }
}
```

#### 2. 权限错误 (403)

```json
{
  "error": {
    "type": "permission_error",
    "code": "model_not_allowed",
    "message": "You don't have permission to access this model",
    "details": {
      "model": "gpt-4",
      "allowed_models": ["gpt-3.5-turbo"]
    }
  }
}
```

#### 3. 请求错误 (400)

```json
{
  "error": {
    "type": "invalid_request",
    "code": "missing_parameter",
    "message": "Missing required parameter: model"
  }
}
```

```json
{
  "error": {
    "type": "invalid_request",
    "code": "invalid_model",
    "message": "The specified model does not exist",
    "details": {
      "model": "invalid-model-name"
    }
  }
}
```

#### 4. 服务错误 (500)

```json
{
  "error": {
    "type": "service_error",
    "code": "no_healthy_backends",
    "message": "No healthy backends available for the requested model",
    "details": {
      "model": "gpt-4",
      "total_backends": 3,
      "healthy_backends": 0
    }
  }
}
```

```json
{
  "error": {
    "type": "service_error",
    "code": "upstream_error",
    "message": "All upstream providers failed",
    "details": {
      "attempts": 3,
      "last_error": "Connection timeout"
    }
  }
}
```

#### 5. 速率限制 (429)

```json
{
  "error": {
    "type": "rate_limit_error",
    "code": "rate_limit_exceeded",
    "message": "Rate limit exceeded",
    "details": {
      "retry_after": 60
    }
  }
}
```

### HTTP状态码

| 状态码 | 描述 |
|--------|------|
| 200 | 请求成功 |
| 400 | 请求参数错误 |
| 401 | 认证失败 |
| 403 | 权限不足 |
| 404 | 资源不存在 |
| 429 | 速率限制 |
| 500 | 服务器内部错误 |
| 502 | 上游服务错误 |
| 503 | 服务不可用 |

---

这份API参考文档提供了Berry API所有接口的详细说明。如需更多信息，请参考主README文档。
