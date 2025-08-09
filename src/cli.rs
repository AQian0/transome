//! 命令行参数解析模块

use clap::Parser;
use anyhow::{bail, Result};

use crate::config;
use crate::translator::PROMPT;

/// 命令行参数结构体
#[derive(Parser, Debug, Clone)]
#[command(name = "transome")]
#[command(version = "v0.2.0")]
#[command(about = "A simple command line translation tool", long_about = None)]
pub struct Cli {
    /// 要翻译的文本
    pub text: Option<String>,
    
    /// 翻译使用的AI模型
    #[arg(short, long, default_value_t = String::from("gemini-2.5-flash-lite"))]
    pub model: String,
    
    /// 自定义API端点URL
    #[arg(short, long)]
    pub url: Option<String>,
    
    /// 用于身份验证的API密钥
    #[arg(short, long, env = "GOOGLE_AI_API_KEY")]
    pub key: String,
    
    /// 自定义翻译提示词
    #[arg(short, long, default_value_t = String::from(PROMPT))]
    pub prompt: String,
    
    /// 列出所有支持的模型
    #[arg(long, help = "List all supported models and their URLs")]
    pub list_models: bool,
}

impl Cli {
    /// 解析命令行参数
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
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
    /// - 必须提供API密钥
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
                    "Text to translate cannot be empty.\n\n\
                    Usage: transome [OPTIONS] <TEXT>\n\n\
                    For more information, use: transome --help"
                );
            },
            Some(_) => {}, // 有效的非空文本
            None => {
                bail!(
                    "Text to translate is required.\n\n\
                    Usage: transome [OPTIONS] <TEXT>\n\n\
                    For more information, use: transome --help"
                );
            }
        }
        
        // 验证API密钥
        if self.key.trim().is_empty() {
            bail!(
                "API key is required. Please provide it via:\n\n\
                - Command line: transome -k <API_KEY> [OPTIONS] <TEXT>\n\
                - Environment variable: export GOOGLE_AI_API_KEY=<API_KEY>\n\n\
                For more information, use: transome --help"
            );
        }
        
        // 验证模型（仅在未提供自定义URL时）
        if self.url.is_none() && !config::is_model_supported(&self.model) {
            bail!("{}", config::create_model_error_message(&self.model));
        }
        
        Ok(())
    }
}
