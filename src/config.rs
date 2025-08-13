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
    pub fn new(
        name: impl Into<String>,
        url: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
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
            (
                "gemini-2.5-pro",
                "https://generativelanguage.googleapis.com/v1beta/openai",
            ),
            (
                "gemini-2.5-flash",
                "https://generativelanguage.googleapis.com/v1beta/openai",
            ),
            (
                "gemini-2.5-flash-lite",
                "https://generativelanguage.googleapis.com/v1beta/openai",
            ),
            (
                "gemini-1.5-pro",
                "https://generativelanguage.googleapis.com/v1beta/openai",
            ),
            (
                "gemini-1.5-flash",
                "https://generativelanguage.googleapis.com/v1beta/openai",
            ),
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
    println!("\n支持的模型:");

    let sorted_providers = get_sorted_providers_with_models();

    for (provider, models) in sorted_providers {
        if let Some((_, first_url)) = models.first() {
            println!("\n{} ({}):", provider, first_url);
            for (model, _) in models {
                println!("  - {}", model);
            }
        }
    }

    println!("\n使用方法:");
    println!("  transome [选项] [文本]");
    println!("\n选项:");
    println!("  -m, --model <模型>    使用上述列表中的支持模型");
    println!("  -u, --url <地址>      使用自定义 API 地址（覆盖模型选择）");
    println!("\n示例:");
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
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.name.cmp(&b.name))
    });

    models
}

