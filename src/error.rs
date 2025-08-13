use reqwest;
use serde_json;
use std::error::Error as StdError;
use std::fmt;
use std::io;

/// 错误类型定义
#[derive(Debug)]
pub enum TransomeError {
    /// 模型未找到错误
    ModelNotFound {
        model_name: String,
        available_models: Vec<String>,
    },

    /// API调用失败错误
    ApiCallFailed {
        endpoint: String,
        status_code: Option<u16>,
        message: String,
    },

    /// 网络错误
    NetworkError { source: reqwest::Error },

    /// JSON解析错误
    JsonError {
        source: serde_json::Error,
        context: String,
    },

    /// IO错误
    IoError { source: io::Error, context: String },

    /// 认证错误
    AuthenticationError { message: String },

    /// 配置错误
    ConfigError { field: String, message: String },

    /// 验证错误
    ValidationError {
        field: String,
        expected: String,
        actual: String,
    },

    /// 模型加载错误
    ModelLoadError { model_path: String, reason: String },

    /// 翻译服务错误
    TranslationServiceError { service: String, message: String },

    /// 通用错误
    General { message: String },
}

impl fmt::Display for TransomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransomeError::ModelNotFound {
                model_name,
                available_models,
            } => {
                write!(
                    f,
                    "未找到模型 '{}'。可用模型：[{}]",
                    model_name,
                    available_models.join(", ")
                )
            }

            TransomeError::ApiCallFailed {
                endpoint,
                status_code,
                message,
            } => match status_code {
                Some(code) => write!(
                    f,
                    "API 调用 '{}' 失败，状态码 {}：{}",
                    endpoint, code, message
                ),
                None => write!(f, "API 调用 '{}' 失败：{}", endpoint, message),
            },

            TransomeError::NetworkError { source } => {
                write!(f, "网络错误：{}", source)
            }

            TransomeError::JsonError { source, context } => {
                write!(f, "{} 中的 JSON 错误：{}", context, source)
            }

            TransomeError::IoError { source, context } => {
                write!(f, "{} 中的 IO 错误：{}", context, source)
            }

            TransomeError::AuthenticationError { message } => {
                write!(f, "认证失败：{}", message)
            }

            TransomeError::ConfigError { field, message } => {
                write!(f, "配置字段 '{}' 错误：{}", field, message)
            }

            TransomeError::ValidationError {
                field,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "验证字段 '{}' 错误：期望 {}，实际 {}",
                    field, expected, actual
                )
            }

            TransomeError::ModelLoadError { model_path, reason } => {
                write!(f, "从 '{}' 加载模型失败：{}", model_path, reason)
            }

            TransomeError::TranslationServiceError { service, message } => {
                write!(f, "翻译服务 '{}' 错误：{}", service, message)
            }

            TransomeError::General { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl StdError for TransomeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            TransomeError::NetworkError { source } => Some(source),
            TransomeError::JsonError { source, .. } => Some(source),
            TransomeError::IoError { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for TransomeError {
    fn from(error: reqwest::Error) -> Self {
        TransomeError::NetworkError { source: error }
    }
}

impl From<serde_json::Error> for TransomeError {
    fn from(error: serde_json::Error) -> Self {
        TransomeError::JsonError {
            source: error,
            context: "未知".to_string(),
        }
    }
}

impl From<io::Error> for TransomeError {
    fn from(error: io::Error) -> Self {
        TransomeError::IoError {
            source: error,
            context: "未知".to_string(),
        }
    }
}

impl From<String> for TransomeError {
    fn from(message: String) -> Self {
        TransomeError::General { message }
    }
}

impl From<&str> for TransomeError {
    fn from(message: &str) -> Self {
        TransomeError::General {
            message: message.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, TransomeError>;

impl TransomeError {
    pub fn model_not_found(model_name: impl Into<String>, available_models: Vec<String>) -> Self {
        TransomeError::ModelNotFound {
            model_name: model_name.into(),
            available_models,
        }
    }

    pub fn api_call_failed(
        endpoint: impl Into<String>,
        status_code: Option<u16>,
        message: impl Into<String>,
    ) -> Self {
        TransomeError::ApiCallFailed {
            endpoint: endpoint.into(),
            status_code,
            message: message.into(),
        }
    }

    pub fn json_error_with_context(error: serde_json::Error, context: impl Into<String>) -> Self {
        TransomeError::JsonError {
            source: error,
            context: context.into(),
        }
    }

    pub fn io_error_with_context(error: io::Error, context: impl Into<String>) -> Self {
        TransomeError::IoError {
            source: error,
            context: context.into(),
        }
    }

    pub fn authentication_error(message: impl Into<String>) -> Self {
        TransomeError::AuthenticationError {
            message: message.into(),
        }
    }

    pub fn config_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        TransomeError::ConfigError {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn validation_error(
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        TransomeError::ValidationError {
            field: field.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    pub fn model_load_error(model_path: impl Into<String>, reason: impl Into<String>) -> Self {
        TransomeError::ModelLoadError {
            model_path: model_path.into(),
            reason: reason.into(),
        }
    }

    pub fn translation_service_error(
        service: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        TransomeError::TranslationServiceError {
            service: service.into(),
            message: message.into(),
        }
    }

    pub fn is_network_error(&self) -> bool {
        matches!(self, TransomeError::NetworkError { .. })
    }

    pub fn is_auth_error(&self) -> bool {
        matches!(self, TransomeError::AuthenticationError { .. })
    }

    pub fn is_config_error(&self) -> bool {
        matches!(self, TransomeError::ConfigError { .. })
    }

    pub fn user_friendly_message(&self) -> String {
        match self {
            TransomeError::ModelNotFound {
                model_name,
                available_models,
            } => {
                if available_models.is_empty() {
                    format!("找不到模型 '{}'，当前没有可用的模型", model_name)
                } else {
                    format!(
                        "找不到模型 '{}'，可用的模型有：{}",
                        model_name,
                        available_models.join("、")
                    )
                }
            }

            TransomeError::ApiCallFailed {
                endpoint: _,
                status_code,
                message,
            } => match status_code {
                Some(code) if *code >= 400 && *code < 500 => {
                    format!("请求错误 ({}): 请检查参数或权限配置", code)
                }
                Some(code) if *code >= 500 => {
                    format!("服务器错误 ({}): 请稍后重试", code)
                }
                _ => format!("API调用失败: {}", message),
            },

            TransomeError::NetworkError { source } => {
                if source.is_connect() {
                    "网络连接失败，请检查网络设置".to_string()
                } else if source.is_timeout() {
                    "请求超时，请稍后重试".to_string()
                } else {
                    "网络错误，请检查网络连接".to_string()
                }
            }

            TransomeError::AuthenticationError { .. } => {
                "认证失败，请检查API密钥或凭据配置".to_string()
            }

            TransomeError::ConfigError { field, .. } => {
                format!("配置错误：请检查 '{}' 字段的设置", field)
            }

            TransomeError::ValidationError {
                field, expected, ..
            } => {
                format!("参数错误：'{}' 字段应为 {}", field, expected)
            }

            TransomeError::ModelLoadError { model_path, .. } => {
                format!("模型加载失败：无法从 '{}' 加载模型", model_path)
            }

            TransomeError::TranslationServiceError { service, .. } => {
                format!("翻译服务 '{}' 暂时不可用，请稍后重试", service)
            }

            _ => "操作失败，请重试".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_not_found_error() {
        // 测试模型未找到错误
        let error = TransomeError::model_not_found("gpt-4", vec!["gpt-3.5".to_string()]);
        assert!(error.to_string().contains("gpt-4"));
        assert!(error.to_string().contains("gpt-3.5"));
    }

    #[test]
    fn test_api_call_failed_error() {
        // 测试API调用失败错误
        let error = TransomeError::api_call_failed("/api/translate", Some(404), "Not found");
        assert!(error.to_string().contains("404"));
        assert!(error.to_string().contains("/api/translate"));
    }

    #[test]
    fn test_user_friendly_message() {
        // 测试用户友好错误消息
        let error = TransomeError::model_not_found("gpt-4", vec!["gpt-3.5".to_string()]);
        let friendly = error.user_friendly_message();
        assert!(friendly.contains("找不到模型"));
        assert!(friendly.contains("可用的模型"));
    }

    #[test]
    fn test_error_type_checks() {
        // 测试错误类型检查
        let auth_error = TransomeError::authentication_error("Invalid key");
        assert!(auth_error.is_auth_error());

        let config_error = TransomeError::config_error("api_key", "Missing");
        assert!(config_error.is_config_error());

        let general_error = TransomeError::General {
            message: "test".to_string(),
        };
        assert!(!general_error.is_network_error());
    }

    #[test]
    fn test_from_conversions() {
        // 测试类型转换
        let string_error: TransomeError = "test error".into();
        assert!(matches!(string_error, TransomeError::General { .. }));

        let owned_string_error: TransomeError = "test error".to_string().into();
        assert!(matches!(owned_string_error, TransomeError::General { .. }));
    }
}
