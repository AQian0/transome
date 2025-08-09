mod cli;

use anyhow::Result;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};
use cli::Cli;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    let config = OpenAIConfig::new()
        .with_api_key(args.key)
        .with_api_base(args.url);
    let client = Client::with_config(config);
    let req = CreateChatCompletionRequestArgs::default()
        .model(args.model)
        .messages([
            ChatCompletionRequestUserMessageArgs::default()
                .content(args.prompt)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(args.text)
                .build()?
                .into(),
        ])
        .build()?;

    match client.chat().create(req).await {
        Ok(res) => {
            for choice in res.choices {
                if let Some(content) = choice.message.content {
                    println!("{}", content);
                }
            }
        }
        Err(e) => {
            return Err(e.into());
        }
    }
    Ok(())
}
