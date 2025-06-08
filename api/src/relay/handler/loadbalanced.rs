use axum::response::Sse;
use axum::response::sse::Event;
use axum::{extract::Json, response::IntoResponse};
use axum_extra::TypedHeader;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Instant;

use crate::loadbalance::{LoadBalanceService, RequestResult};
use crate::relay::client::openai::OpenAIClient;

use super::types::create_error_json;

/// 负载均衡的OpenAI兼容处理器
pub struct LoadBalancedHandler {
    load_balancer: std::sync::Arc<LoadBalanceService>,
}

impl LoadBalancedHandler {
    pub fn new(load_balancer: std::sync::Arc<LoadBalanceService>) -> Self {
        Self { load_balancer }
    }

    /// 处理聊天完成请求（支持负载均衡和智能重试）
    pub async fn handle_completions(
        self: Arc<Self>,
        TypedHeader(authorization): TypedHeader<
            headers::Authorization<headers::authorization::Bearer>,
        >,
        TypedHeader(content_type): TypedHeader<headers::ContentType>,
        Json(mut body): Json<Value>,
    ) -> axum::response::Response {
        let start_time = Instant::now();

        // 从请求体中提取模型名称
        let model_name = match body.get("model").and_then(|m| m.as_str()) {
            Some(name) => name.to_string(),
            None => {
                tracing::error!("Missing model field in request");
                return Json(create_error_json(
                    &crate::relay::client::ClientError::HeaderParseError(
                        "Missing model field in request".to_string(),
                    ),
                ))
                .into_response();
            }
        };

        // 尝试处理请求，带内部重试机制
        match self
            .try_handle_with_retries(
                &model_name,
                &mut body,
                &authorization,
                &content_type,
                start_time,
            )
            .await
        {
            Ok(response) => response,
            Err(e) => {
                tracing::error!(
                    "All retry attempts failed for model '{}': {}",
                    model_name,
                    e
                );
                Json(create_error_json(
                    &crate::relay::client::ClientError::HeaderParseError(format!(
                        "Request failed after all retries: {}",
                        e
                    )),
                ))
                .into_response()
            }
        }
    }

