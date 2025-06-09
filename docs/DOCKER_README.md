# Berry API Docker 部署指南

## 🚀 快速开始

### 1. 准备配置文件

```bash
# 复制配置模板
cp docker/config.toml config.toml

# 编辑配置文件，填入你的 API 密钥
vim config.toml
```

**重要：** 请确保在 `config.toml` 中填入真实的 API 密钥，替换所有 `your-*-here` 占位符。

### 2. 启动服务

```bash
# 使用 docker-compose 启动
docker-compose up -d

# 查看日志
docker-compose logs -f berry-api
```

### 3. 验证部署

```bash
# 检查健康状态
curl http://localhost:3000/health

# 获取可用模型
curl http://localhost:3000/v1/models

# 发送测试请求
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-admin-token-here" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## 🔧 常用命令

```bash
# 停止服务
docker-compose down

# 重新构建并启动
docker-compose up --build -d

# 查看容器状态
docker-compose ps

# 进入容器
docker-compose exec berry-api bash
```

## 📝 配置说明

编辑 `config.toml` 文件时，请确保：

1. 替换所有 `your-*-here` 占位符为真实值
2. 配置至少一个有效的 AI 服务提供商
3. 设置安全的用户令牌
4. 根据需要调整负载均衡策略

## 🔒 安全提醒

- 使用强随机令牌
- 不要将包含真实 API 密钥的配置文件提交到版本控制
- 在生产环境中使用 HTTPS
- 定期轮换 API 密钥
