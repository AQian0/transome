mod cli;

use anyhow::{bail, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};
use cli::Cli;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    
    // Handle --list-models flag
    if args.list_models {
        Cli::list_all_models();
        return Ok(());
    }
    
    // Check if text is provided
    let text = match &args.text {
        Some(t) => t.clone(),
        None => {
            bail!("Text to translate is required.\n\nUsage:\n  transome [OPTIONS] <TEXT>\n\nFor more information, use: transome --help");
        }
    };
    
    let url = args.resolve_url()?;
    let config = OpenAIConfig::new()
        .with_api_key(args.key)
        .with_api_base(url);
    let client = Client::with_config(config);
    let req = CreateChatCompletionRequestArgs::default()
        .model(args.model)
        .messages([
            ChatCompletionRequestUserMessageArgs::default()
                .content(args.prompt)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(text)
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
            bail!(e);
        }
    }
    Ok(())
}
