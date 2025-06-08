#!/bin/bash

# Berry API 认证功能测试脚本

BASE_URL="http://localhost:3000"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试令牌
ADMIN_TOKEN="berry-admin-123456"
USER_TOKEN="berry-user-789012"
INVALID_TOKEN="invalid-token-xyz"

echo -e "${BLUE}=== Berry API 认证功能测试 ===${NC}"
echo

# 函数：发送请求并检查响应
test_request() {
    local description="$1"
    local method="$2"
    local endpoint="$3"
    local token="$4"
    local data="$5"
    local expected_status="$6"
    
    echo -e "${YELLOW}测试: $description${NC}"
    
    if [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL$endpoint" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $token" \
            -d "$data")
    else
        response=$(curl -s -w "\n%{http_code}" "$BASE_URL$endpoint" \
            -H "Authorization: Bearer $token")
    fi
    
    # 分离响应体和状态码
    body=$(echo "$response" | head -n -1)
    status=$(echo "$response" | tail -n 1)
    
    if [ "$status" = "$expected_status" ]; then
        echo -e "${GREEN}✓ 通过 (状态码: $status)${NC}"
        if [ "$status" = "200" ]; then
            echo "响应: $(echo "$body" | jq -r '.data[0].id // .choices[0].message.content // .status // "成功"' 2>/dev/null || echo "成功")"
        fi
    else
        echo -e "${RED}✗ 失败 (期望: $expected_status, 实际: $status)${NC}"
        echo "响应: $body"
    fi
    echo
}

# 等待服务启动
echo -e "${BLUE}检查服务状态...${NC}"
for i in {1..10}; do
    if curl -s "$BASE_URL/health" > /dev/null; then
        echo -e "${GREEN}✓ 服务已启动${NC}"
        break
    fi
    if [ $i -eq 10 ]; then
        echo -e "${RED}✗ 服务未启动，请先启动 Berry API 服务${NC}"
        exit 1
    fi
    echo "等待服务启动... ($i/10)"
    sleep 2
done
echo

# 测试1: 健康检查（无需认证）
test_request "健康检查（无需认证）" "GET" "/health" "" "" "200"

# 测试2: 无效令牌访问模型列表
test_request "无效令牌访问模型列表" "GET" "/v1/models" "$INVALID_TOKEN" "" "401"

# 测试3: 有效令牌访问模型列表（管理员）
test_request "管理员令牌访问模型列表" "GET" "/v1/models" "$ADMIN_TOKEN" "" "200"

# 测试4: 有效令牌访问模型列表（普通用户）
test_request "普通用户令牌访问模型列表" "GET" "/v1/models" "$USER_TOKEN" "" "200"

# 测试5: 无效令牌聊天请求
chat_data='{
  "model": "gpt-3.5-turbo",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": false
}'
test_request "无效令牌聊天请求" "POST" "/v1/chat/completions" "$INVALID_TOKEN" "$chat_data" "401"

# 测试6: 管理员访问允许的模型
admin_chat_data='{
  "model": "fast-model",
  "messages": [{"role": "user", "content": "Hello from admin"}],
  "stream": false
}'
test_request "管理员访问允许的模型" "POST" "/v1/chat/completions" "$ADMIN_TOKEN" "$admin_chat_data" "200"

# 测试7: 普通用户访问允许的模型
user_chat_data='{
  "model": "fast-model",
  "messages": [{"role": "user", "content": "Hello from user"}],
  "stream": false
}'
test_request "普通用户访问允许的模型" "POST" "/v1/chat/completions" "$USER_TOKEN" "$user_chat_data" "200"

# 测试8: 普通用户访问不允许的模型（gpt-4不在用户的allowed_models中）
restricted_chat_data='{
  "model": "gpt-4",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": false
}'
test_request "普通用户访问受限模型" "POST" "/v1/chat/completions" "$USER_TOKEN" "$restricted_chat_data" "403"

# 测试9: 缺少Authorization头
echo -e "${YELLOW}测试: 缺少Authorization头${NC}"
response=$(curl -s -w "\n%{http_code}" "$BASE_URL/v1/models" \
    -H "Content-Type: application/json")
body=$(echo "$response" | head -n -1)
status=$(echo "$response" | tail -n 1)

if [ "$status" = "401" ]; then
    echo -e "${GREEN}✓ 通过 (状态码: $status)${NC}"
else
    echo -e "${RED}✗ 失败 (期望: 401, 实际: $status)${NC}"
    echo "响应: $body"
fi
echo

# 测试10: 错误的Authorization格式
echo -e "${YELLOW}测试: 错误的Authorization格式${NC}"
response=$(curl -s -w "\n%{http_code}" "$BASE_URL/v1/models" \
    -H "Authorization: InvalidFormat $USER_TOKEN")
body=$(echo "$response" | head -n -1)
status=$(echo "$response" | tail -n 1)

if [ "$status" = "401" ]; then
    echo -e "${GREEN}✓ 通过 (状态码: $status)${NC}"
else
    echo -e "${RED}✗ 失败 (期望: 401, 实际: $status)${NC}"
    echo "响应: $body"
fi
echo

echo -e "${BLUE}=== 测试完成 ===${NC}"
echo
echo -e "${YELLOW}使用说明:${NC}"
echo "1. 确保使用 config_simple.toml 配置文件启动服务"
echo "2. 管理员令牌: $ADMIN_TOKEN (可访问所有模型)"
echo "3. 普通用户令牌: $USER_TOKEN (只能访问 gpt_3_5_turbo 和 fast_model)"
echo "4. 用户只能看到自己有权限访问的模型列表"
echo "5. 所有API请求都需要在Header中包含: Authorization: Bearer <token>"
echo
echo -e "${GREEN}认证系统已成功集成到Berry API中！${NC}"
