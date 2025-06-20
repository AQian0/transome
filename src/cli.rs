use clap::Parser;

const PROMPT: &str = "你是一个极简翻译工具，接下来我将输入一段内容，请按照以下规则将它翻译：1、如果输入内容是中文则翻译成英文，反之亦然。2、仅输出翻译后的内容，不要携带其他内容。3、如果翻译后的内容是单个词语，则首字母不需要大写。";

#[derive(Parser)]
#[command(name = "transome")]
#[command(version = "v0.1.0")]
#[command(about = "A simple command line translation tool", long_about = None)]
pub struct Cli {
    pub text: String,
    #[arg(short, long, default_value_t = String::from("gemini-2.5-flash-lite-preview-06-17"))]
    pub model: String,
    #[arg(short, long, env = "GOOGLE_AI_API_KEY")]
    pub key: String,
    #[arg(short, long, default_value_t = String::from(PROMPT))]
    pub prompt: String,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
