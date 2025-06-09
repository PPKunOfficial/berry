# Berry API 架构设计文档

## 1. 系统概述

Berry API 是一个高性能的AI服务负载均衡网关，提供智能的后端选择、健康检查、故障转移和请求转发功能。系统采用模块化设计，支持多种负载均衡策略和认证机制。

## 2. 核心模块架构

### 2.1 模块组织结构

```
api/src/
├── app.rs                    # 应用入口和状态管理
├── config/                   # 配置管理模块
│   ├── model.rs             # 配置数据结构
│   └── loader.rs            # 配置加载器
├── auth/                     # 认证模块
│   ├── middleware.rs        # 认证中间件
│   └── types.rs             # 认证类型定义
├── loadbalance/             # 负载均衡模块
│   ├── service.rs           # 负载均衡服务
│   ├── manager.rs           # 负载均衡管理器
│   ├── selector.rs          # 后端选择器
│   └── health_checker.rs    # 健康检查器
├── relay/                   # 请求转发模块
│   ├── handler/             # 请求处理器
│   └── client/              # 客户端实现
├── router/                  # 路由模块
│   ├── router.rs            # 路由配置
│   ├── chat.rs              # 聊天API路由
│   ├── health.rs            # 健康检查路由
│   ├── metrics.rs           # 指标路由
│   └── models.rs            # 模型列表路由
└── static_files.rs          # 静态文件服务
```

## 3. 详细时序图

### 3.1 系统启动时序图

```mermaid
sequenceDiagram
    participant Main as main.rs
    participant App as AppState
    participant Config as ConfigLoader
    participant LB as LoadBalanceService
    participant Manager as LoadBalanceManager
    participant Health as HealthChecker
    participant Router as AxumRouter

    Main->>App: AppState::new()
    App->>Config: load_config()
    Config-->>App: Config

    App->>LB: LoadBalanceService::new(config)
    LB->>Manager: LoadBalanceManager::new(config)
    LB->>Health: HealthChecker::new(config, metrics)
    LB-->>App: LoadBalanceService

    App->>LB: start()
    LB->>Manager: initialize()
    Manager->>Manager: 为每个模型创建BackendSelector
    Manager-->>LB: 初始化完成

    LB->>Health: start() (异步任务)
    Health->>Health: 开始定期健康检查循环
    LB-->>App: 服务启动完成

    App->>Router: create_app_router()
    Router-->>App: 配置好的路由

    Main->>Main: 启动HTTP服务器
```

### 3.2 请求处理完整时序图

```mermaid
sequenceDiagram
    participant Client as 客户端
    participant Router as Axum路由
    participant Auth as 认证中间件
    participant Handler as LoadBalancedHandler
    participant LB as LoadBalanceService
    participant Selector as BackendSelector
    participant Metrics as MetricsCollector
    participant RelayClient as OpenAIClient
    participant Provider as AI Provider

    Client->>Router: POST /v1/chat/completions
    Router->>Auth: 认证中间件检查

    Auth->>Auth: 提取Authorization头
    Auth->>Auth: 验证Bearer Token
    Auth->>Auth: 检查用户状态和权限
    alt 认证失败
        Auth-->>Client: 401/403 错误响应
    else 认证成功
        Auth->>Router: 继续处理请求
    end

    Router->>Handler: chat_completions(request)
    Handler->>Handler: 解析请求体
    Handler->>Handler: 提取模型名称
    Handler->>Auth: 检查模型访问权限

    alt 权限检查失败
        Handler-->>Client: 403 权限错误
    else 权限检查通过
        Handler->>LB: select_backend(model_name)

        LB->>Selector: select()
        Selector->>Metrics: 获取后端健康状态
        Selector->>Selector: 根据策略选择后端
        alt 无可用后端
            Selector-->>LB: 选择失败错误
            LB-->>Handler: 后端选择失败
            Handler-->>Client: 503 服务不可用
        else 成功选择后端
            Selector-->>LB: SelectedBackend
            LB-->>Handler: 选中的后端信息
        end

        Handler->>Handler: 构建转发请求
        Handler->>RelayClient: 发送请求到选中后端
        RelayClient->>Provider: HTTP请求

        alt 请求失败
            Provider-->>RelayClient: 错误响应
            RelayClient-->>Handler: 请求失败
            Handler->>Metrics: 记录失败指标
            Handler->>Handler: 重试逻辑
        else 请求成功
            Provider-->>RelayClient: 成功响应
            RelayClient-->>Handler: 响应数据
            Handler->>Metrics: 记录成功指标和延迟
            Handler-->>Client: 转发响应
        end
    end
```

