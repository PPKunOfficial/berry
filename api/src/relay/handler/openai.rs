use axum::response::sse::Event;
use axum::response::Sse;
use axum::{extract::Json, response::IntoResponse};
use axum_extra::TypedHeader;
use eventsource_stream::Eventsource;
use futures::{Stream, StreamExt};
use serde_json::{Value, json};

use crate::relay::client::openai::OpenAIClient;
use super::types::{create_error_event, create_error_json, create_network_error_json, create_upstream_error_json};

// SSE 流式响应处理
async fn sse_completions(
    TypedHeader(authorization): TypedHeader<headers::Authorization<headers::authorization::Bearer>>,
    TypedHeader(content_type): TypedHeader<headers::ContentType>,
    Json(body): Json<Value>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let client = OpenAIClient::new();

    let event_stream = async move {
        // 构建请求头
        let headers = match client.build_request_headers(&authorization, &content_type) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("构建请求头失败: {:?}", e);
                return futures::stream::once(async move { Ok(create_error_event(&e)) }).boxed();
            }
        };

        let request_result = client.chat_completions(headers, &body).await;
        match request_result {
            Ok(resp) if resp.status().is_success() => {
                // 成功情况
                resp.bytes_stream()
                    .eventsource()
                    .map(|result| match result {
                        Ok(event) => {
                            tracing::debug!("SSE event: {:?}", event.data);
                            Ok(Event::default().data(event.data))
                        }
                        Err(err) => {
                            tracing::error!("SSE error: {:?}", err);
                            Ok(
                                Event::default()
                                    .data(json!({"error": err.to_string()}).to_string()),
                            )
                        }
                    })
                    .boxed() // 统一类型
            }
            Ok(resp) => {
                // HTTP 错误
                tracing::error!("请求上游失败: {}", resp.status());
                futures::stream::once(async move {
                    Ok(Event::default().data(
                        json!({
                            "error": {
                                "message": "请求上游失败",
                                "status": resp.status().as_u16()
                            }
                        })
                        .to_string(),
                    ))
                })
                .boxed()
            }
            Err(e) => {
                // 网络错误
                tracing::error!("请求异常: {:?}", e);
                futures::stream::once(async move {
                    Ok(Event::default().data(
                        json!({
                            "error": {
                                "message": "请求异常",
                                "details": e.to_string()
                            }
                        })
                        .to_string(),
                    ))
                })
                .boxed()
            }
        }
    }
    .await;
    Sse::new(event_stream)
}

// 非流式响应处理
async fn no_sse_completions(
    TypedHeader(authorization): TypedHeader<headers::Authorization<headers::authorization::Bearer>>,
    TypedHeader(content_type): TypedHeader<headers::ContentType>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let client = OpenAIClient::new();

    // 构建请求头
    let headers = match client.build_request_headers(&authorization, &content_type) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("构建请求头失败: {:?}", e);
            return Json(create_error_json(&e));
        }
    };

    // 发送API请求
    let response = match client.chat_completions(headers, &body).await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("API请求失败: {:?}", e);
            return Json(create_error_json(&e));
        }
    };

    // 处理响应
    if response.status().is_success() {
        match response.text().await {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => Json(value),
                Err(e) => {
                    tracing::error!("JSON解析失败: {:?}", e);
                    Json(create_network_error_json(
                        "上游返回数据格式错误",
                        Some(text),
                    ))
                }
            },
            Err(e) => {
                tracing::error!("读取响应体失败: {:?}", e);
                Json(create_network_error_json(
                    "无法读取响应数据",
                    Some(e.to_string()),
                ))
            }
        }
    } else {
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取错误响应".to_string());

        tracing::error!("上游API错误: 状态码 {}, 响应: {}", status, body);
        Json(create_upstream_error_json(
            "上游API返回错误",
            Some(status),
            Some(body),
        ))
    }
}

// 主要的聊天完成处理函数
pub async fn handle_completions(
    TypedHeader(authorization): TypedHeader<headers::Authorization<headers::authorization::Bearer>>,
    TypedHeader(content_type): TypedHeader<headers::ContentType>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let is_stream = body
        .get("stream")
        .unwrap_or(&Value::Bool(true))
        .as_bool()
        .unwrap_or(true);

    if is_stream {
        sse_completions(
            TypedHeader(authorization),
            TypedHeader(content_type),
            Json(body),
        )
        .await
        .into_response()
    } else {
        no_sse_completions(
            TypedHeader(authorization),
            TypedHeader(content_type),
            Json(body),
        )
        .await
        .into_response()
    }
}

// 模型列表处理函数
pub async fn handle_model(
    TypedHeader(authorization): TypedHeader<headers::Authorization<headers::authorization::Bearer>>,
) -> Json<Value> {
    let client = OpenAIClient::new();
    let token = authorization.token();
    
    match client.models(token).await {
        Ok(response) => {
            if response.is_success {
                match serde_json::from_str(&response.body) {
                    Ok(val) => Json(val),
                    Err(_) => Json(
                        json!({"berry-api-error": response.body,"错误信息": "上游返回数据格式错误，解析失败"}),
                    ),
                }
            } else {
                Json(json!({"berry-api-error": response.body,"错误信息": "请求上游失败"}))
            }
        }
        Err(e) => Json(json!({"berry-api-error": e.to_string(),"错误信息": "请求上游失败"})),
    }
}
