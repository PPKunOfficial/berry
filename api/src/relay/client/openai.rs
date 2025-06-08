use reqwest::Client;
use serde_json::Value;
use super::types::{ClientError, ClientResponse};

const OPENAI_API_URL: &str = "https://aigc.x-see.cn/v1";

#[derive(Clone)]
pub struct OpenAIClient {
    client: Client,
    base_url: String,
}

impl OpenAIClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: OPENAI_API_URL.to_string(),
        }
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    // 构建请求头
    pub fn build_request_headers(
        &self,
        authorization: &headers::Authorization<headers::authorization::Bearer>,
        content_type: &headers::ContentType,
    ) -> Result<reqwest::header::HeaderMap, ClientError> {
        let mut headers = reqwest::header::HeaderMap::new();

        let auth_value = format!("Bearer {}", authorization.token())
            .parse()
            .map_err(|e| ClientError::HeaderParseError(format!("Authorization header: {}", e)))?;

        let content_type_value = content_type
            .to_string()
            .parse()
            .map_err(|e| ClientError::HeaderParseError(format!("Content-Type header: {}", e)))?;

        headers.insert("Authorization", auth_value);
        headers.insert("Content-Type", content_type_value);

        Ok(headers)
    }

    // 发送聊天完成请求
    pub async fn chat_completions(
        &self,
        headers: reqwest::header::HeaderMap,
        body: &Value,
    ) -> Result<reqwest::Response, ClientError> {
        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .headers(headers)
            .json(body)
            .send()
            .await?;

        Ok(response)
    }

    // 获取模型列表
    pub async fn models(
        &self,
        token: &str,
    ) -> Result<ClientResponse, ClientError> {
        let auth_header_value = format!("Bearer {}", token);
        let response = self.client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", auth_header_value)
            .send()
            .await?;

        let status = response.status().as_u16();
        let body = response.text().await?;

        Ok(ClientResponse::new(status, body))
    }
}

impl Default for OpenAIClient {
    fn default() -> Self {
        Self::new()
    }
}