### 3.3 负载均衡选择时序图

```mermaid
sequenceDiagram
    participant Handler as LoadBalancedHandler
    participant LB as LoadBalanceService
    participant Manager as LoadBalanceManager
    participant Selector as BackendSelector
    participant Metrics as MetricsCollector

    Handler->>LB: select_backend("gpt-4")
    LB->>Manager: select_backend("gpt-4")
    Manager->>Manager: 查找模型选择器
    Manager->>Selector: select()

    Selector->>Selector: 获取启用的后端列表
    Selector->>Metrics: 检查后端健康状态
    Metrics-->>Selector: 健康状态信息

    alt 策略: WeightedRandom
        Selector->>Selector: 按权重随机选择
    else 策略: RoundRobin
        Selector->>Selector: 轮询选择
    else 策略: LeastLatency
        Selector->>Metrics: 获取延迟信息
        Metrics-->>Selector: 延迟数据
        Selector->>Selector: 选择延迟最低的后端
    else 策略: Failover
        Selector->>Selector: 按优先级选择第一个健康后端
    else 策略: SmartWeightedFailover
        Selector->>Selector: 智能权重故障转移选择
    end

    Selector-->>Manager: 选中的Backend
    Manager-->>LB: SelectedBackend
    LB-->>Handler: 后端信息
```

### 3.4 健康检查时序图

```mermaid
sequenceDiagram
    participant Timer as 定时器
    participant Health as HealthChecker
    participant Metrics as MetricsCollector
    participant Client as HTTPClient
    participant Provider as AI Provider

    Timer->>Health: 定期触发健康检查
    Health->>Health: check_all_providers()

    loop 遍历所有Provider
        Health->>Health: check_provider_health()
        Health->>Health: 检查计费模式

        alt 按Token计费模型
            Health->>Client: 发送models API请求
            Client->>Provider: GET /v1/models
            Provider-->>Client: 模型列表响应
            Client-->>Health: 响应结果

            alt 响应成功
                Health->>Metrics: record_success()
                Health->>Metrics: record_latency()
                Health->>Metrics: update_health_check()
            else 响应失败
                Health->>Metrics: record_failure()
            end
        else 按请求计费模型
            Health->>Health: 跳过主动检查
            Health->>Metrics: 使用被动验证策略
        end
    end

    Health->>Health: check_recovery()
    loop 遍历不健康后端
        Health->>Health: 检查是否需要恢复验证
        alt 需要恢复检查且为Token计费
            Health->>Client: 发送chat请求
            Client->>Provider: POST /v1/chat/completions
            Provider-->>Client: 聊天响应
            Client-->>Health: 响应结果

            alt 恢复成功
                Health->>Metrics: 标记为健康
            else 恢复失败
                Health->>Metrics: 保持不健康状态
            end
        end
    end
```

### 3.5 配置热重载时序图

```mermaid
sequenceDiagram
    participant Admin as 管理员
    participant Config as ConfigLoader
    participant LB as LoadBalanceService
    participant Manager as LoadBalanceManager
    participant Health as HealthChecker
    participant Selector as BackendSelector

    Admin->>Config: 修改配置文件
    Admin->>LB: reload_config()

    LB->>Config: load_config()
    Config-->>LB: 新配置

    LB->>Manager: reload_config(new_config)
    Manager->>Manager: 停止旧选择器
    Manager->>Manager: 创建新选择器
    loop 遍历新模型配置
        Manager->>Selector: 创建新的BackendSelector
        Selector-->>Manager: 新选择器实例
    end
    Manager-->>LB: 重载完成

    LB->>Health: reload_config(new_config)
    Health->>Health: 更新健康检查配置
    Health->>Health: 重新开始健康检查
    Health-->>LB: 重载完成

    LB-->>Admin: 配置重载成功
```

### 3.6 错误处理和重试时序图

