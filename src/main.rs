mod cli;

use cli::{Args, Cli};
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

fn main() {
    let args: Args = Cli::parse().into();
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
