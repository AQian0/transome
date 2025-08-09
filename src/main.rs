//! Transome 命令行程序入口

use anyhow::Result;
use transome::{Cli, Translator};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

/// 主程序逻辑
async fn run() -> Result<()> {
    // Parse command line arguments
    let args = Cli::parse();
    
    // Handle model listing request early
    if args.list_models {
        handle_list_models();
        return Ok(());
    }
    
    // Perform comprehensive validation
    args.validate()?;
    
    // Extract validated text - we know it's safe after validation
    let text = args.text.as_ref().unwrap();
    
    // Resolve API URL from model or custom URL
    let url = args.resolve_url()?;
    
    // Execute translation with better error context
    let result = execute_translation(&args, text, &url).await
        .map_err(|e| {
            anyhow::anyhow!(
                "Translation failed: {}\n\n\
                Troubleshooting tips:\n\
                - Verify your API key is correct\n\
                - Check your internet connection\n\
                - Try a different model with --model <MODEL>\n\
                - Use --list-models to see available options", e
            )
        })?;
    
    // Output the result
    println!("{}", result);
    
    Ok(())
}

/// 处理 --list-models 命令
fn handle_list_models() {
    Cli::list_all_models();
}


/// 执行翻译
async fn execute_translation(args: &Cli, text: &str, url: &str) -> Result<String> {
    // Create translator instance with resolved configuration
    let translator = Translator::new(
        args.key.clone(), 
        url.to_string(), 
        args.model.clone()
    );
    
    // Perform translation with custom or default prompt
    translator.translate(text, Some(&args.prompt)).await
}