```mermaid
sequenceDiagram
    participant Handler as LoadBalancedHandler
    participant LB as LoadBalanceService
    participant Client as OpenAIClient
    participant Provider as AI Provider
    participant Metrics as MetricsCollector

    Handler->>LB: select_backend()
    LB-->>Handler: SelectedBackend

    loop 重试循环 (max_retries次)
        Handler->>Client: 发送请求
        Client->>Provider: HTTP请求

        alt 网络错误
            Provider-->>Client: 连接超时/网络错误
            Client-->>Handler: 请求失败
            Handler->>Metrics: record_failure()
            Handler->>Handler: 准备重试
        else HTTP错误
            Provider-->>Client: 4xx/5xx错误
            Client-->>Handler: HTTP错误响应
            Handler->>Metrics: record_failure()
            Handler->>Handler: 准备重试
        else 成功响应
            Provider-->>Client: 200 OK
            Client-->>Handler: 成功响应
            Handler->>Metrics: record_success()
            Handler->>Metrics: record_latency()
        end

        alt 未达到最大重试次数
            Handler->>LB: select_backend() (重新选择)
            LB-->>Handler: 新的SelectedBackend
        else 达到最大重试次数
            Handler-->>Handler: 返回最终错误
        end
    end
```

## 4. 模块职责详解

### 4.1 应用层 (app.rs)
- **AppState**: 管理全局应用状态，包括负载均衡服务、处理器和配置
- **生命周期管理**: 负责服务的启动和优雅关闭
- **依赖注入**: 为各个组件提供共享的状态和配置

### 4.2 配置模块 (config/)
- **Config结构**: 定义完整的配置数据结构
- **Provider配置**: AI服务提供商的连接信息和认证
- **Model映射**: 自定义模型到Provider模型的映射关系
- **用户管理**: 用户认证和权限配置
- **全局设置**: 超时、重试、健康检查等全局参数

### 4.3 认证模块 (auth/)
- **Bearer Token认证**: 基于HTTP Authorization头的认证
- **用户权限验证**: 检查用户对特定模型的访问权限
- **中间件集成**: 与Axum框架的中间件系统集成
- **错误处理**: 标准化的认证错误响应

### 4.4 负载均衡模块 (loadbalance/)
- **LoadBalanceService**: 负载均衡的主服务接口
- **LoadBalanceManager**: 管理所有模型的选择器
- **BackendSelector**: 实现具体的负载均衡策略
- **HealthChecker**: 定期检查后端健康状态
- **MetricsCollector**: 收集性能指标和健康状态

### 4.5 转发模块 (relay/)
- **LoadBalancedHandler**: 负载均衡的请求处理器
- **OpenAIClient**: OpenAI兼容的HTTP客户端
- **请求转换**: 处理请求格式转换和模型名称映射
- **响应处理**: 支持流式和非流式响应

### 4.6 路由模块 (router/)
- **路由配置**: 定义所有API端点
- **请求分发**: 将请求分发到相应的处理器
- **中间件集成**: 集成认证、日志等中间件
- **静态文件服务**: 提供监控界面的静态文件

## 5. 关键设计特性

### 5.1 负载均衡策略
- **WeightedRandom**: 基于权重的随机选择
- **RoundRobin**: 轮询选择
- **LeastLatency**: 选择延迟最低的后端
- **Failover**: 优先级故障转移
- **SmartWeightedFailover**: 智能权重故障转移

### 5.2 健康检查机制
- **主动检查**: 定期发送API请求验证后端状态
- **被动验证**: 基于实际请求结果更新健康状态
- **计费模式感知**: 根据计费模式选择检查策略
- **恢复验证**: 不健康后端的恢复检查

### 5.3 错误处理
- **多层重试**: 请求级别和后端级别的重试机制
- **故障隔离**: 快速识别和隔离故障后端
- **优雅降级**: 在部分后端不可用时继续服务
- **详细错误信息**: 提供有用的错误诊断信息

### 5.4 性能优化
- **异步架构**: 全异步处理提高并发性能
- **连接复用**: HTTP客户端连接池
- **指标收集**: 实时性能监控
- **内存效率**: 优化的数据结构和算法

## 6. 高级时序图

### 6.1 流式响应处理时序图

