//! Transome - 命令行翻译工具库

// 声明所有模块
pub mod cli;
pub mod config;
pub mod error;
pub mod translator;

// 重新导出主要的公共接口

// 从 cli 模块导出
pub use cli::Cli;

// 从 config 模块导出
pub use config::{
    ModelConfig, create_model_error_message, get_all_models, get_model_url, get_provider_name,
    get_supported_model_names, is_model_supported as config_is_model_supported, list_models,
};

// 从 error 模块导出
pub use error::{Result, TransomeError};

// 从 translator 模块导出
pub use translator::{PROMPT, Translator};

// 类型别名和常量
/// 版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 库描述
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// 结果类型
pub type LibResult<T> = std::result::Result<T, TransomeError>;

// 功能函数

/// 创建翻译器实例
pub fn create_translator(
    api_key: String,
    model: String,
    custom_url: Option<String>,
) -> LibResult<Translator> {
    let api_base = match custom_url {
        Some(url) => url,
        None => get_model_url(&model).ok_or_else(|| {
            let available_models = get_supported_model_names();
            TransomeError::model_not_found(model.clone(), available_models)
        })?,
    };

    Ok(Translator::new(api_key, api_base, model))
}

/// 获取支持的模型列表
pub fn get_supported_models() -> Vec<String> {
    get_supported_model_names()
}

/// 检查模型是否支持
pub fn is_model_supported(model_name: &str) -> bool {
    config_is_model_supported(model_name)
}

/// 获取模型提供商名称
pub fn get_model_provider(model_name: &str) -> &'static str {
    if is_model_supported(model_name) {
        get_provider_name(model_name)
    } else {
        "Unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
        assert_eq!(NAME, "transome");
    }

    #[test]
    fn test_get_supported_models() {
        let models = get_supported_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"gpt-4".to_string()));
        assert!(models.contains(&"gemini-2.5-flash".to_string()));
    }

    #[test]
    fn test_is_model_supported() {
        assert!(is_model_supported("gpt-4"));
        assert!(is_model_supported("gemini-2.5-flash"));
        assert!(!is_model_supported("nonexistent-model"));
    }

    #[test]
    fn test_get_model_provider() {
        assert_eq!(get_model_provider("gpt-4"), "OpenAI");
        assert_eq!(get_model_provider("gemini-2.5-flash"), "Google Gemini");
        assert_eq!(get_model_provider("nonexistent-model"), "Unknown");
    }

    #[test]
    fn test_create_translator_with_valid_model() {
        let result = create_translator("test-key".to_string(), "gpt-4".to_string(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_translator_with_invalid_model() {
        let result = create_translator(
            "test-key".to_string(),
            "nonexistent-model".to_string(),
            None,
        );
        assert!(result.is_err());

        if let Err(TransomeError::ModelNotFound { model_name, .. }) = result {
            assert_eq!(model_name, "nonexistent-model");
        } else {
            panic!("Expected ModelNotFound error");
        }
    }

    #[test]
    fn test_create_translator_with_custom_url() {
        let result = create_translator(
            "test-key".to_string(),
            "custom-model".to_string(),
            Some("https://custom.api.com/v1".to_string()),
        );
        assert!(result.is_ok());
    }
}
