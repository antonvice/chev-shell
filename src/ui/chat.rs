use std::io::{self, Write};
use crate::ai::OllamaClient;
use anyhow::Result;

pub async fn run_ai_chat(internal: bool) -> Result<()> {
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

    loop {
        print!("{}YOU:{} ", teal, reset);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
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

        println!("{}🐕 Thinking...{}", gray, reset);

        // Simple context building
        let mut prompt = String::new();
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