    /// 尝试处理请求，带重试机制
    async fn try_handle_with_retries(
        &self,
        model_name: &str,
        body: &mut Value,
        authorization: &headers::Authorization<headers::authorization::Bearer>,
        content_type: &headers::ContentType,
        start_time: Instant,
    ) -> Result<axum::response::Response, anyhow::Error> {
        let max_retries = 3; // 可以从配置中读取
        let original_model = model_name.to_string();

        for attempt in 0..max_retries {
            // 重置模型名称为原始请求的模型名称
            body["model"] = Value::String(original_model.clone());

            // 使用负载均衡器选择后端
            let selected_backend = match self.load_balancer.select_backend(model_name).await {
                Ok(backend) => backend,
                Err(e) => {
                    if attempt == max_retries - 1 {
                        return Err(anyhow::anyhow!("Failed to select backend: {}", e));
                    }
                    tracing::warn!(
                        "Backend selection failed on attempt {}, retrying: {}",
                        attempt + 1,
                        e
                    );
                    continue;
                }
            };

            tracing::info!(
                "Selected backend for model '{}' (attempt {}): provider='{}', model='{}', selection_time={}ms",
                model_name,
                attempt + 1,
                selected_backend.backend.provider,
                selected_backend.backend.model,
                selected_backend.selection_time.as_millis()
            );

            // 更新请求体中的模型名称为后端的真实模型名称
            body["model"] = Value::String(selected_backend.backend.model.clone());

            // 获取API密钥
            let api_key = match selected_backend.get_api_key() {
                Ok(key) => key,
                Err(e) => {
                    self.load_balancer
                        .record_request_result(
                            &selected_backend.backend.provider,
                            &selected_backend.backend.model,
                            RequestResult::Failure {
                                error: e.to_string(),
                            },
                        )
                        .await;

                    if attempt == max_retries - 1 {
                        return Err(anyhow::anyhow!("API key not found: {}", e));
                    }
                    tracing::warn!("API key error on attempt {}, retrying: {}", attempt + 1, e);
                    continue;
                }
            };

            // 创建客户端
            let client = OpenAIClient::with_base_url(selected_backend.provider.base_url.clone());

            // 构建请求头
            let headers = match client.build_request_headers(&authorization, &content_type) {
                Ok(mut h) => {
                    // 使用选中后端的API密钥
                    h.insert(
                        "Authorization",
                        format!("Bearer {}", api_key).parse().unwrap(),
                    );

                    // 添加自定义头部
                    for (key, value) in selected_backend.get_headers() {
                        if let (Ok(header_name), Ok(header_value)) = (
                            key.parse::<reqwest::header::HeaderName>(),
                            value.parse::<reqwest::header::HeaderValue>(),
                        ) {
                            h.insert(header_name, header_value);
                        }
                    }
                    h
                }
                Err(e) => {
                    self.load_balancer
                        .record_request_result(
                            &selected_backend.backend.provider,
                            &selected_backend.backend.model,
                            RequestResult::Failure {
                                error: e.to_string(),
                            },
                        )
                        .await;

                    if attempt == max_retries - 1 {
                        return Err(anyhow::anyhow!("Failed to build request headers: {}", e));
                    }
                    tracing::warn!(
                        "Header build error on attempt {}, retrying: {}",
                        attempt + 1,
                        e
                    );
                    continue;
                }
            };

            // 尝试发送请求
            match self
                .try_single_request(&client, headers, body, &selected_backend, start_time)
                .await
            {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // 记录失败
                    self.load_balancer
                        .record_request_result(
                            &selected_backend.backend.provider,
                            &selected_backend.backend.model,
                            RequestResult::Failure {
                                error: e.to_string(),
                            },
                        )
                        .await;

                    if attempt == max_retries - 1 {
                        return Err(anyhow::anyhow!("Request failed after all retries: {}", e));
                    }
                    tracing::warn!("Request failed on attempt {}, retrying: {}", attempt + 1, e);
                    continue;
                }
            }
        }

