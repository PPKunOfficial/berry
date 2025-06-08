use thiserror::Error;

// 定义客户端错误类型
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("请求头解析失败: {0}")]
    HeaderParseError(String),
    #[error("HTTP请求失败: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("JSON解析失败: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("上游API返回错误: 状态码 {status}")]
    UpstreamError { status: u16, body: String },
}

// 客户端响应类型
#[derive(Debug)]
pub struct ClientResponse {
    pub status: u16,
    pub body: String,
    pub is_success: bool,
}

impl ClientResponse {
    pub fn new(status: u16, body: String) -> Self {
        Self {
            status,
            body,
            is_success: status >= 200 && status < 300,
        }
    }
}
