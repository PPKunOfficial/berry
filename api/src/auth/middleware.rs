use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::config::model::Config;
use super::types::{AuthenticatedUser, AuthError};

/// 认证中间件
pub struct AuthMiddleware;

impl AuthMiddleware {
    /// 从请求中提取并验证API密钥
    pub async fn authenticate(
        State(config): State<Arc<Config>>,
        mut request: Request,
        next: Next,
    ) -> Result<Response, Response> {
        // 提取Authorization头
        let auth_header = request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok());

        let token = match auth_header {
            Some(header) => {
                if header.starts_with("Bearer ") {
                    &header[7..] // 移除 "Bearer " 前缀
                } else {
                    return Err(create_auth_error_response(AuthError::missing_token()));
                }
            }
            None => {
                return Err(create_auth_error_response(AuthError::missing_token()));
            }
        };

        // 验证令牌
        let user_token = match config.validate_user_token(token) {
            Some(user) => user,
            None => {
                return Err(create_auth_error_response(AuthError::invalid_token()));
            }
        };

        // 检查用户是否启用
        if !user_token.enabled {
            return Err(create_auth_error_response(AuthError::disabled_user()));
        }

        // 创建认证用户信息
        let authenticated_user = AuthenticatedUser::new(
            token.to_string(), // 使用token作为user_id
            user_token.clone(),
        );

        // 将认证用户信息添加到请求扩展中
        request.extensions_mut().insert(authenticated_user);

        // 继续处理请求
        Ok(next.run(request).await)
    }

    /// 验证用户对特定模型的访问权限
    pub fn check_model_access(
        user: &AuthenticatedUser,
        model_name: &str,
        config: &Config,
    ) -> Result<(), AuthError> {
        // 检查模型是否存在且启用
        let model = config.get_model(model_name);
        if model.is_none() || !model.unwrap().enabled {
            return Err(AuthError::model_access_denied(model_name));
        }

        // 检查用户是否有权限访问该模型
        if !config.user_can_access_model(&user.user_token, model_name) {
            return Err(AuthError::model_access_denied(model_name));
        }

        Ok(())
    }
}

/// 创建认证错误响应
fn create_auth_error_response(error: AuthError) -> Response {
    let status_code = match error.status {
        401 => StatusCode::UNAUTHORIZED,
        403 => StatusCode::FORBIDDEN,
        429 => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (
        status_code,
        Json(json!({
            "error": {
                "type": error.error,
                "message": error.message,
                "code": error.status
            }
        })),
    )
        .into_response()
}

/// 从请求扩展中获取认证用户信息
pub fn get_authenticated_user(request: &Request) -> Option<&AuthenticatedUser> {
    request.extensions().get::<AuthenticatedUser>()
}

/// 简化的认证检查函数
pub fn validate_request_token<'a>(config: &'a Config, token: &str) -> Result<&'a crate::config::model::UserToken, AuthError> {
    match config.validate_user_token(token) {
        Some(user) if user.enabled => Ok(user),
        Some(_) => Err(AuthError::disabled_user()),
        None => Err(AuthError::invalid_token()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::{UserToken, RateLimit};
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut users = HashMap::new();
        users.insert("test-user".to_string(), UserToken {
            name: "Test User".to_string(),
            token: "test-token-123".to_string(),
            allowed_models: vec!["gpt-4".to_string()],
            enabled: true,
            rate_limit: Some(RateLimit {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                requests_per_day: 10000,
            }),
            tags: vec!["test".to_string()],
        });

        users.insert("admin-user".to_string(), UserToken {
            name: "Admin User".to_string(),
            token: "admin-token-456".to_string(),
            allowed_models: vec![], // 允许所有模型
            enabled: true,
            rate_limit: None,
            tags: vec!["admin".to_string()],
        });

        Config {
            providers: HashMap::new(),
            models: HashMap::new(),
            users,
            settings: Default::default(),
        }
    }

    #[test]
    fn test_validate_user_token() {
        let config = create_test_config();
        
        // 测试有效令牌
        let user = config.validate_user_token("test-token-123");
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "Test User");

        // 测试无效令牌
        let user = config.validate_user_token("invalid-token");
        assert!(user.is_none());
    }

    #[test]
    fn test_user_can_access_model() {
        let config = create_test_config();
        
        let test_user = config.validate_user_token("test-token-123").unwrap();
        let admin_user = config.validate_user_token("admin-token-456").unwrap();

        // 测试用户只能访问允许的模型
        assert!(config.user_can_access_model(test_user, "gpt-4"));
        assert!(!config.user_can_access_model(test_user, "gpt-3.5-turbo"));

        // 管理员用户可以访问所有模型
        assert!(config.user_can_access_model(admin_user, "gpt-4"));
        assert!(config.user_can_access_model(admin_user, "gpt-3.5-turbo"));
    }
}
