# Berry API 用户认证指南

## 🔐 认证系统概述

Berry API 现在支持基于API密钥的用户认证系统，所有用户请求都需要提供有效的API密钥才能访问服务。

## 📋 配置用户令牌

### 1. 在TOML配置文件中定义用户

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
allowed_models = ["gpt-3.5-turbo", "fast-chat"]  # 只能访问这些模型
enabled = true
tags = ["user", "basic"]

# 高级用户 - 可以访问高级模型
[users.premium]
name = "Premium User"
token = "berry-premium-token-abcde"
allowed_models = ["gpt-4", "gpt-4-turbo", "premium", "claude_3"]
enabled = true
tags = ["premium", "advanced"]

# 禁用的用户
[users.disabled]
name = "Disabled User"
token = "berry-disabled-token-xyz"
allowed_models = ["gpt-3.5-turbo"]
enabled = false  # 已禁用，无法使用
tags = ["disabled"]
```

### 2. 用户配置字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | String | ✅ | 用户显示名称 |
| `token` | String | ✅ | API密钥令牌 |
| `allowed_models` | Array | ❌ | 允许访问的模型列表，空表示所有模型 |
| `enabled` | Boolean | ❌ | 是否启用用户，默认true |
| `rate_limit` | Object | ❌ | 速率限制配置（暂未实现） |
| `tags` | Array | ❌ | 用户标签，用于分类管理 |

## 🚀 API使用方法

### 1. 聊天完成请求

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer berry-user1-token-67890" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "user", "content": "Hello, world!"}
    ],
    "stream": false
  }'
```

### 2. 获取可用模型列表

```bash
curl http://localhost:3000/v1/models \
  -H "Authorization: Bearer berry-user1-token-67890"
```

**注意**：返回的模型列表会根据用户的`allowed_models`配置进行过滤。

### 3. 健康检查（无需认证）

```bash
curl http://localhost:3000/health
```

## 🔒 认证流程

### 1. 请求认证
- 客户端在请求头中包含`Authorization: Bearer <token>`
- 系统验证令牌是否存在且有效
- 检查用户是否启用

### 2. 权限检查
- 验证用户是否有权限访问请求的模型
- 如果`allowed_models`为空，允许访问所有模型
- 如果`allowed_models`有值，只允许访问列表中的模型

### 3. 错误响应

#### 无效令牌 (401)
```json
{
  "error": {
    "type": "invalid_token",
    "message": "The provided API key is invalid",
    "code": 401
  }
}
```

#### 模型访问被拒绝 (403)
```json
{
  "error": {
    "type": "model_access_denied",
    "message": "Access denied for model: gpt-4",
    "code": 403
  }
}
```

#### 用户已禁用 (403)
```json
{
  "error": {
    "type": "disabled_user",
    "message": "User account is disabled",
    "code": 403
  }
}
```

## 📊 用户管理最佳实践

### 1. 令牌安全
- 使用强随机字符串作为令牌
- 定期轮换API密钥
- 不要在日志中记录完整的令牌

### 2. 权限管理
- 遵循最小权限原则
- 根据用户需求分配模型访问权限
- 使用标签进行用户分类管理

### 3. 监控和审计
- 监控API使用情况
- 记录认证失败事件
- 定期审查用户权限

## 🔧 配置示例

### 基础配置
```toml
# 基础用户 - 只能使用经济型模型
[users.basic]
name = "Basic User"
token = "berry-basic-user-token"
allowed_models = ["gpt-3.5-turbo", "economy"]
enabled = true
tags = ["basic", "limited"]

# 高级用户 - 可以使用所有模型
[users.premium]
name = "Premium User"
token = "berry-premium-user-token"
allowed_models = []  # 允许所有模型
enabled = true
tags = ["premium", "unlimited"]
```

### 企业配置
```toml
# 开发团队
[users.dev-team]
name = "Development Team"
token = "berry-dev-team-token"
allowed_models = ["gpt-4", "gpt-3.5-turbo", "test"]
enabled = true
tags = ["development", "internal"]

# 生产环境
[users.production]
name = "Production Service"
token = "berry-prod-service-token"
allowed_models = ["gpt-4", "premium"]
enabled = true
tags = ["production", "critical"]

# 测试环境
[users.testing]
name = "Testing Environment"
token = "berry-test-env-token"
allowed_models = ["test", "economy"]
enabled = true
tags = ["testing", "sandbox"]
```

## 🚦 故障排除

### 1. 常见问题

**Q: 为什么我的请求返回401错误？**
A: 检查Authorization头是否正确设置，令牌是否有效，用户是否启用。

**Q: 为什么我无法访问某个模型？**
A: 检查用户的`allowed_models`配置，确保包含要访问的模型。

**Q: 如何添加新用户？**
A: 在配置文件中添加新的`[users.xxx]`部分，重启服务或热重载配置。

### 2. 调试技巧

- 检查服务日志中的认证相关信息
- 使用`/health`端点验证服务状态
- 使用`/v1/models`端点查看用户可访问的模型

## 🔄 配置热重载

系统支持配置热重载，可以在不重启服务的情况下更新用户配置：

```bash
# 修改配置文件后，发送重载请求（功能待实现）
curl -X POST http://localhost:3000/admin/reload \
  -H "Authorization: Bearer admin-token"
```

## 📈 未来功能

- [ ] 速率限制实现
- [ ] 用户使用统计
- [ ] 动态用户管理API
- [ ] JWT令牌支持
- [ ] 细粒度权限控制

这套认证系统为Berry API提供了企业级的安全保障，确保只有授权用户才能访问AI服务。
