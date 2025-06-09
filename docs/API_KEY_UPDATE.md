# API密钥配置更新说明

## 🔄 更新内容

根据您的要求，我已经将API密钥的配置方式从环境变量改为直接在TOML配置文件中配置。

## 📝 配置变更

### 之前的配置方式（环境变量）
```toml
[providers.openai-primary]
name = "OpenAI Primary Account"
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY_PRIMARY"  # 从环境变量读取
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
```

需要设置环境变量：
```bash
export OPENAI_API_KEY_PRIMARY="sk-your-key"
```

### 现在的配置方式（直接配置）
```toml
[providers.openai-primary]
name = "OpenAI Primary Account"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-openai-key-here"  # 直接在配置文件中设置
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true
```

不再需要环境变量，直接在配置文件中设置API密钥。

## 🔧 代码变更

### 1. 配置结构体更新
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Provider {
    pub name: String,
    pub base_url: String,
    pub api_key: String,  // 改为直接存储API密钥
    pub models: Vec<String>,
    // ... 其他字段
}
```

### 2. API密钥获取方式更新
```rust
// 之前：从环境变量获取
let api_key = std::env::var(&provider.api_key_env)?;

// 现在：直接从配置获取
let api_key = &provider.api_key;
```

### 3. 健康检查更新
```rust
// 直接使用配置中的API密钥
let api_key = &provider.api_key;

if api_key.is_empty() {
    warn!("API key is empty for provider {}", provider_id);
    return;
}
```

## 📋 配置文件示例

### 完整配置示例
```toml
# 全局设置
[settings]
health_check_interval_seconds = 30
request_timeout_seconds = 30
max_retries = 3

# Provider配置
[providers.openai-main]
name = "OpenAI Main Account"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-openai-api-key-here"
models = ["gpt-4", "gpt-3.5-turbo"]
enabled = true

[providers.azure-openai]
name = "Azure OpenAI"
base_url = "https://your-resource.openai.azure.com"
api_key = "your-azure-api-key-here"
models = ["gpt-4", "gpt-35-turbo"]
enabled = true
[providers.azure-openai.headers]
"api-version" = "2024-02-01"

# 模型映射
[models.gpt_4]
name = "gpt-4"
strategy = "weighted_random"
enabled = true

[[models.gpt_4.backends]]
provider = "openai-main"
model = "gpt-4"
weight = 0.7
priority = 1
enabled = true

[[models.gpt_4.backends]]
provider = "azure-openai"
model = "gpt-4"
weight = 0.3
priority = 2
enabled = true
```

## 🚀 使用方法

### 1. 更新配置文件
将您的API密钥直接写入配置文件：
```bash
cp config_simple.toml config.toml
# 编辑config.toml，填入真实的API密钥
```

### 2. 启动服务
```bash
# 可选：指定配置文件路径
export CONFIG_PATH="config.toml"

# 启动服务
cargo run
```

### 3. 测试配置
```bash
# 检查健康状态
curl http://localhost:3000/health

# 获取可用模型
curl http://localhost:3000/v1/models

# 发送聊天请求
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer any-key" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## ✅ 优势

1. **简化配置**：不需要管理环境变量
2. **集中管理**：所有配置都在一个文件中
3. **易于部署**：只需要配置文件，不需要设置环境
4. **版本控制友好**：可以将配置文件（去除敏感信息后）纳入版本控制

## ⚠️ 安全注意事项

1. **保护配置文件**：确保配置文件的访问权限正确设置
2. **不要提交密钥**：不要将包含真实API密钥的配置文件提交到版本控制
3. **使用模板**：可以创建配置模板文件，部署时替换为真实密钥

## 📁 相关文件

- `config_simple.toml` - 简化的配置示例
- `config_example.toml` - 完整的配置示例
- `test_config.toml` - 测试配置

所有配置文件都已更新为新的API密钥配置方式。
