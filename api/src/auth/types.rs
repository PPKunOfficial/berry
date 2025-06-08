use crate::config::model::UserToken;
use serde::Serialize;

/// 认证用户信息
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub user_token: UserToken,
}

impl AuthenticatedUser {
    pub fn new(user_id: String, user_token: UserToken) -> Self {
        Self { user_id, user_token }
    }

    /// 检查用户是否可以访问指定模型
    pub fn can_access_model(&self, model_name: &str) -> bool {
        // 如果allowed_models为空，表示允许访问所有模型
        if self.user_token.allowed_models.is_empty() {
            return true;
        }
        
        // 检查模型是否在允许列表中
        self.user_token.allowed_models.contains(&model_name.to_string())
    }

    /// 获取用户名
    pub fn get_name(&self) -> &str {
        &self.user_token.name
    }

    /// 获取用户标签
    pub fn get_tags(&self) -> &[String] {
        &self.user_token.tags
    }

    /// 检查用户是否有指定标签
    pub fn has_tag(&self, tag: &str) -> bool {
        self.user_token.tags.contains(&tag.to_string())
    }
}

/// 认证错误类型
#[derive(Debug, Clone, Serialize)]
pub struct AuthError {
    pub error: String,
    pub message: String,
    pub status: u16,
}

impl AuthError {
    pub fn missing_token() -> Self {
        Self {
            error: "missing_authorization".to_string(),
            message: "Authorization header is missing or invalid".to_string(),
            status: 401,
        }
    }

    pub fn invalid_token() -> Self {
        Self {
            error: "invalid_token".to_string(),
            message: "The provided API key is invalid".to_string(),
            status: 401,
        }
    }

    pub fn disabled_user() -> Self {
        Self {
            error: "disabled_user".to_string(),
            message: "User account is disabled".to_string(),
            status: 403,
        }
    }

    pub fn model_access_denied(model_name: &str) -> Self {
        Self {
            error: "model_access_denied".to_string(),
            message: format!("Access denied for model: {}", model_name),
            status: 403,
        }
    }

    pub fn rate_limit_exceeded() -> Self {
        Self {
            error: "rate_limit_exceeded".to_string(),
            message: "Rate limit exceeded. Please try again later".to_string(),
            status: 429,
        }
    }
}
