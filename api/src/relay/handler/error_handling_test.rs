#[cfg(test)]
mod tests {
    use super::super::types::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use crate::relay::client::ClientError;

    #[test]
    fn test_error_type_from_message() {
        // 测试不同错误消息的分类
        assert!(matches!(
            ErrorType::from_error_message("unauthorized access"),
            ErrorType::Unauthorized
        ));

        assert!(matches!(
            ErrorType::from_error_message("invalid token provided"),
            ErrorType::Unauthorized
        ));

        assert!(matches!(
            ErrorType::from_error_message("permission denied"),
            ErrorType::Forbidden
        ));

        assert!(matches!(
            ErrorType::from_error_message("model not found"),
            ErrorType::NotFound
        ));

        assert!(matches!(
            ErrorType::from_error_message("request timeout occurred"),
            ErrorType::GatewayTimeout
        ));

        assert!(matches!(
            ErrorType::from_error_message("too many requests"),
            ErrorType::TooManyRequests
        ));

        assert!(matches!(
            ErrorType::from_error_message("service unavailable - no available backends"),
            ErrorType::ServiceUnavailable
        ));

        assert!(matches!(
            ErrorType::from_error_message("all backends are unhealthy"),
            ErrorType::ServiceUnavailable
        ));

        assert!(matches!(
            ErrorType::from_error_message("bad request format"),
            ErrorType::BadRequest
        ));

        assert!(matches!(
            ErrorType::from_error_message("unknown error occurred"),
            ErrorType::InternalServerError
        ));
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(ErrorType::BadRequest.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(ErrorType::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(ErrorType::Forbidden.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(ErrorType::NotFound.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ErrorType::RequestTimeout.status_code(), StatusCode::REQUEST_TIMEOUT);
        assert_eq!(ErrorType::TooManyRequests.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(ErrorType::InternalServerError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(ErrorType::ServiceUnavailable.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(ErrorType::GatewayTimeout.status_code(), StatusCode::GATEWAY_TIMEOUT);
    }

    #[test]
    fn test_create_error_response() {
        let response = create_error_response(
            ErrorType::ServiceUnavailable,
            "Service is down",
            Some("All backends are unhealthy".to_string()),
        );

        // 这里我们无法直接测试响应内容，但可以确保函数不会panic
        let _response = response.into_response();
    }

    #[test]
    fn test_create_client_error_response() {
        let client_error = ClientError::HeaderParseError("Invalid authorization header".to_string());
        let response = create_client_error_response(&client_error);

        // 确保函数不会panic
        let _response = response.into_response();
    }

    #[test]
    fn test_service_unavailable_response() {
        let response = create_service_unavailable_response(
            "Backend selection failed",
            Some("No healthy backends available".to_string()),
        );

        let _response = response.into_response();
    }

    #[test]
    fn test_internal_error_response() {
        let response = create_internal_error_response(
            "Configuration error",
            Some("API key not found".to_string()),
        );

        let _response = response.into_response();
    }

    #[test]
    fn test_gateway_timeout_response() {
        let response = create_gateway_timeout_response(
            "Request timeout",
            Some("Backend did not respond in time".to_string()),
        );

        let _response = response.into_response();
    }
}
