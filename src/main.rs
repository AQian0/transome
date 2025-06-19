use clap::Parser;
use reqwest::{self, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Parser)]
#[command(name = "transome")]
#[command(version = "v0.1.0")]
#[command(about = "A simple command line translation tool", long_about = None)]
struct Cli {
    text: String,
    #[arg(short, long, default_value_t = String::from("gemini-2.5-flash-lite-preview-06-17"))]
    model: String,
    #[arg(short, long)]
    key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiRes {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Serialize)]
struct GeminiReq {
    contents: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

impl From<String> for Part {
    fn from(value: String) -> Self {
        Self { text: value }
    }
}

impl From<Vec<Part>> for Content {
    fn from(value: Vec<Part>) -> Self {
        Self { parts: value }
    }
}

impl From<Vec<Content>> for GeminiReq {
    fn from(value: Vec<Content>) -> Self {
        Self { contents: value }
    }
}

fn main() {
    let cli = Cli::parse();
    let prompt = String::from(
        "你是一个极简翻译工具，接下来我将输入一段内容，请按照以下规则将它翻译：1、如果输入内容是中文则翻译成英文，反之亦然。2、仅输出翻译后的内容，不要携带其他内容。3、如果翻译后的内容是单个词语，则首字母不需要大写。",
    );
    let contents = GeminiReq::from(vec![Content::from(vec![Part::from(format!(
        "{}\n\n{}",
        prompt, cli.text
    ))])]);
    let client = reqwest::blocking::Client::new();
    let api_key = match cli.key {
        Some(key) => key,
        None => match env::var("GOOGLE_AI_API_KEY") {
            Ok(key) => key,
            Err(e) => {
                eprintln!("获取Key错误: {}", e);
                std::process::exit(1);
            }
        },
    };
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        cli.model
    );
    let res = client
        .post(&url)
        .query(&[("key", api_key)])
        .json(&contents)
        .send();
    match res {
        Ok(res) => match res.status() {
            StatusCode::OK => {
                let text = res
                    .json::<GeminiRes>()
                    .unwrap()
                    .candidates
                    .first()
                    .unwrap()
                    .content
                    .parts
                    .first()
                    .unwrap()
                    .text
                    .clone();
                println!("{}", text);
            }
            _ => {
                eprintln!("状态码错误");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("响应错误: {}", e);
            std::process::exit(1);
        }
    }
}
