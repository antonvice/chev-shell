use crate::ai::OllamaClient;
use anyhow::Result;
use std::io::{self, Write};

#[derive(Default)]
struct PageMetadata {
    title: String,
    image_url: Option<String>,
    description: String,
}

pub async fn run_ai_browser(url: &str) -> Result<()> {
    let teal = "\x1b[38;2;110;209;195m";
    let reset = "\x1b[0m";
    let gray = "\x1b[90m";
    
    // Clear screen and show header
    print!("\x1b[2J\x1b[H"); 
    println!("{}🐕 AI Browser V2 - Rich Research Mode{}", teal, reset);
    println!("{}URL: {}{}", gray, url, reset);
    println!("---");

    // Fetch URL
    println!("{}📡 Fetching rich content...{}", gray, reset);
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
    
    // Extract text and metadata
    let (text, meta) = extract_content(&body);
    
    if text.is_empty() {
        println!("{}❌ Could not extract meaningful text from this page.{}", teal, reset);
        wait_for_exit();
        return Ok(());
    }

    // Render Mini Card
    print!("\x1b[2J\x1b[H"); 
    render_mini_card(&meta, url);
    println!("---");

    println!("{}🐕 Analyzing with AI...{}", gray, reset);

    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
    let ai_client = OllamaClient::new(model);
    
    let base_prompt = format!(
        "You are an AI browser assistant. Below is the text content extracted from a webpage ({}). \
         Title: {}\n\
         Description: {}\n\n\
         --- WEBPAGE CONTENT ---\n\
         {}\n\
         --- END CONTENT ---",
        url, meta.title, meta.description,
        if text.len() > 10000 { &text[..10000] } else { &text }
    );

    let mut conversation_history = String::new();
    
    // Initial Summary
    let summary_prompt = format!("{}\nPlease provide a concise summary of this page.", base_prompt);
    match ai_client.generate(summary_prompt, false).await {
        Ok(response) => {
            println!("{}🤖 Summary:{}", teal, reset);
            println!("{}\n", response);
            conversation_history.push_str(&format!("Assistant: {}\n", response));
        }
        Err(e) => {
            println!("{}❌ AI Error:{} {}", teal, reset, e);
        }
    }

    // Recursive Q&A Loop
    loop {
        print!("{}╭─ ASK THIS PAGE 🐕───────────╮{}", teal, reset);
        print!("\n{}│{} ", teal, reset);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() { continue; }
        if input == "/quit" || input == "/exit" { break; }
        
        if input == "/preview" {
            if let Some(img_url) = &meta.image_url {
                println!("{}🖼️ Attempting to preview image...{}", gray, reset);
                if let Err(e) = preview_remote_image(img_url).await {
                    println!("{}❌ Error previewing image:{} {}", teal, reset, e);
                }
            } else {
                println!("{}❌ No image found on this page.{}", teal, reset);
            }
            continue;
        }

        if input == "/clear" {
            print!("\x1b[2J\x1b[H"); 
            render_mini_card(&meta, url);
            println!("---");
            continue;
        }

        println!("{}🐕 Thinking...{}", gray, reset);
        
        let qa_prompt = format!(
            "{}\n\nPast Conversation:\n{}\nUser: {}\nPlease answer based on the page content.",
            base_prompt, conversation_history, input
        );

        match ai_client.generate(qa_prompt, false).await {
            Ok(response) => {
                println!("{}🤖 AI:{}", teal, reset);
                println!("{}\n", response);
                conversation_history.push_str(&format!("User: {}\nAssistant: {}\n", input, response));
                // Keep history small
                if conversation_history.len() > 2000 {
                    conversation_history = conversation_history[conversation_history.len()-2000..].to_string();
                }
            }
            Err(e) => {
                println!("{}❌ AI Error:{} {}", teal, reset, e);
            }
        }
    }

    Ok(())
}

fn render_mini_card(meta: &PageMetadata, url: &str) {
    let teal = "\x1b[38;2;110;209;195m";
    let yellow = "\x1b[33m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let gray = "\x1b[90m";

    println!("{}╔════════════════════════════════════════════════╗{}", teal, reset);
    
    let title = if meta.title.is_empty() { "Webpage Content" } else { &meta.title };
    let display_title = if title.len() > 44 { 
        format!("{}...", &title[..41])
    } else { 
        title.to_string() 
    };
    println!("{}║ {}{:^44}{} ║{}", teal, bold, display_title, reset, teal);
    
    println!("{}║                                                ║{}", teal, reset);
    
    let display_url = if url.len() > 44 { 
        format!("{}...", &url[..41])
    } else { 
        url.to_string() 
    };
    println!("{}║ {}🔗 {:<43}{} ║{}", teal, reset, display_url, teal, reset);

    if !meta.description.is_empty() {
        let desc = if meta.description.len() > 44 { 
            format!("{}...", &meta.description[..41])
        } else { 
            meta.description.to_string() 
        };
        println!("{}║ {}📝 {:<43}{} ║{}", teal, reset, desc, teal, reset);
    }

    if meta.image_url.is_some() {
         println!("{}║ {}🖼️  Image found! Type /preview to view      {} ║{}", teal, yellow, reset, teal);
    }
    
    println!("{}╚════════════════════════════════════════════════╝{}", teal, reset);
    println!("{}Commands: /quit, /clear, /preview, or just ask a question.{}", gray, reset);
}

async fn preview_remote_image(url: &str) -> Result<()> {
    // Download to temp file
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;
    let bytes = res.bytes().await?;
    
    let tmp_path = std::env::temp_dir().join("chev-browser-preview.jpg");
    std::fs::write(&tmp_path, bytes)?;
    
    crate::ui::protocol::send_rio(crate::ui::protocol::RioAction::Preview(tmp_path.to_string_lossy().to_string()));
    Ok(())
}

fn wait_for_exit() {
    let gray = "\x1b[90m";
    let reset = "\x1b[0m";
    println!("\n{}Press Enter to close this sidebar...{}", gray, reset);
    let mut dummy = String::new();
    let _ = io::stdin().read_line(&mut dummy);
}

fn extract_content(html: &str) -> (String, PageMetadata) {
    let mut output = String::new();
    let mut meta = PageMetadata::default();
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
            
            // Extract Title
            if tag == "title" {
                 // Next text will be title, but we need to handle that separately?
                 // Simple greedy approach: find next </title>
            }

            if tag.starts_with("meta") {
                if (tag.contains("property=\"og:title\"") || tag.contains("name=\"title\"")) && meta.title.is_empty() {
                    meta.title = extract_attr(&tag, "content");
                }
                if (tag.contains("property=\"og:image\"") || tag.contains("name=\"image\"")) && meta.image_url.is_none() {
                    meta.image_url = Some(extract_attr(&tag, "content"));
                }
                if (tag.contains("property=\"og:description\"") || tag.contains("name=\"description\"")) && meta.description.is_empty() {
                    meta.description = extract_attr(&tag, "content");
                }
            }
            
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
    
    // Fallback for title if og:title missing
    if meta.title.is_empty() {
        if let Some(start) = html.to_lowercase().find("<title>") {
            if let Some(end) = html.to_lowercase()[start..].find("</title>") {
                meta.title = html[start+7..start+end].trim().to_string();
            }
        }
    }

    let lines: Vec<&str> = output.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    
    (lines.join("\n"), meta)
}

fn extract_attr(tag: &str, attr: &str) -> String {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = tag.find(&pattern) {
        let content_start = start + pattern.len();
        if let Some(end) = tag[content_start..].find("\"") {
            return tag[content_start..content_start+end].to_string();
        }
    }
    String::new()
}
