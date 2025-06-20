use clap::Parser;
use std::env;

const PROMPT: &str = "你是一个极简翻译工具，接下来我将输入一段内容，请按照以下规则将它翻译：1、如果输入内容是中文则翻译成英文，反之亦然。2、仅输出翻译后的内容，不要携带其他内容。3、如果翻译后的内容是单个词语，则首字母不需要大写。";

#[derive(Parser)]
#[command(name = "transome")]
#[command(version = "v0.1.0")]
#[command(about = "A simple command line translation tool", long_about = None)]
pub struct Cli {
    pub text: String,
    #[arg(short, long, default_value_t = String::from("gemini-2.5-flash-lite-preview-06-17"))]
    pub model: String,
    #[arg(short, long)]
    pub key: Option<String>,
    #[arg(short, long, default_value_t = String::from(PROMPT))]
    pub prompt: String,
}

pub struct Args {
    pub text: String,
    pub model: String,
    pub key: String,
    pub prompt: String,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}

impl From<Cli> for Args {
    fn from(value: Cli) -> Self {
        let key = match value.key {
            Some(key) => key,
            None => match env::var("GOOGLE_AI_API_KEY") {
                Ok(key) => key,
                Err(e) => {
                    eprintln!("获取Key错误: {}", e);
                    std::process::exit(1);
                }
            },
        };
        Self {
            text: value.text,
            key,
            model: value.model,
            prompt: value.prompt,
        }
    }
}
