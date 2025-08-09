//! 模型配置和 URL 映射模块

use std::collections::HashMap;
use std::sync::OnceLock;

/// AI 模型配置结构
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelConfig {
    pub name: String,
    pub url: String,
    pub provider: String,
}

impl ModelConfig {
    /// 创建新的模型配置实例
    pub fn new(name: impl Into<String>, url: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            provider: provider.into(),
        }
    }
}

/// 模型名称到 API 端点的静态映射
static MODEL_TO_URL: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

/// 获取模型到 URL 的映射
fn get_model_to_url() -> &'static HashMap<&'static str, &'static str> {
    MODEL_TO_URL.get_or_init(|| {
        HashMap::from([
            // Google Gemini 模型 - 使用 OpenAI 兼容端点
            ("gemini-2.5-pro", "https://generativelanguage.googleapis.com/v1beta/openai"),
            ("gemini-2.5-flash", "https://generativelanguage.googleapis.com/v1beta/openai"),
            ("gemini-2.5-flash-lite", "https://generativelanguage.googleapis.com/v1beta/openai"),
            ("gemini-1.5-pro", "https://generativelanguage.googleapis.com/v1beta/openai"),
            ("gemini-1.5-flash", "https://generativelanguage.googleapis.com/v1beta/openai"),
            
            // OpenAI 模型 - 官方 API 端点
            ("gpt-4", "https://api.openai.com/v1"),
            ("gpt-4-turbo", "https://api.openai.com/v1"),
            ("gpt-4o", "https://api.openai.com/v1"),
            ("gpt-4o-mini", "https://api.openai.com/v1"),
            ("gpt-3.5-turbo", "https://api.openai.com/v1"),
            ("gpt-3.5-turbo-16k", "https://api.openai.com/v1"),
        ])
    })
}

/// 获取模型的 API URL
pub fn get_model_url(model: &str) -> Option<String> {
    let model_to_url = get_model_to_url();
    model_to_url.get(model).map(|&url| url.to_string())
}

/// 获取提供商名称
pub fn get_provider_name(model_or_url: &str) -> &'static str {
    // 首先尝试从模型名称获取 URL
    let url = if let Some(model_url) = get_model_url(model_or_url) {
        model_url
    } else {
        // 如果未找到模型名称，则将其视为 URL
        model_or_url.to_string()
    };

    if url.contains("generativelanguage.googleapis.com") {
        "Google Gemini"
    } else if url.contains("api.openai.com") {
        "OpenAI"
    } else {
        "Other"
    }
}

/// 按提供商分组模型
fn group_models_by_provider() -> HashMap<&'static str, Vec<(&'static str, &'static str)>> {
    let model_to_url = get_model_to_url();
    let mut providers: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
    
    for (&model, &url) in model_to_url.iter() {
        let provider = get_provider_name(model);
        providers.entry(provider).or_default().push((model, url));
    }
    
    providers
}

/// 获取排序后的提供商及其模型
fn get_sorted_providers_with_models() -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
    let providers = group_models_by_provider();
    let mut sorted_providers: Vec<_> = providers.into_iter().collect();
    sorted_providers.sort_by_key(|&(name, _)| name);
    
    // 对每个提供商内的模型进行排序
    for (_, models) in &mut sorted_providers {
        models.sort_by_key(|&(model, _)| model);
    }
    
    sorted_providers
}

/// 列出所有支持的模型
pub fn list_models() {
    println!("\nSupported models:");
    
    let sorted_providers = get_sorted_providers_with_models();
    
    for (provider, models) in sorted_providers {
        if let Some((_, first_url)) = models.first() {
            println!("\n{} ({}):", provider, first_url);
            for (model, _) in models {
                println!("  - {}", model);
            }
        }
    }
    
    println!("\nUsage:");
    println!("  transome [OPTIONS] [TEXT]");
    println!("\nOptions:");
    println!("  -m, --model <MODEL>    Use a supported model from the list above");
    println!("  -u, --url <URL>        Use a custom API URL (overrides model selection)");
    println!("\nExamples:");
    println!("  transome -m gpt-4 \"Hello world\"");
    println!("  transome -u https://custom.api.com/v1 -m custom-model \"Hello world\"");
}

/// 获取所有可用模型
pub fn get_all_models() -> Vec<ModelConfig> {
    let model_to_url = get_model_to_url();
    let mut models = Vec::new();
    
    for (&model, &url) in model_to_url.iter() {
        let provider = get_provider_name(model);
        models.push(ModelConfig::new(model, url, provider));
    }
    
    // 首先按提供商排序，然后按模型名称排序以保持一致的顺序
    models.sort_by(|a, b| {
        a.provider.cmp(&b.provider)
            .then_with(|| a.name.cmp(&b.name))
    });
    
    models
}

