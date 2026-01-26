use std::io::{self, Write};
use crate::ai::OllamaClient;
use anyhow::Result;
pub async fn run_ai_chat(internal: bool) -> Result<()> {
    // Wrap the chat loop to catch errors and keep window open
    if let Err(e) = chat_loop(internal).await {
        let red = "\x1b[31m";
        let reset = "\x1b[0m";
        println!("\n{}❌ AI Chat Crashed: {}{}", red, e, reset);
        println!("Press Enter to close this window...");
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
    }

    Ok(())
}

async fn chat_loop(internal: bool) -> Result<()> {
    let teal = "\x1b[38;2;110;209;195m";
    let reset = "\x1b[0m";
    let gray = "\x1b[90m";

    if internal {
        // Clear screen and show a minimal header for sidebar
        print!("\x1b[2J\x1b[H"); 
    }
    
    println!("{}🐕 AI Chat Sidebar - Linked to Shell Session{}", teal, reset);
    println!("{}Type your message below. /quit to exit.{}", gray, reset);
    println!("---");

    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
    let client = OllamaClient::new(model);
    
    let mut history: Vec<(String, String)> = Vec::new();
    let my_pid = std::process::id();
    let context_path = format!("/tmp/chev-context-{}.txt", my_pid);

    loop {
        print!("{}YOU:{} ", teal, reset);
        io::stdout().flush()?;
        
        let mut input = String::new();
        let bytes = io::stdin().read_line(&mut input)?;
        if bytes == 0 {
             // EOF detected
             break; 
        }
        let input = input.trim();
        
        if input == "/quit" || input == "/exit" {
            break;
        }
        
        if input == "/clear" {
            print!("\x1b[2J\x1b[H");
            println!("{}🐕 AI Chat Sidebar - Linked to Shell Session{}", teal, reset);
            continue;
        }
        
        if input.is_empty() {
            continue;
        }

        println!("{}🐕 Thinking... Contextualizing shell session...{}", gray, reset);

        // Fetch fresh context from Rio
        crate::ui::protocol::send_rio(crate::ui::protocol::RioAction::RequestHistory);
        // Small delay for file write
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        let shell_context = std::fs::read_to_string(&context_path).unwrap_or_else(|_| "No shell context available.".to_string());

        // Simple context building
        let mut prompt = format!(
            "You are a helpful AI assistant integrated into the Chev Shell. \
             Below is the current visible content of the user's terminal session (sibling pane).\n\
             --- TERMINAL CONTEXT ---\n\
             {}\n\
             --- END CONTEXT ---\n\n",
            shell_context
        );

        for (u, a) in &history {
            prompt.push_str(&format!("User: {}\nAssistant: {}\n", u, a));
        }
        prompt.push_str(&format!("User: {}", input));

        match client.generate(prompt, false).await {
            Ok(response) => {
                println!("{}🤖 AI:{} {}", teal, reset, response);
                history.push((input.to_string(), response));
                // Keep history manageable
                if history.len() > 10 {
                    history.remove(0);
                }
            }
            Err(e) => {
                println!("{}❌ Error:{} {}", teal, reset, e);
            }
        }
        println!();
    }
    Ok(())
}
