# Berry API 文档索引

欢迎使用Berry API！这里是所有文档的索引页面，帮助您快速找到需要的信息。

## 📚 文档结构

### 🚀 快速开始
- **[README.md](README.md)** - 项目主文档，包含特性介绍、安装指南和基础使用
- **[config_example.toml](config_example.toml)** - 完整的配置示例文件
- **[config_simple.toml](config_simple.toml)** - 简化的配置示例

### 📖 详细指南
- **[USAGE_GUIDE.md](USAGE_GUIDE.md)** - 详细使用指南，包含高级配置和最佳实践
- **[API_REFERENCE.md](API_REFERENCE.md)** - 完整的API接口参考文档
- **[CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md)** - 各种场景的配置示例集合

### 🔧 开发相关
- **[api/examples/](api/examples/)** - 代码示例和演示程序
- **[test_auth.sh](test_auth.sh)** - 认证功能测试脚本
- **[debug_demo.sh](debug_demo.sh)** - 调试演示脚本

## 🎯 按需求查找文档

### 我是新用户，想快速开始
1. 阅读 [README.md](README.md) 的"快速开始"部分
2. 复制 [config_simple.toml](config_simple.toml) 作为起点
3. 参考 [API_REFERENCE.md](API_REFERENCE.md) 了解接口使用

### 我需要配置多Provider负载均衡
1. 查看 [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md) 的企业级配置
2. 参考 [USAGE_GUIDE.md](USAGE_GUIDE.md) 的负载均衡策略详解
3. 使用 [config_example.toml](config_example.toml) 作为模板

### 我需要设置用户权限管理
1. 查看 [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md) 的多租户配置
2. 参考 [USAGE_GUIDE.md](USAGE_GUIDE.md) 的用户认证配置
3. 阅读 [API_REFERENCE.md](API_REFERENCE.md) 的认证部分

### 我遇到了问题需要调试
1. 查看 [README.md](README.md) 的故障排除部分
2. 参考 [USAGE_GUIDE.md](USAGE_GUIDE.md) 的故障排除章节
3. 运行 [debug_demo.sh](debug_demo.sh) 查看调试信息

### 我需要优化性能
1. 阅读 [README.md](README.md) 的性能优化部分
2. 查看 [USAGE_GUIDE.md](USAGE_GUIDE.md) 的性能调优章节
3. 参考 [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md) 的高可用配置

### 我需要部署到生产环境
1. 查看 [README.md](README.md) 的生产部署部分
2. 参考 [USAGE_GUIDE.md](USAGE_GUIDE.md) 的最佳实践
3. 使用 [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md) 的企业级配置

## 📋 功能特性对照表

| 功能 | 主要文档 | 配置示例 | API参考 |
|------|----------|----------|---------|
| 基础安装配置 | [README.md](README.md) | [config_simple.toml](config_simple.toml) | - |
| 用户认证 | [USAGE_GUIDE.md](USAGE_GUIDE.md) | [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md) | [API_REFERENCE.md](API_REFERENCE.md) |
| 负载均衡 | [README.md](README.md) | [config_example.toml](config_example.toml) | - |
| 健康检查 | [README.md](README.md) | [USAGE_GUIDE.md](USAGE_GUIDE.md) | [API_REFERENCE.md](API_REFERENCE.md) |
| 聊天完成 | [README.md](README.md) | - | [API_REFERENCE.md](API_REFERENCE.md) |
| 模型管理 | [USAGE_GUIDE.md](USAGE_GUIDE.md) | [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md) | [API_REFERENCE.md](API_REFERENCE.md) |
| 故障转移 | [README.md](README.md) | [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md) | - |
| 性能监控 | [README.md](README.md) | [USAGE_GUIDE.md](USAGE_GUIDE.md) | [API_REFERENCE.md](API_REFERENCE.md) |

## 🔍 快速查找

### 配置相关
- **全局设置**: [README.md](README.md#配置详解) → [USAGE_GUIDE.md](USAGE_GUIDE.md#详细配置指南)
- **Provider配置**: [README.md](README.md#配置详解) → [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md#企业级配置)
- **模型映射**: [README.md](README.md#配置详解) → [USAGE_GUIDE.md](USAGE_GUIDE.md#模型映射高级配置)
- **用户管理**: [USAGE_GUIDE.md](USAGE_GUIDE.md#用户认证配置) → [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md#多租户权限管理)

### API相关
- **认证方式**: [API_REFERENCE.md](API_REFERENCE.md#认证)
- **聊天接口**: [API_REFERENCE.md](API_REFERENCE.md#聊天完成接口)
- **模型列表**: [API_REFERENCE.md](API_REFERENCE.md#模型列表接口)
- **健康检查**: [API_REFERENCE.md](API_REFERENCE.md#健康检查接口)
- **错误处理**: [API_REFERENCE.md](API_REFERENCE.md#错误处理)

### 负载均衡策略
- **策略概览**: [README.md](README.md#负载均衡策略详解)
- **策略详解**: [README.md](README.md#负载均衡策略详解)
- **配置示例**: [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md)
- **最佳实践**: [USAGE_GUIDE.md](USAGE_GUIDE.md#最佳实践)

### 故障排除
- **常见问题**: [README.md](README.md#故障排除) → [USAGE_GUIDE.md](USAGE_GUIDE.md#故障排除)
- **调试方法**: [README.md](README.md#测试与调试) → [USAGE_GUIDE.md](USAGE_GUIDE.md#调试技巧)
- **日志分析**: [USAGE_GUIDE.md](USAGE_GUIDE.md#故障排除)

## 🛠️ 开发者资源

### 代码示例
- **[api/examples/debug_logging_demo.rs](api/examples/debug_logging_demo.rs)** - 调试日志演示
- **[api/examples/initial_health_check_demo.rs](api/examples/initial_health_check_demo.rs)** - 健康检查演示

### 测试脚本
- **[test_auth.sh](test_auth.sh)** - 认证功能测试
- **[debug_demo.sh](debug_demo.sh)** - 调试功能演示

### 配置文件
- **[config_example.toml](config_example.toml)** - 完整配置示例
- **[config_simple.toml](config_simple.toml)** - 简单配置示例
- **[test_config.toml](test_config.toml)** - 测试配置

## 📞 获取帮助

### 在线资源
- **GitHub Issues**: [提交问题](https://github.com/PPKunOfficial/berry-api/issues)
- **GitHub Discussions**: [讨论交流](https://github.com/PPKunOfficial/berry-api/discussions)

### 文档反馈
如果您发现文档有任何问题或需要改进的地方，请：
1. 在GitHub上提交Issue
2. 提交Pull Request改进文档
3. 在Discussions中提出建议

### 常见问题快速解答
1. **如何开始使用？** → 查看 [README.md](README.md#快速开始)
2. **如何配置多个Provider？** → 查看 [CONFIGURATION_EXAMPLES.md](CONFIGURATION_EXAMPLES.md#企业级配置)
3. **如何设置用户权限？** → 查看 [USAGE_GUIDE.md](USAGE_GUIDE.md#用户认证配置)
4. **API如何调用？** → 查看 [API_REFERENCE.md](API_REFERENCE.md)
5. **遇到错误怎么办？** → 查看 [USAGE_GUIDE.md](USAGE_GUIDE.md#故障排除)

---

**提示**: 建议按照 README.md → USAGE_GUIDE.md → API_REFERENCE.md → CONFIGURATION_EXAMPLES.md 的顺序阅读文档，这样可以从基础到高级逐步掌握Berry API的使用方法。
