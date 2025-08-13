//! 命令行参数解析模块

use anyhow::{Result, bail};
use clap::Parser;

use crate::config;
use crate::translator::PROMPT;

/// 命令行参数结构体
#[derive(Parser, Debug, Clone)]
#[command(name = "transome")]
#[command(version = "v0.2.0")]
#[command(about = "一个简单的命令行翻译工具", long_about = None)]
pub struct Cli {
    /// 要翻译的文本
    pub text: Option<String>,

    /// 翻译使用的AI模型
    #[arg(short, long, default_value_t = String::from("gemini-2.5-flash-lite"))]
    pub model: String,

    /// 自定义API端点URL
    #[arg(short, long)]
    pub url: Option<String>,

    /// 用于身份验证的API密钥（会根据模型自动选择环境变量）
    #[arg(short, long)]
    pub key: Option<String>,

    /// 自定义翻译提示词
    #[arg(short, long, default_value_t = String::from(PROMPT))]
    pub prompt: String,

    /// 列出所有支持的模型
    #[arg(long, help = "列出所有支持的模型及其 URL")]
    pub list_models: bool,
}

impl Cli {
    /// 解析命令行参数
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }

    /// 智能获取 API 密钥
    ///
    /// 密钥解析优先级：
    /// 1. 如果用户通过 -k/--key 参数提供了密钥，直接返回该密钥
    /// 2. 否则，调用 config::get_env_var_name_for_model 获取对应的环境变量名
    /// 3. 尝试从该环境变量读取密钥
    /// 4. 如果环境变量不存在或为空，返回友好的错误信息，指导用户设置正确的环境变量
    pub fn resolve_api_key(&self) -> Result<String> {
        // 如果用户通过 -k/--key 参数提供了密钥，直接返回该密钥
        if let Some(key) = &self.key {
            return Ok(key.clone());
        }

        // 否则，调用 config::get_env_var_name_for_model 获取对应的环境变量名
        let env_var_name = config::get_env_var_name_for_model(&self.model)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "无法为模型 '{}' 确定对应的环境变量。\n\n\
                    支持的模型及其环境变量：\n\
                    - OpenAI 模型 (gpt-4, gpt-4o, gpt-3.5-turbo 等): OPENAI_API_KEY\n\
                    - Google Gemini 模型 (gemini-2.5-flash, gemini-1.5-pro 等): GOOGLE_AI_API_KEY\n\n\
                    解决方法：\n\
                    1. 使用支持的模型: transome -m <支持的模型名称> <文本>\n\
                    2. 手动提供 API 密钥: transome -k <your_api_key> -m {} <文本>\n\
                    3. 查看所有支持的模型: transome --list-models",
                    self.model, self.model
                )
            })?;

        // 尝试从该环境变量读取密钥
        match std::env::var(env_var_name) {
            Ok(key) if !key.trim().is_empty() => Ok(key),
            Ok(_) => {
                // 环境变量存在但为空
                bail!(
                    "环境变量 {} 已设置但为空。\n\n\
                    解决方法：\n\
                    1. 设置环境变量: export {}=<your_api_key>\n\
                    2. 或者手动提供密钥: transome -k <your_api_key> -m {} <文本>\n\n\
                    获取 API 密钥的方法：\n\
                    - OpenAI API 密钥: https://platform.openai.com/api-keys\n\
                    - Google AI API 密钥: https://aistudio.google.com/app/apikey",
                    env_var_name,
                    env_var_name,
                    self.model
                )
            }
            Err(_) => {
                // 环境变量不存在
                bail!(
                    "环境变量 {} 未设置。\n\n\
                    解决方法：\n\
                    1. 设置环境变量: export {}=<your_api_key>\n\
                    2. 或者手动提供密钥: transome -k <your_api_key> -m {} <文本>\n\n\
                    获取 API 密钥的方法：\n\
                    - OpenAI API 密钥: https://platform.openai.com/api-keys\n\
                    - Google AI API 密钥: https://aistudio.google.com/app/apikey",
                    env_var_name,
                    env_var_name,
                    self.model
                )
            }
        }
    }

    /// 解析要使用的API URL
    ///
    /// URL解析优先级：
    /// 1. 使用自定义URL（如果通过 --url 参数提供）
    /// 2. 从配置中查找模型的默认URL
    /// 3. 如果找不到模型则返回错误
    pub fn resolve_url(&self) -> Result<String> {
        if let Some(url) = &self.url {
            Ok(url.clone())
        } else if let Some(url) = config::get_model_url(&self.model) {
            Ok(url)
        } else {
            bail!("{}", config::create_model_error_message(&self.model));
        }
    }

    /// 显示所有支持的模型
    pub fn list_all_models() {
        config::list_models();
    }

    /// 验证必填字段配置
    ///
    /// 验证规则：
    /// - 非列表模式时，文本输入必填
    /// - 文本输入不能为空或仅包含空白字符
    /// - 验证API密钥是否可用（调用 resolve_api_key 方法）
    /// - 必须支持所选模型（除非提供了自定义URL）
    pub fn validate(&self) -> Result<()> {
        // 如果只是列出模型则跳过验证
        if self.list_models {
            return Ok(());
        }

        // 验证文本输入
        match &self.text {
            Some(text) if text.trim().is_empty() => {
                bail!(
                    "要翻译的文本不能为空\n\n\
                    使用方法: transome [选项] <文本>\n\n\
                    获取更多信息，使用: transome --help"
                );
            }
            Some(_) => {} // 有效的非空文本
            None => {
                bail!(
                    "要翻译的文本是必需的\n\n\
                    使用方法: transome [选项] <文本>\n\n\
                    获取更多信息，使用: transome --help"
                );
            }
        }

        // 验证API密钥是否可用
        self.resolve_api_key().map_err(|e| {
            anyhow::anyhow!(
                "API 密钥验证失败：{}\n\n\
                请确保为所选模型设置了正确的环境变量或通过 -k 参数提供密钥。",
                e
            )
        })?;

        // 验证模型（仅在未提供自定义URL时）
        if self.url.is_none() && !config::is_model_supported(&self.model) {
            bail!("{}", config::create_model_error_message(&self.model));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// 创建基本 CLI 实例的辅助函数，包含必需字段
    fn create_test_cli(model: &str) -> Cli {
        Cli {
            text: Some("test text".to_string()),
            model: model.to_string(),
            url: None,
            key: None,
            prompt: "test prompt".to_string(),
            list_models: false,
        }
    }

    /// 创建带有自定义密钥的 CLI 实例的辅助函数
    fn create_test_cli_with_key(model: &str, key: &str) -> Cli {
        Cli {
            text: Some("test text".to_string()),
            model: model.to_string(),
            url: None,
            key: Some(key.to_string()),
            prompt: "test prompt".to_string(),
            list_models: false,
        }
    }

    /// 临时设置环境变量的辅助函数
    fn with_env_var<T, F>(key: &str, value: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let old_value = env::var(key).ok();
        unsafe {
            env::set_var(key, value);
        }
        let result = f();

        // 恢复原始值
        unsafe {
            match old_value {
                Some(val) => env::set_var(key, val),
                None => env::remove_var(key),
            }
        }

        result
    }

    /// 临时移除环境变量的辅助函数
    fn without_env_var<T, F>(key: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let old_value = env::var(key).ok();
        unsafe {
            env::remove_var(key);
        }
        let result = f();

        // 恢复原始值
        unsafe {
            if let Some(val) = old_value {
                env::set_var(key, val);
            }
        }

        result
    }

    #[test]
    fn test_手动密钥优先级() {
        // 测试手动提供的密钥具有最高优先级
        let cli = create_test_cli_with_key("gpt-4", "manual-key-123");

        // 即使设置了环境变量，也应该使用手动密钥
        with_env_var("OPENAI_API_KEY", "env-key-456", || {
            let result = cli.resolve_api_key().unwrap();
            assert_eq!(result, "manual-key-123");
        });
    }

    #[test]
    fn test_resolve_api_key_openai_env_var() {
        let cli = create_test_cli("gpt-4");

        with_env_var("OPENAI_API_KEY", "openai-test-key", || {
            let result = cli.resolve_api_key().unwrap();
            assert_eq!(result, "openai-test-key");
        });
    }

    #[test]
    fn test_resolve_api_key_google_env_var() {
        let cli = create_test_cli("gemini-2.5-flash");

        with_env_var("GOOGLE_AI_API_KEY", "google-test-key", || {
            let result = cli.resolve_api_key().unwrap();
            assert_eq!(result, "google-test-key");
        });
    }

    #[test]
    fn test_resolve_api_key_different_openai_models() {
        let openai_models = vec![
            "gpt-4",
            "gpt-4o",
            "gpt-3.5-turbo",
            "gpt-4-turbo",
            "gpt-4o-mini",
        ];

        for model in openai_models {
            let cli = create_test_cli(model);

            with_env_var("OPENAI_API_KEY", "openai-key", || {
                let result = cli.resolve_api_key().unwrap();
                assert_eq!(result, "openai-key", "模型 {} 失败", model);
            });
        }
    }

    #[test]
    fn test_resolve_api_key_different_gemini_models() {
        let gemini_models = vec![
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
        ];

        for model in gemini_models {
            let cli = create_test_cli(model);

            with_env_var("GOOGLE_AI_API_KEY", "google-key", || {
                let result = cli.resolve_api_key().unwrap();
                assert_eq!(result, "google-key", "模型 {} 失败", model);
            });
        }
    }

    #[test]
    fn test_resolve_api_key_unsupported_model_error() {
        let cli = create_test_cli("unsupported-model");

        let result = cli.resolve_api_key();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("无法为模型 'unsupported-model' 确定对应的环境变量"));
        assert!(error_msg.contains("OpenAI 模型"));
        assert!(error_msg.contains("Google Gemini 模型"));
        assert!(error_msg.contains("OPENAI_API_KEY"));
        assert!(error_msg.contains("GOOGLE_AI_API_KEY"));
    }

    #[test]
    fn test_resolve_api_key_env_var_not_set() {
        let cli = create_test_cli("gpt-4");

        without_env_var("OPENAI_API_KEY", || {
            let result = cli.resolve_api_key();
            assert!(result.is_err());

            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("环境变量 OPENAI_API_KEY 未设置"));
            assert!(error_msg.contains("export OPENAI_API_KEY=<your_api_key>"));
            assert!(error_msg.contains("transome -k <your_api_key>"));
            assert!(error_msg.contains("https://platform.openai.com/api-keys"));
        });
    }

    #[test]
    fn test_resolve_api_key_env_var_empty() {
        let cli = create_test_cli("gpt-4");

        with_env_var("OPENAI_API_KEY", "", || {
            let result = cli.resolve_api_key();
            assert!(result.is_err());

            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("环境变量 OPENAI_API_KEY 已设置但为空"));
        });
    }

    #[test]
    fn test_resolve_api_key_env_var_whitespace_only() {
        let cli = create_test_cli("gpt-4");

        with_env_var("OPENAI_API_KEY", "   \t\n   ", || {
            let result = cli.resolve_api_key();
            assert!(result.is_err());

            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("环境变量 OPENAI_API_KEY 已设置但为空"));
        });
    }

    #[test]
    fn test_resolve_api_key_gemini_env_var_not_set() {
        let cli = create_test_cli("gemini-2.5-flash");

        without_env_var("GOOGLE_AI_API_KEY", || {
            let result = cli.resolve_api_key();
            assert!(result.is_err());

            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("环境变量 GOOGLE_AI_API_KEY 未设置"));
            assert!(error_msg.contains("https://aistudio.google.com/app/apikey"));
        });
    }

    #[test]
    fn test_resolve_url_custom_url_priority() {
        let mut cli = create_test_cli("gpt-4");
        cli.url = Some("https://custom.api.com/v1".to_string());

        let result = cli.resolve_url().unwrap();
        assert_eq!(result, "https://custom.api.com/v1");
    }

    #[test]
    fn test_resolve_url_model_default() {
        let cli = create_test_cli("gpt-4");

        let result = cli.resolve_url().unwrap();
        assert_eq!(result, "https://api.openai.com/v1");
    }

    #[test]
    fn test_resolve_url_unsupported_model() {
        let cli = create_test_cli("unsupported-model");

        let result = cli.resolve_url();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("找不到模型 'unsupported-model'"));
    }

    #[test]
    fn test_manual_key_overrides_env_different_providers() {
        // 测试在设置 Google 环境变量时使用手动密钥的 OpenAI 模型
        let cli = create_test_cli_with_key("gpt-4", "manual-openai-key");

        with_env_var("GOOGLE_AI_API_KEY", "google-env-key", || {
            let result = cli.resolve_api_key().unwrap();
            assert_eq!(result, "manual-openai-key");
        });

        // 测试在设置 OpenAI 环境变量时使用手动密钥的 Gemini 模型
        let cli = create_test_cli_with_key("gemini-2.5-flash", "manual-gemini-key");

        with_env_var("OPENAI_API_KEY", "openai-env-key", || {
            let result = cli.resolve_api_key().unwrap();
            assert_eq!(result, "manual-gemini-key");
        });
    }

    #[test]
    fn test_automatic_env_var_selection() {
        // 设置两个环境变量
        with_env_var("OPENAI_API_KEY", "openai-key", || {
            with_env_var("GOOGLE_AI_API_KEY", "google-key", || {
                // OpenAI 模型应该使用 OPENAI_API_KEY
                let openai_cli = create_test_cli("gpt-4");
                let result = openai_cli.resolve_api_key().unwrap();
                assert_eq!(result, "openai-key");

                // Gemini 模型应该使用 GOOGLE_AI_API_KEY
                let gemini_cli = create_test_cli("gemini-2.5-flash");
                let result = gemini_cli.resolve_api_key().unwrap();
                assert_eq!(result, "google-key");
            });
        });
    }

    #[test]
    fn test_validate_list_models_skips_validation() {
        let mut cli = create_test_cli("unsupported-model");
        cli.list_models = true;
        cli.text = None; // 这通常会导致验证失败

        // 不应该失败，因为 list_models 为 true
        let result = cli.validate();
        assert!(result.is_ok());
    }
}