```mermaid
sequenceDiagram
    participant Client as 客户端
    participant Handler as LoadBalancedHandler
    participant RelayClient as OpenAIClient
    participant Provider as AI Provider

    Client->>Handler: POST /v1/chat/completions (stream=true)
    Handler->>Handler: 检测到流式请求
    Handler->>RelayClient: 创建流式请求
    RelayClient->>Provider: 建立SSE连接

    Provider-->>RelayClient: HTTP 200 + Transfer-Encoding: chunked
    RelayClient-->>Handler: 开始流式响应
    Handler-->>Client: 转发响应头

    loop 流式数据传输
        Provider-->>RelayClient: data: {"choices":[...]}
        RelayClient-->>Handler: 流式数据块
        Handler-->>Client: 转发数据块
    end

    Provider-->>RelayClient: data: [DONE]
    RelayClient-->>Handler: 流结束标记
    Handler-->>Client: 结束流式响应

    Handler->>Handler: 记录请求指标
```

### 6.2 权重恢复机制时序图

```mermaid
sequenceDiagram
    participant Metrics as MetricsCollector
    participant Selector as BackendSelector
    participant Backend as 不健康后端

    Note over Backend: 后端标记为不健康 (权重=0)

    Selector->>Backend: 尝试请求 (10%权重)
    Backend-->>Selector: 请求成功
    Selector->>Metrics: record_success()
    Metrics->>Metrics: 更新权重恢复状态 (30%)

    Selector->>Backend: 下次请求 (30%权重)
    Backend-->>Selector: 请求成功
    Selector->>Metrics: record_success()
    Metrics->>Metrics: 更新权重恢复状态 (50%)

    Selector->>Backend: 下次请求 (50%权重)
    Backend-->>Selector: 请求成功
    Selector->>Metrics: record_success()
    Metrics->>Metrics: 完全恢复 (100%权重)

    Note over Backend: 后端完全恢复健康状态
```

### 6.3 并发请求处理时序图

```mermaid
sequenceDiagram
    participant C1 as 客户端1
    participant C2 as 客户端2
    participant C3 as 客户端3
    participant Router as 路由器
    participant Handler as 处理器
    participant LB as 负载均衡器

    par 并发请求处理
        C1->>Router: 请求1
        Router->>Handler: 异步处理1
        Handler->>LB: 选择后端1
        LB-->>Handler: 后端A

    and
        C2->>Router: 请求2
        Router->>Handler: 异步处理2
        Handler->>LB: 选择后端2
        LB-->>Handler: 后端B

    and
        C3->>Router: 请求3
        Router->>Handler: 异步处理3
        Handler->>LB: 选择后端3
        LB-->>Handler: 后端A
    end

    par 并发响应
        Handler-->>C1: 响应1
    and
        Handler-->>C2: 响应2
    and
        Handler-->>C3: 响应3
    end
```

### 6.4 监控和指标收集时序图

```mermaid
sequenceDiagram
    participant Client as 监控客户端
    participant Router as 路由器
    participant Metrics as 指标处理器
    participant LB as 负载均衡服务
    participant Collector as MetricsCollector

    Client->>Router: GET /metrics
    Router->>Metrics: metrics()
    Metrics->>LB: get_service_health()

    LB->>Collector: 收集健康统计
    Collector->>Collector: 计算成功率
    Collector->>Collector: 统计后端状态
    Collector->>Collector: 收集延迟信息
    Collector-->>LB: 健康摘要

    LB-->>Metrics: ServiceHealth
    Metrics->>Metrics: 构建指标响应
    Metrics-->>Client: JSON指标数据
```

## 7. 数据流图

### 7.1 配置数据流

```mermaid
flowchart TD
    A[config.toml] --> B[ConfigLoader]
    B --> C[Config结构]
    C --> D[Provider配置]
    C --> E[Model映射]
    C --> F[User配置]
    C --> G[全局设置]

    D --> H[LoadBalanceService]
    E --> I[LoadBalanceManager]
    F --> J[AuthMiddleware]
    G --> K[HealthChecker]

    H --> L[后端选择]
    I --> L
    J --> M[请求认证]
    K --> N[健康监控]
```

### 7.2 请求数据流