        Err(anyhow::anyhow!("Unexpected end of retry loop"))
    }

    /// 尝试单次请求
    async fn try_single_request(
        &self,
        client: &OpenAIClient,
        headers: reqwest::header::HeaderMap,
        body: &Value,
        selected_backend: &crate::loadbalance::SelectedBackend,
        start_time: Instant,
    ) -> Result<axum::response::Response, anyhow::Error> {
        // 检查是否为流式请求
        let is_stream = body
            .get("stream")
            .unwrap_or(&Value::Bool(false))
            .as_bool()
            .unwrap_or(false);

        if is_stream {
            // 流式请求：尝试发送请求，失败时返回错误以触发重试
            match self
                .try_streaming_request(
                    client.clone(),
                    headers,
                    body.clone(),
                    selected_backend.clone(),
                    start_time,
                )
                .await
            {
                Ok(response) => Ok(response.into_response()),
                Err(e) => Err(anyhow::anyhow!("Streaming request failed: {}", e)),
            }
        } else {
            // 非流式请求：尝试发送请求，失败时返回错误以触发重试
            match self
                .try_non_streaming_request(
                    client.clone(),
                    headers,
                    body.clone(),
                    selected_backend.clone(),
                    start_time,
                )
                .await
            {
                Ok(response) => Ok(response.into_response()),
                Err(e) => Err(anyhow::anyhow!("Non-streaming request failed: {}", e)),
            }
        }
    }

    /// 尝试流式请求（可能失败以触发重试）
    async fn try_streaming_request(
        &self,
        client: OpenAIClient,
        headers: reqwest::header::HeaderMap,
        body: Value,
        selected_backend: crate::loadbalance::SelectedBackend,
        start_time: Instant,
    ) -> Result<
        Sse<futures::stream::BoxStream<'static, Result<Event, std::convert::Infallible>>>,
        anyhow::Error,
    > {
        let provider = &selected_backend.backend.provider;
        let model = &selected_backend.backend.model;

        // 发送API请求
        let response = match client.chat_completions(headers, &body).await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::debug!("Streaming request failed: {:?}", e);
                // 记录失败但不在这里处理，让重试机制处理
                self.load_balancer
                    .record_request_result(
                        provider,
                        model,
                        RequestResult::Failure {
                            error: e.to_string(),
                        },
                    )
                    .await;
                return Err(anyhow::anyhow!("API request failed: {}", e));
            }
        };

        // 检查HTTP状态
        if !response.status().is_success() {
            let status = response.status();
            tracing::debug!("Streaming request failed with status: {}", status);
            // 记录失败但不在这里处理，让重试机制处理
            self.load_balancer
                .record_request_result(
                    provider,
                    model,
                    RequestResult::Failure {
                        error: format!("HTTP {}", status),
                    },
                )
                .await;
            return Err(anyhow::anyhow!("HTTP error: {}", status));
        }

        // 成功情况 - 创建流式响应
        Ok(self
            .create_successful_stream(response, selected_backend, start_time)
            .await)
    }

    /// 创建成功的流式响应
    async fn create_successful_stream(
        &self,
        response: reqwest::Response,
        selected_backend: crate::loadbalance::SelectedBackend,
        start_time: Instant,
    ) -> Sse<futures::stream::BoxStream<'static, Result<Event, std::convert::Infallible>>> {
        let load_balancer = self.load_balancer.clone();
        let provider = selected_backend.backend.provider.clone();
        let model = selected_backend.backend.model.clone();
        let latency = start_time.elapsed();

        // 检查backend是否在不健康列表中
        let backend_key = format!("{}:{}", provider, model);
        let metrics = load_balancer.get_metrics();

        if metrics.is_in_unhealthy_list(&backend_key) {
            // 不健康的backend请求成功，主动恢复为健康状态
            tracing::info!(
                "Unhealthy backend {} streaming request succeeded, automatically marking as healthy",
                backend_key
            );
        }

        // 无论之前是否健康，都记录成功（实现自动恢复）
        let lb_clone = load_balancer.clone();
        let provider_clone = provider.clone();
        let model_clone = model.clone();
        tokio::spawn(async move {
            lb_clone
                .record_request_result(
                    &provider_clone,
                    &model_clone,
                    RequestResult::Success { latency },
                )
                .await;
        });

        // 创建成功的流式响应
        let stream = response
            .bytes_stream()
            .eventsource()
            .map(|result| match result {
                Ok(event) => {
                    tracing::debug!("SSE event: {:?}", event.data);
                    Ok(Event::default().data(event.data))
                }
                Err(err) => {
                    tracing::error!("SSE error: {:?}", err);
                    Ok(Event::default().data(json!({"error": err.to_string()}).to_string()))
                }
            })
            .boxed();

        Sse::new(stream)
    }

    /// 尝试非流式请求（可能失败以触发重试）
    async fn try_non_streaming_request(
        &self,
        client: OpenAIClient,
        headers: reqwest::header::HeaderMap,
        body: Value,
        selected_backend: crate::loadbalance::SelectedBackend,
        start_time: Instant,
    ) -> Result<Json<Value>, anyhow::Error> {
        let provider = &selected_backend.backend.provider;
        let model = &selected_backend.backend.model;

        // 发送API请求
        let response = match client.chat_completions(headers, &body).await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::debug!("Non-streaming request failed: {:?}", e);
                // 记录失败但不在这里处理，让重试机制处理
                self.load_balancer
                    .record_request_result(
                        provider,
                        model,
                        RequestResult::Failure {
                            error: e.to_string(),
                        },
                    )
                    .await;
                return Err(anyhow::anyhow!("API request failed: {}", e));
            }
        };

        let latency = start_time.elapsed();

        // 处理响应
        if response.status().is_success() {
            // 检查backend是否在不健康列表中
            let backend_key = format!("{}:{}", provider, model);
            let metrics = self.load_balancer.get_metrics();

            if metrics.is_in_unhealthy_list(&backend_key) {
                // 不健康的backend请求成功，主动恢复为健康状态
                tracing::info!(
                    "Unhealthy backend {} request succeeded, automatically marking as healthy",
                    backend_key
                );
            }

            // 无论之前是否健康，都记录成功（实现自动恢复）
            self.load_balancer
                .record_request_result(provider, model, RequestResult::Success { latency })
                .await;

            match response.text().await {
                Ok(text) => match serde_json::from_str::<Value>(&text) {
                    Ok(value) => Ok(Json(value)),
                    Err(e) => {
                        tracing::error!("JSON parsing failed: {:?}", e);
                        Err(anyhow::anyhow!("JSON parsing failed: {}", e))
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to read response body: {:?}", e);
                    Err(anyhow::anyhow!("Failed to read response body: {}", e))
                }
            }
        } else {
            // 记录失败
            let status = response.status().as_u16();
            self.load_balancer
                .record_request_result(
                    provider,
                    model,
                    RequestResult::Failure {
                        error: format!("HTTP {}", status),
                    },
                )
                .await;

            tracing::debug!("Non-streaming request failed with status: {}", status);
            Err(anyhow::anyhow!("HTTP error: {}", status))
        }
    }

    #[allow(dead_code)]
    /// 处理流式请求（兜底方法，当重试失败时使用）
    async fn handle_streaming_request(
        &self,
        client: OpenAIClient,
        headers: reqwest::header::HeaderMap,
        body: Value,
        selected_backend: crate::loadbalance::SelectedBackend,
        start_time: Instant,
    ) -> Sse<futures::stream::BoxStream<'static, Result<Event, std::convert::Infallible>>> {
        // 尝试请求，如果失败则返回错误流
        match self
            .try_streaming_request(client, headers, body, selected_backend, start_time)
            .await
        {
            Ok(sse) => sse,
            Err(e) => {
                // 创建错误流
                let error_stream = futures::stream::once(async move {
                    Ok(Event::default().data(
                        json!({
                            "error": {
                                "message": "All retry attempts failed",
                                "details": e.to_string()
                            }
                        })
                        .to_string(),
                    ))
                })
                .boxed();

                Sse::new(error_stream)
            }
        }
    }

    #[allow(dead_code)]
    /// 处理非流式请求（兜底方法，当重试失败时使用）
    async fn handle_non_streaming_request(
        &self,
        client: OpenAIClient,
        headers: reqwest::header::HeaderMap,
        body: Value,
        selected_backend: crate::loadbalance::SelectedBackend,
        start_time: Instant,
    ) -> Json<Value> {
        // 尝试请求，如果失败则返回错误响应
        match self
            .try_non_streaming_request(client, headers, body, selected_backend, start_time)
            .await
        {
            Ok(response) => response,
            Err(e) => Json(json!({
                "error": {
                    "message": "All retry attempts failed",
                    "details": e.to_string()
                }
            })),
        }
    }

    /// 获取可用模型列表（根据用户权限过滤）
    pub async fn handle_models_for_user(&self, user_models: Vec<String>) -> Json<Value> {
        let model_list: Vec<Value> = user_models
            .into_iter()
            .map(|model_name| {
                json!({
                    "id": model_name,
                    "object": "model",
                    "created": chrono::Utc::now().timestamp(),
                    "owned_by": "berry-api"
                })
            })
            .collect();

        Json(json!({
            "object": "list",
            "data": model_list
        }))
    }
}