/// 为不支持的模型创建错误消息
pub fn create_model_error_message(model: &str) -> String {
    let sorted_providers = get_sorted_providers_with_models();
    
    let mut error_msg = format!("Model '{}' not found.\n\nSupported models:", model);
    
    for (provider, models) in sorted_providers {
        let model_names: Vec<&str> = models.iter().map(|&(name, _)| name).collect();
        error_msg.push_str(&format!("\n\n{}: {}", provider, model_names.join(", ")));
    }
    
    error_msg.push_str("\n\nUsage:");
    error_msg.push_str("\n  Use one of the supported models: transome -m <MODEL> \"<text>\"");
    error_msg.push_str("\n  Or provide a custom URL: transome -u <URL> -m <MODEL> \"<text>\"");
    error_msg.push_str("\n  List all models: transome --list-models");
    
    error_msg
}

/// 检查模型是否被支持
pub fn is_model_supported(model_name: &str) -> bool {
    get_model_url(model_name).is_some()
}

/// 获取所有支持的模型名称列表
pub fn get_supported_model_names() -> Vec<String> {
    get_all_models().into_iter().map(|m| m.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_url() {
        assert_eq!(get_model_url("gpt-4"), Some("https://api.openai.com/v1".to_string()));
        assert_eq!(get_model_url("gemini-2.5-flash"), Some("https://generativelanguage.googleapis.com/v1beta/openai".to_string()));
        assert_eq!(get_model_url("nonexistent"), None);
    }
    
    #[test]
    fn test_is_model_supported() {
        assert!(is_model_supported("gpt-4"));
        assert!(is_model_supported("gemini-2.5-flash"));
        assert!(!is_model_supported("nonexistent-model"));
    }
    
    #[test]
    fn test_get_supported_model_names() {
        let models = get_supported_model_names();
        assert!(!models.is_empty());
        assert!(models.contains(&"gpt-4".to_string()));
        assert!(models.contains(&"gemini-2.5-flash".to_string()));
    }

    #[test]
    fn test_get_provider_name() {
        assert_eq!(get_provider_name("gpt-4"), "OpenAI");
        assert_eq!(get_provider_name("gemini-2.5-flash"), "Google Gemini");
        assert_eq!(get_provider_name("https://api.openai.com/v1"), "OpenAI");
        assert_eq!(get_provider_name("https://custom.api.com"), "Other");
    }

    #[test]
    fn test_get_all_models() {
        let models = get_all_models();
        assert!(!models.is_empty());
        
        // Check that we have models from both providers
        let has_openai = models.iter().any(|m| m.provider == "OpenAI");
        let has_gemini = models.iter().any(|m| m.provider == "Google Gemini");
        assert!(has_openai);
        assert!(has_gemini);
    }

    #[test]
    fn test_model_config() {
        let config = ModelConfig::new("test-model", "https://test.com", "TestProvider");
        assert_eq!(config.name, "test-model");
        assert_eq!(config.url, "https://test.com");
        assert_eq!(config.provider, "TestProvider");
    }
    
    #[test]
    fn test_group_models_by_provider() {
        let providers = group_models_by_provider();
        assert!(!providers.is_empty());
        assert!(providers.contains_key("OpenAI"));
        assert!(providers.contains_key("Google Gemini"));
    }
    
    #[test]
    fn test_get_sorted_providers_with_models() {
        let sorted_providers = get_sorted_providers_with_models();
        assert!(!sorted_providers.is_empty());
        
        // Check that providers are sorted alphabetically
        for i in 1..sorted_providers.len() {
            assert!(sorted_providers[i-1].0 <= sorted_providers[i].0);
        }
        
        // Check that models within each provider are sorted
        for (_, models) in sorted_providers {
            for i in 1..models.len() {
                assert!(models[i-1].0 <= models[i].0);
            }
        }
    }
    
    #[test]
    fn test_create_model_error_message() {
        let error_msg = create_model_error_message("nonexistent-model");
        assert!(error_msg.contains("nonexistent-model"));
        assert!(error_msg.contains("Model"));
        assert!(error_msg.contains("not found"));
        assert!(error_msg.contains("OpenAI"));
        assert!(error_msg.contains("Google Gemini"));
        assert!(error_msg.contains("Usage"));
    }
}
