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
                "翻译文本不能为空\n\n\
                请提供非空的文本进行翻译"
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
                        "构建提示消息失败: {}\n\n\
                        这可能是由于提示格式无效导致的。\
                        请检查您的提示内容。", e
                    ))?
                    .into(),
                // 用户消息包含待翻译文本
                ChatCompletionRequestUserMessageArgs::default()
                    .content(text)
                    .build()
                    .map_err(|e| anyhow!(
                        "构建用户消息失败: {}\n\n\
                        这可能是由于无效的文本内容导致的。\
                        请检查您的输入文本。", e
                    ))?
                    .into(),
            ])
            .build()
            .map_err(|e| anyhow!(
                "构建聊天请求失败: {}\n\n\
                这可能是由于无效的模型名称或请求参数导致的。\
                请检查您的配置。", e
            ))?;

        // 发送请求并处理响应
        let response = self.client.chat().create(req).await
            .map_err(|e| {
                let error_str = e.to_string();
                if error_str.contains("401") || error_str.contains("authentication") {
                    anyhow!(
                        "认证失败: {}\n\n\
                        请检查您的 API 密钥是否正确并具有必要的权限。\n\
                        对于 OpenAI: 确保您的 API 密钥以 'sk-' 开头\n\
                        对于 Gemini: 确保您使用的是有效的 Google AI API 密钥", e
                    )
                } else if error_str.contains("404") || error_str.contains("not found") {
                    anyhow!(
                        "找不到模型或端点: {}\n\n\
                        请验证以下内容:\n\
                        - 模型名称 '{}' 是否正确且可用\n\
                        - API 端点是否可访问\n\
                        - 您是否有权使用此模型", e, self.model
                    )
                } else if error_str.contains("429") || error_str.contains("rate limit") {
                    anyhow!(
                        "超出频率限制: {}\n\n\
                        请稍后再试。\
                        如果频繁出现这种情况，请考虑升级您的 API 套餐。", e
                    )
                } else if error_str.contains("timeout") || error_str.contains("connection") {
                    anyhow!(
                        "网络错误: {}\n\n\
                        请检查您的网络连接并重试。\n\
                        如果问题持续，API 服务可能暂时不可用。", e
                    )
                } else {
                    anyhow!(
                        "API 请求失败: {}\n\n\
                        请检查您的网络连接、API 密钥和模型名称。\n\
                        如果问题持续，AI 服务可能暂时不可用。", e
                    )
                }
            })?;

        // 验证响应结构
        if response.choices.is_empty() {
            return Err(anyhow!(
                "API 响应中没有翻译结果\n\n\
                这可能表明 AI 模型或服务存在问题。\
                请重试或使用不同的模型。"
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
                "翻译结果为空\n\n\
                AI 模型返回了空响应。这可能是由于:\n\
                - 输入文本不清晰或无法翻译\n\
                - 模型或提示存在问题\n\
                - 服务暂时问题\n\n\
                请使用不同的文本重试或检查模型状态。"
            ));
        }

        Ok(result.trim().to_string())
    }
    
    /// 获取当前配置的模型名称
    pub fn model_name(&self) -> &str {
        &self.model
    }
    
}