```mermaid
flowchart TD
    A[客户端请求] --> B[Axum路由]
    B --> C[认证中间件]
    C --> D{认证成功?}
    D -->|否| E[401/403错误]
    D -->|是| F[LoadBalancedHandler]

    F --> G[解析请求]
    G --> H[模型权限检查]
    H --> I{权限检查}
    I -->|失败| J[403错误]
    I -->|通过| K[负载均衡选择]

    K --> L[BackendSelector]
    L --> M[MetricsCollector]
    M --> N{有健康后端?}
    N -->|否| O[503错误]
    N -->|是| P[选择后端]

    P --> Q[构建转发请求]
    Q --> R[OpenAIClient]
    R --> S[发送到Provider]
    S --> T{请求成功?}
    T -->|否| U[重试逻辑]
    T -->|是| V[返回响应]

    U --> K
    V --> W[记录指标]
    W --> X[返回客户端]
```

## 8. 组件交互图

### 8.1 核心组件关系图

```mermaid
graph TB
    subgraph "应用层"
        A[AppState]
        B[main.rs]
    end

    subgraph "路由层"
        C[Router]
        D[AuthMiddleware]
        E[ChatHandler]
        F[HealthHandler]
        G[MetricsHandler]
    end

    subgraph "业务层"
        H[LoadBalancedHandler]
        I[LoadBalanceService]
        J[LoadBalanceManager]
        K[BackendSelector]
    end

    subgraph "基础设施层"
        L[HealthChecker]
        M[MetricsCollector]
        N[OpenAIClient]
        O[ConfigLoader]
    end

    B --> A
    A --> C
    A --> I
    A --> H

    C --> D
    C --> E
    C --> F
    C --> G

    D --> O
    E --> H
    F --> I
    G --> I

    H --> I
    I --> J
    I --> L
    J --> K
    K --> M

    H --> N
    L --> M
    L --> N
```

### 8.2 数据存储和状态管理

```mermaid
graph LR
    subgraph "配置状态"
        A[Config]
        B[Providers]
        C[Models]
        D[Users]
    end

    subgraph "运行时状态"
        E[HealthStatus]
        F[Metrics]
        G[LoadBalanceState]
        H[ConnectionPools]
    end

    subgraph "持久化"
        I[config.toml]
        J[日志文件]
    end

    I --> A
    A --> B
    A --> C
    A --> D

    B --> E
    C --> G
    D --> F

    E --> F
    F --> G
    G --> H

    F --> J
    E --> J
```

## 9. 安全架构

### 9.1 认证授权流程

```mermaid
sequenceDiagram
    participant Client as 客户端
    participant Auth as 认证中间件
    participant Config as 配置
    participant Handler as 处理器

    Client->>Auth: 请求 + Bearer Token
    Auth->>Auth: 提取Token
    Auth->>Config: 验证Token
    Config-->>Auth: 用户信息

    Auth->>Auth: 检查用户状态
    alt 用户被禁用
        Auth-->>Client: 403 Forbidden
    else 用户正常
        Auth->>Handler: 转发请求 + 用户信息
        Handler->>Config: 检查模型权限
        Config-->>Handler: 权限结果

        alt 无权限
            Handler-->>Client: 403 Model Access Denied
        else 有权限
            Handler->>Handler: 处理请求
            Handler-->>Client: 正常响应
        end
    end
```

### 9.2 API密钥管理

```mermaid
graph TD
    A[config.toml] --> B[Provider配置]
    B --> C[API密钥]
    C --> D{密钥验证}
    D -->|有效| E[存储在内存]
    D -->|无效| F[标记Provider不健康]

    E --> G[请求转发时使用]
    F --> H[跳过该Provider]

    G --> I[HTTPS传输]
    I --> J[目标API服务]
```

## 10. 性能和可扩展性

### 10.1 性能优化策略

- **异步I/O**: 使用Tokio异步运行时，支持高并发
- **连接池**: HTTP客户端连接复用，减少连接开销
- **内存管理**: 使用Arc和RwLock实现高效的共享状态
- **负载均衡**: 智能后端选择，避免热点问题
- **缓存策略**: 配置和健康状态的内存缓存

### 10.2 可扩展性设计

- **模块化架构**: 各组件松耦合，易于扩展和维护
- **插件化策略**: 负载均衡策略可插拔
- **配置驱动**: 通过配置文件控制行为，无需代码修改
- **水平扩展**: 支持多实例部署和负载分担

### 10.3 监控和观测性

- **结构化日志**: 使用tracing框架提供详细日志
- **指标收集**: 实时性能和健康指标
- **健康检查**: 多层次的健康状态监控
- **错误追踪**: 详细的错误信息和调用链