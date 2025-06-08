#!/bin/bash

# Debug日志演示脚本
# 这个脚本展示如何启用debug日志来观察健康检查系统的详细工作过程

echo "=== Berry API 健康检查 Debug 日志演示 ==="
echo ""

echo "1. 运行健康检查相关测试，启用debug日志："
echo "   RUST_LOG=debug cargo test health_check --test health_check_integration_test"
echo ""

echo "2. 运行debug日志测试，查看详细输出："
echo "   RUST_LOG=debug cargo test debug_logging --test debug_logging_test"
echo ""

echo "3. 运行所有负载均衡测试，启用debug日志："
echo "   RUST_LOG=debug cargo test loadbalance"
echo ""

echo "4. 启动API服务器并观察实时健康检查日志："
echo "   RUST_LOG=debug cargo run"
echo ""

echo "=== Debug日志级别说明 ==="
echo ""
echo "RUST_LOG环境变量可以设置不同的日志级别："
echo "- RUST_LOG=error    : 只显示错误信息"
echo "- RUST_LOG=warn     : 显示警告和错误信息"
echo "- RUST_LOG=info     : 显示信息、警告和错误"
echo "- RUST_LOG=debug    : 显示所有调试信息（推荐用于观察健康检查）"
echo "- RUST_LOG=trace    : 显示最详细的跟踪信息"
echo ""

echo "=== 健康检查Debug日志示例 ==="
echo ""
echo "当启用debug日志时，您将看到类似以下的输出："
echo ""
echo "DEBUG Starting health check for 2 enabled providers"
echo "DEBUG Scheduling health check for provider: test-provider (Test Provider)"
echo "DEBUG Starting health check task for provider: test-provider"
echo "DEBUG Starting health check for provider: test-provider (base_url: https://httpbin.org)"
echo "DEBUG API key present for provider test-provider, proceeding with health check"
echo "DEBUG Detected test provider (httpbin), using HTTP status check for test-provider"
echo "DEBUG Testing provider test-provider with URL: https://httpbin.org/status/200"
echo "DEBUG Sending HTTP request to test provider test-provider"
echo "DEBUG Received response from provider test-provider with status: 200 OK (245ms)"
echo "DEBUG Provider test-provider health check passed, marking 1 models as healthy"
echo "DEBUG Marking backend test-provider:test-model as healthy (latency: 245ms)"
echo "DEBUG Completed health check for provider test-provider in 246ms"
echo ""

echo "=== 恢复检查Debug日志示例 ==="
echo ""
echo "DEBUG Starting recovery check process (interval: 120s)"
echo "DEBUG Unhealthy backends: [\"failing-provider:failing-model\"]"
echo "DEBUG Evaluating recovery check for backend: failing-provider:failing-model"
echo "DEBUG Backend failing-provider:failing-model needs recovery check"
echo "DEBUG Parsed backend key: provider=failing-provider, model=failing-model"
echo "DEBUG Recording recovery attempt for backend: failing-provider:failing-model"
echo "DEBUG Starting chat-based recovery check for failing-provider:failing-model"
echo "DEBUG Created OpenAI client for recovery check (base_url: https://api.failing.com)"
echo "DEBUG Built test chat request for recovery check: {\"model\":\"failing-model\",...}"
echo "DEBUG Sending chat request for recovery check to failing-provider:failing-model"
echo "DEBUG Network/API error during recovery check: connection failed"
echo "DEBUG Completed recovery check for failing-provider:failing-model in 5002ms"
echo ""

echo "=== 智能重试Debug日志示例 ==="
echo ""
echo "DEBUG Backend selection attempt 1 for model 'test-model'"
echo "DEBUG Load balancer selected backend: failing-provider:failing-model"
echo "DEBUG Health check for failing-provider:failing-model: UNHEALTHY"
echo "DEBUG Selected backend failing-provider:failing-model is unhealthy, retrying... (attempt 1/3)"
echo "DEBUG Backend selection attempt 2 for model 'test-model'"
echo "DEBUG Load balancer selected backend: test-provider:test-model"
echo "DEBUG Health check for test-provider:test-model: HEALTHY"
echo "DEBUG Selected healthy backend for model 'test-model': provider='test-provider', model='test-model', selection_time=2ms"
echo ""

echo "=== 运行演示 ==="
echo ""
read -p "是否运行健康检查测试并显示debug日志？(y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "运行健康检查测试..."
    RUST_LOG=debug cargo test health_check_basic_functionality --test health_check_integration_test
fi

echo ""
read -p "是否运行指标debug日志测试？(y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "运行指标debug日志测试..."
    RUST_LOG=debug cargo test metrics_debug_logging --test debug_logging_test
fi

echo ""
echo "演示完成！"
echo ""
echo "提示：在生产环境中，建议使用 RUST_LOG=info 或 RUST_LOG=warn"
echo "      只在调试问题时才使用 RUST_LOG=debug"
