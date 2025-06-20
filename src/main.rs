mod cli;

use anyhow::{Result, anyhow};
use cli::Cli;
use reqwest::{self, StatusCode};
use serde::{Deserialize, Serialize};

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

fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    let contents = GeminiReq::from(vec![Content::from(vec![Part::from(format!(
        "{}\n\n{}",
        args.prompt, args.text
    ))])]);
    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        args.model
    );
    let res = client
        .post(&url)
        .query(&[("key", args.key)])
        .json(&contents)
        .send()
        .map_err(|_| anyhow!("请求发送失败"))?;
    match res.status() {
        StatusCode::OK => {
            let gemini_res: GeminiRes = res.json().map_err(|_| anyhow!("解析请求体失败"))?;
            let first_candidate = gemini_res
                .candidates
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("获取candidate失败"))?;
            let first_part = first_candidate
                .content
                .parts
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("获取part失败"))?;
            println!("{}", first_part.text);
            Ok(())
        }
        status => Err(anyhow!("状态码错误: {}", status,)),
    }
}
