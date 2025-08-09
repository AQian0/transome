use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::{bail, Result};
use clap::Parser;

const PROMPT: &str = "你是一个极简翻译工具，接下来我将输入一段内容，请按照以下规则将它翻译：1、如果输入内容是中文则翻译成英文，反之亦然。2、仅输出翻译后的内容，不要携带其他内容。3、如果翻译后的内容是单个词语，则首字母不需要大写。";

static MODEL_TO_URL: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn get_model_to_url() -> &'static HashMap<&'static str, &'static str> {
    MODEL_TO_URL.get_or_init(|| {
        HashMap::from([
            // Gemini models (keep existing mappings)
            ("gemini-2.5-pro", "https://generativelanguage.googleapis.com/v1beta/openai"),
            ("gemini-2.5-flash", "https://generativelanguage.googleapis.com/v1beta/openai"),
            ("gemini-2.5-flash-lite", "https://generativelanguage.googleapis.com/v1beta/openai"),
            ("gemini-1.5-pro", "https://generativelanguage.googleapis.com/v1beta/openai"),
            ("gemini-1.5-flash", "https://generativelanguage.googleapis.com/v1beta/openai"),
            
            // OpenAI models
            ("gpt-4", "https://api.openai.com/v1"),
            ("gpt-4-turbo", "https://api.openai.com/v1"),
            ("gpt-4o", "https://api.openai.com/v1"),
            ("gpt-4o-mini", "https://api.openai.com/v1"),
            ("gpt-3.5-turbo", "https://api.openai.com/v1"),
            ("gpt-3.5-turbo-16k", "https://api.openai.com/v1"),
        ])
    })
}

#[derive(Parser)]
#[command(name = "transome")]
#[command(version = "v0.1.0")]
#[command(about = "A simple command line translation tool", long_about = None)]
pub struct Cli {
    pub text: Option<String>,
    #[arg(short, long, default_value_t = String::from("gemini-2.5-flash-lite"))]
    pub model: String,
    #[arg(short, long)]
    pub url: Option<String>,
    #[arg(short, long, env = "GOOGLE_AI_API_KEY")]
    pub key: String,
    #[arg(short, long, default_value_t = String::from(PROMPT))]
    pub prompt: String,
    #[arg(long, help = "List all supported models and their URLs")]
    pub list_models: bool,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }

    pub fn list_all_models() {
        let model_to_url = get_model_to_url();
        println!("\nSupported models:");
        
        // Group models by provider for better organization
        let mut providers: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
        
        for (&model, &url) in model_to_url.iter() {
            let provider = if url.contains("generativelanguage.googleapis.com") {
                "Google Gemini"
            } else if url.contains("api.openai.com") {
                "OpenAI"
            } else {
                "Other"
            };
            providers.entry(provider).or_default().push((model, url));
        }
        
        // Sort providers alphabetically and display
        let mut sorted_providers: Vec<_> = providers.iter().collect();
        sorted_providers.sort_by_key(|&(name, _)| name);
        
        for (provider, models) in sorted_providers {
            println!("\n{} ({}):", provider, models[0].1);
            let mut sorted_models = models.clone();
            sorted_models.sort_by_key(|&(model, _)| model);
            for (model, _) in sorted_models {
                println!("  - {}", model);
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

    pub fn resolve_url(&self) -> Result<String> {
        if let Some(url) = &self.url {
            return Ok(url.clone());
        }

        let model_to_url = get_model_to_url();
        if let Some(url) = model_to_url.get(self.model.as_str()) {
            Ok(url.to_string())
        } else {
            // Create a better formatted error message with grouped models
            let mut providers: HashMap<&str, Vec<&str>> = HashMap::new();
            
            for (&model, &url) in model_to_url.iter() {
                let provider = if url.contains("generativelanguage.googleapis.com") {
                    "Google Gemini"
                } else if url.contains("api.openai.com") {
                    "OpenAI"
                } else if url.contains("api.anthropic.com") {
                    "Anthropic"
                } else if url.contains("api.cohere.ai") {
                    "Cohere"
                } else if url.contains("api.mistral.ai") {
                    "Mistral"
                } else if url.contains("api.perplexity.ai") {
                    "Perplexity"
                } else {
                    "Other"
                };
                providers.entry(provider).or_default().push(model);
            }
            
            let mut error_msg = format!("Model '{}' not found.\n\nSupported models:", self.model);
            
            let mut sorted_providers: Vec<_> = providers.iter().collect();
            sorted_providers.sort_by_key(|&(name, _)| name);
            
            for (provider, models) in sorted_providers {
                error_msg.push_str(&format!("\n\n{}: {}", provider, {
                    let mut sorted_models = models.clone();
                    sorted_models.sort();
                    sorted_models.join(", ")
                }));
            }
            
            error_msg.push_str("\n\nUsage:");
            error_msg.push_str("\n  Use one of the supported models: transome -m <MODEL> \"<text>\"");
            error_msg.push_str("\n  Or provide a custom URL: transome -u <URL> -m <MODEL> \"<text>\"");
            error_msg.push_str("\n  List all models: transome --list-models");
            
            bail!("{}", error_msg);
        }
    }
}
