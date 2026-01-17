use crate::ai::OllamaClient;
use anyhow::Result;
use std::io;

pub async fn run_ai_browser(url: &str) -> Result<()> {
    let teal = "\x1b[38;2;110;209;195m";
    let reset = "\x1b[0m";
    let gray = "\x1b[90m";
    
    // Clear screen and show header
    print!("\x1b[2J\x1b[H"); 
    println!("{}🐕 AI Browser - Researching...{}", teal, reset);
    println!("{}URL: {}{}", gray, url, reset);
    println!("---");

    // Fetch URL
    println!("{}📡 Fetching page content...{}", gray, reset);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let res = match client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await {
            Ok(r) => r,
            Err(e) => {
                println!("{}❌ Failed to reach URL:{} {}", teal, reset, e);
                wait_for_exit();
                return Ok(());
            }
        };
    
    if !res.status().is_success() {
        println!("{}❌ HTTP Error:{} {}", teal, reset, res.status());
        wait_for_exit();
        return Ok(());
    }

    let body = res.text().await?;
    
    // Extract text and filter out boilerplate
    let text = extract_clean_text(&body);
    
    if text.is_empty() {
        println!("{}❌ Could not extract meaningful text from this page.{}", teal, reset);
        wait_for_exit();
        return Ok(());
    }

    println!("{}🐕 Analyzing with AI ({} chars extracted)...{}", gray, text.len(), reset);

    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
    let ai_client = OllamaClient::new(model);
    
    let prompt = format!(
        "You are an AI browser assistant. Below is the text content extracted from a webpage ({}). \
         Please provide a concise, structured summary of the key information, main points, and any important details. \
         Format your response with clear headings if necessary.\n\n\
         --- WEBPAGE CONTENT ---\n\
         {}\n\
         --- END CONTENT ---",
        url, 
        if text.len() > 10000 { &text[..10000] } else { &text } // Truncate to avoid context limit issues
    );

    match ai_client.generate(prompt, false).await {
        Ok(response) => {
            println!("{}🤖 Summary for {}:{}", teal, url, reset);
            println!("\n{}\n", response);
        }
        Err(e) => {
            println!("{}❌ AI Error:{} {}", teal, reset, e);
        }
    }
    
    wait_for_exit();

    Ok(())
}

fn wait_for_exit() {
    let gray = "\x1b[90m";
    let reset = "\x1b[0m";
    println!("\n{}Press Enter to close this sidebar...{}", gray, reset);
    let mut dummy = String::new();
    let _ = io::stdin().read_line(&mut dummy);
}

fn extract_clean_text(html: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut in_script_or_style = false;
    let mut tag_content = String::new();
    
    let mut skip_stack: usize = 0;

    for c in html.chars() {
        if c == '<' {
            in_tag = true;
            tag_content.clear();
            continue;
        }
        if c == '>' {
            in_tag = false;
            let tag = tag_content.to_lowercase();
            
            if tag.starts_with("script") || tag.starts_with("style") {
                in_script_or_style = true;
                skip_stack += 1;
            } else if tag == "/script" || tag == "/style" {
                skip_stack = skip_stack.saturating_sub(1);
                if skip_stack == 0 {
                    in_script_or_style = false;
                }
            } else if tag == "p" || tag == "div" || tag == "br" || tag == "h1" || tag == "h2" || tag == "h3" || tag == "li" {
                output.push('\n');
            } else if tag == "/p" || tag == "/div" || tag == "/h1" || tag == "/h2" || tag == "/h3" || tag == "/li" {
                 output.push('\n');
            }
            continue;
        }
        
        if in_tag {
            tag_content.push(c);
        } else if !in_script_or_style {
            output.push(c);
        }
    }
    
    // Post-process: collapse multiple newlines and trim
    let lines: Vec<&str> = output.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    
    lines.join("\n")
}