/// 为不支持的模型创建错误消息
pub fn create_model_error_message(model: &str) -> String {
    let sorted_providers = get_sorted_providers_with_models();

    let mut error_msg = format!("找不到模型 '{}'\n\n支持的模型:", model);

    for (provider, models) in sorted_providers {
        let model_names: Vec<&str> = models.iter().map(|&(name, _)| name).collect();
        error_msg.push_str(&format!("\n\n{}: {}", provider, model_names.join(", ")));
    }

    error_msg.push_str("\n\n使用方法:");
    error_msg.push_str("\n  使用支持的模型: transome -m <模型名称> \"<文本>\"");
    error_msg.push_str("\n  或提供自定义 URL: transome -u <URL> -m <模型名称> \"<文本>\"");
    error_msg.push_str("\n  列出所有模型: transome --list-models");

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

/// 根据模型名称获取对应的环境变量名
pub fn get_env_var_name_for_model(model: &str) -> Option<&'static str> {
    let provider = get_provider_name(model);

    match provider {
        "OpenAI" => Some("OPENAI_API_KEY"),
        "Google Gemini" => Some("GOOGLE_AI_API_KEY"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_url() {
        assert_eq!(
            get_model_url("gpt-4"),
            Some("https://api.openai.com/v1".to_string())
        );
        assert_eq!(
            get_model_url("gemini-2.5-flash"),
            Some("https://generativelanguage.googleapis.com/v1beta/openai".to_string())
        );
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

        // 检查是否包含两个提供商的模型
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

        // 检查提供商是否按字母顺序排序
        for i in 1..sorted_providers.len() {
            assert!(sorted_providers[i - 1].0 <= sorted_providers[i].0);
        }

        // 检查每个提供商内的模型是否已排序
        for (_, models) in sorted_providers {
            for i in 1..models.len() {
                assert!(models[i - 1].0 <= models[i].0);
            }
        }
    }

    #[test]
    fn test_create_model_error_message() {
        let error_msg = create_model_error_message("nonexistent-model");
        assert!(error_msg.contains("nonexistent-model"));
        assert!(error_msg.contains("找不到模型"));
        assert!(error_msg.contains("支持的模型"));
        assert!(error_msg.contains("OpenAI"));
        assert!(error_msg.contains("Google Gemini"));
        assert!(error_msg.contains("使用方法"));
    }

    #[test]
    fn test_get_env_var_name_for_model() {
        // 测试 OpenAI 模型
        assert_eq!(get_env_var_name_for_model("gpt-4"), Some("OPENAI_API_KEY"));
        assert_eq!(get_env_var_name_for_model("gpt-4o"), Some("OPENAI_API_KEY"));
        assert_eq!(
            get_env_var_name_for_model("gpt-3.5-turbo"),
            Some("OPENAI_API_KEY")
        );
        assert_eq!(
            get_env_var_name_for_model("gpt-4-turbo"),
            Some("OPENAI_API_KEY")
        );
        assert_eq!(
            get_env_var_name_for_model("gpt-4o-mini"),
            Some("OPENAI_API_KEY")
        );
        assert_eq!(
            get_env_var_name_for_model("gpt-3.5-turbo-16k"),
            Some("OPENAI_API_KEY")
        );

        // 测试 Google Gemini 模型
        assert_eq!(
            get_env_var_name_for_model("gemini-2.5-flash"),
            Some("GOOGLE_AI_API_KEY")
        );
        assert_eq!(
            get_env_var_name_for_model("gemini-1.5-pro"),
            Some("GOOGLE_AI_API_KEY")
        );
        assert_eq!(
            get_env_var_name_for_model("gemini-2.5-pro"),
            Some("GOOGLE_AI_API_KEY")
        );
        assert_eq!(
            get_env_var_name_for_model("gemini-2.5-flash-lite"),
            Some("GOOGLE_AI_API_KEY")
        );
        assert_eq!(
            get_env_var_name_for_model("gemini-1.5-flash"),
            Some("GOOGLE_AI_API_KEY")
        );

        // 测试不支持的模型
        assert_eq!(get_env_var_name_for_model("nonexistent-model"), None);
        assert_eq!(get_env_var_name_for_model("claude-3"), None);
        assert_eq!(get_env_var_name_for_model("llama-2"), None);

        // 测试 URL 输入（非模型 URL 应返回 None）
        assert_eq!(get_env_var_name_for_model("https://custom.api.com"), None);
        assert_eq!(get_env_var_name_for_model("http://localhost:8080"), None);
    }

    #[test]
    fn test_get_env_var_name_for_model_comprehensive() {
        // 测试每个支持的模型返回正确的环境变量
        let openai_models = vec![
            "gpt-4",
            "gpt-4-turbo",
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-3.5-turbo",
            "gpt-3.5-turbo-16k",
        ];
        let gemini_models = vec![
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-2.5-flash-lite",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
        ];

        // 所有 OpenAI 模型应返回 OPENAI_API_KEY
        for model in openai_models {
            assert_eq!(
                get_env_var_name_for_model(model),
                Some("OPENAI_API_KEY"),
                "Model {} should return OPENAI_API_KEY",
                model
            );
        }

        // 所有 Gemini 模型应返回 GOOGLE_AI_API_KEY
        for model in gemini_models {
            assert_eq!(
                get_env_var_name_for_model(model),
                Some("GOOGLE_AI_API_KEY"),
                "Model {} should return GOOGLE_AI_API_KEY",
                model
            );
        }
    }

    #[test]
    fn test_get_env_var_name_for_model_edge_cases() {
        // 测试空字符串
        assert_eq!(get_env_var_name_for_model(""), None);

        // 测试具有相似前缀但不完全匹配的字符串
        assert_eq!(get_env_var_name_for_model("gpt"), None);
        assert_eq!(get_env_var_name_for_model("gemini"), None);
        assert_eq!(get_env_var_name_for_model("gpt-5"), None); // 假设的未来模型

        // 测试大小写敏感性 - 我们的模型是小写的，所以大写应该失败
        assert_eq!(get_env_var_name_for_model("GPT-4"), None);
        assert_eq!(get_env_var_name_for_model("GEMINI-2.5-FLASH"), None);

        // 测试额外的空白字符（应该失败，因为我们不进行修剪）
        assert_eq!(get_env_var_name_for_model(" gpt-4 "), None);
        assert_eq!(get_env_var_name_for_model("gpt-4\n"), None);
    }
}
