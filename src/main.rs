//! Transome 命令行程序入口

use anyhow::Result;
use transome::{Cli, Translator};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run().await {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

/// 主程序逻辑
async fn run() -> Result<()> {
    // 解析命令行参数
    let args = Cli::parse();
    
    // 处理模型列表请求
    if args.list_models {
        handle_list_models();
        return Ok(());
    }
    
    // 执行全面验证
    args.validate()?;
    
    // 提取验证后的文本 - 验证后已确保安全
    let text = args.text.as_ref().unwrap();
    
    // 从模型或自定义URL解析API地址
    let url = args.resolve_url()?;
    
    // 执行翻译并提供更好的错误上下文
    let result = execute_translation(&args, text, &url).await
        .map_err(|e| {
            anyhow::anyhow!(
                "翻译失败: {}\n\n\
                故障排除建议:\n\
                - 验证您的 API 密钥是否正确\n\
                - 检查您的网络连接\n\
                - 使用 --model <MODEL> 尝试不同的模型\n\
                - 使用 --list-models 查看可用选项", e
            )
        })?;
    
    println!("{}", result);
    
    Ok(())
}

/// 处理 --list-models 命令
fn handle_list_models() {
    Cli::list_all_models();
}


/// 执行翻译
async fn execute_translation(args: &Cli, text: &str, url: &str) -> Result<String> {
    // 使用解析后的配置创建翻译器实例
    let translator = Translator::new(
        args.key.clone(), 
        url.to_string(), 
        args.model.clone()
    );
    
    // 使用自定义或默认提示执行翻译
    translator.translate(text, Some(&args.prompt)).await
}
