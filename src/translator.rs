//! 翻译功能核心实现

use anyhow::{anyhow, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};

/// 默认的双向中英文翻译提示词
pub const PROMPT: &str = "你是一个极简翻译工具，接下来我将输入一段内容，请按照以下规则将它翻译：1、如果输入内容是中文则翻译成英文，反之亦然。2、仅输出翻译后的内容，不要携带其他内容。3、如果翻译后的内容是单个词语，则首字母不需要大写。";

/// 翻译器结构体
#[derive(Debug, Clone)]
pub struct Translator {
    client: Client<OpenAIConfig>,
    model: String,
}

impl Translator {
    /// 创建新的翻译器实例
    pub fn new(api_key: String, api_base: String, model: String) -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base);
        let client = Client::with_config(config);
        
        Self {
            client,
            model,
        }
    }

    /// 执行文本翻译
    pub async fn translate(&self, text: &str, prompt: Option<&str>) -> Result<String> {
        // 验证输入文本
        if text.trim().is_empty() {
            return Err(anyhow!(
                "Translation text cannot be empty.\n\n\
                Please provide non-empty text to translate."
            ));
        }

        let prompt_text = prompt.unwrap_or(PROMPT);
        
        // 构建聊天完成请求
        let req = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                // 系统/指令消息
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt_text)
                    .build()
                    .map_err(|e| anyhow!(
                        "Failed to build prompt message: {}\n\n\
                        This is likely due to an invalid prompt format. \
                        Please check your prompt content.", e
                    ))?
                    .into(),
                // 用户消息包含待翻译文本
                ChatCompletionRequestUserMessageArgs::default()
                    .content(text)
                    .build()
                    .map_err(|e| anyhow!(
                        "Failed to build user message: {}\n\n\
                        This is likely due to invalid text content. \
                        Please check your input text.", e
                    ))?
                    .into(),
            ])
            .build()
            .map_err(|e| anyhow!(
                "Failed to build chat request: {}\n\n\
                This may be due to invalid model name or request parameters. \
                Please check your configuration.", e
            ))?;

        // 发送请求并处理响应
        let response = self.client.chat().create(req).await
            .map_err(|e| {
                let error_str = e.to_string();
                if error_str.contains("401") || error_str.contains("authentication") {
                    anyhow!(
                        "Authentication failed: {}\n\n\
                        Please check that your API key is correct and has the necessary permissions.\n\
                        For OpenAI: Ensure your API key starts with 'sk-'\n\
                        For Gemini: Ensure you're using a valid Google AI API key", e
                    )
                } else if error_str.contains("404") || error_str.contains("not found") {
                    anyhow!(
                        "Model or endpoint not found: {}\n\n\
                        Please verify that:\n\
                        - The model name '{}' is correct and available\n\
                        - The API endpoint is accessible\n\
                        - You have permission to use this model", e, self.model
                    )
                } else if error_str.contains("429") || error_str.contains("rate limit") {
                    anyhow!(
                        "Rate limit exceeded: {}\n\n\
                        Please wait a moment before trying again. \
                        Consider upgrading your API plan if this happens frequently.", e
                    )
                } else if error_str.contains("timeout") || error_str.contains("connection") {
                    anyhow!(
                        "Network error: {}\n\n\
                        Please check your internet connection and try again.\n\
                        If the problem persists, the API service may be temporarily unavailable.", e
                    )
                } else {
                    anyhow!(
                        "API request failed: {}\n\n\
                        Please check your network connection, API key, and model name.\n\
                        If the problem persists, the AI service may be temporarily unavailable.", e
                    )
                }
            })?;

        // 验证响应结构
        if response.choices.is_empty() {
            return Err(anyhow!(
                "No translation results in API response.\n\n\
                This may indicate an issue with the AI model or service. \
                Please try again or use a different model."
            ));
        }

        // 提取并合并所有响应内容
        let mut result = String::new();
        for choice in response.choices {
            if let Some(content) = choice.message.content {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&content);
            }
        }

        if result.trim().is_empty() {
            return Err(anyhow!(
                "Translation result is empty.\n\n\
                The AI model returned an empty response. This may be due to:\n\
                - The input text being unclear or untranslatable\n\
                - Issues with the model or prompt\n\
                - Temporary service problems\n\n\
                Please try again with different text or check the model status."
            ));
        }

        Ok(result.trim().to_string())
    }
    
    /// 获取当前配置的模型名称
    pub fn model_name(&self) -> &str {
        &self.model
    }
    
}
