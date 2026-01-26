use std::io::Write;
use rustyline::error::ReadlineError;
use rustyline::{Cmd, KeyEvent, KeyCode, Modifiers, Movement, Word, At};
use clap::Parser;

use std::sync::{Arc, Mutex};
use chev_shell::engine::jobs::JobManager;
use chev_shell::engine::env::EnvManager;
use chev_shell::engine::macros::MacroManager;
use chev_shell::{engine, ui};

#[derive(Parser, Debug)]
#[command(author, version, about = "🐕 Chev Shell - An AI-native shell built in Rust")]
struct Args {
    /// Command to execute directly (optional)
    #[arg(short, long)]
    command: Option<String>,

    #[command(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// AI features
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },
    /// Internal tools
    Internal {
        #[command(subcommand)]
        action: InternalAction,
    },
}

#[derive(clap::Subcommand, Debug)]
enum InternalAction {
    /// Broot wrapper for IDE mode
    IdeBroot,
    /// AI Browser for summarization
    Browse { url: String },
}

#[derive(clap::Subcommand, Debug)]
enum AiAction {
    /// Start a chat session
    Chat {
        /// Internal flag for sidebar use
        #[arg(long)]
        internal: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Ignore terminal signals in the shell process to prevent it from stopping
    // when a child process is given control of the terminal.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        libc::signal(libc::SIGTTIN, libc::SIG_IGN);
    }

    let jobs = Arc::new(Mutex::new(JobManager::new()));
    let env_manager = Arc::new(Mutex::new(EnvManager::new()));
    let macro_manager = Arc::new(Mutex::new(MacroManager::new()));

    // Background task to reap zombie processes
    let jobs_for_reaper = Arc::clone(&jobs);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let mut jobs = jobs_for_reaper.lock().unwrap();
            let mut to_remove = Vec::new();

            for job in jobs.get_jobs() {
                // Non-blocking check for process status
                let pid = job.pgid;
                match nix::sys::wait::waitpid(pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
                    Ok(nix::sys::wait::WaitStatus::Exited(_, _)) | Ok(nix::sys::wait::WaitStatus::Signaled(_, _, _)) => {
                        to_remove.push(pid);
                    }
                    _ => {}
                }
            }

            for pid in to_remove {
                jobs.remove_job(pid);
            }
        }
    });

    if let Some(Commands::Ai { action }) = args.subcommand {
        match action {
            AiAction::Chat { internal } => {
                ui::chat::run_ai_chat(internal).await?;
                return Ok(());
            }
        }
    }

    if let Some(Commands::Internal { action }) = args.subcommand {
        match action {
            InternalAction::IdeBroot => {
                let tmp_file = "/tmp/broot-out-chev";
                let _ = std::fs::remove_file(tmp_file);
                
                let mut child = std::process::Command::new("broot")
                    .arg("--outcmd")
                    .arg(tmp_file)
                    .spawn()?;
                
                child.wait()?;

                if let Ok(content) = std::fs::read_to_string(tmp_file) {
                    if let Some(path) = content.strip_prefix("edit ") {
                         let path = path.trim();
                         chev_shell::ui::protocol::send_rio(chev_shell::ui::protocol::RioAction::Edit(path.to_string()));
                    }
                }
                return Ok(());
            }
            InternalAction::Browse { url } => {
                ui::browser::run_ai_browser(&url).await?;
                return Ok(());
            }
        }
    }

    if let Some(cmd) = args.command {
        // Execute a single command and exit
        if let Err(e) = engine::executor::execute_command(&cmd, &jobs, &env_manager, &macro_manager).await {
            eprintln!("Chev Error: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    let checker = engine::executor::AiChecker::new();
    let model_name = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
    let ollama_running = checker.is_ollama_running().await;
    let has_model = if ollama_running { checker.has_model(&model_name).await } else { false };

    let mut intro_lines = vec!["🐚  Chev Shell v0.1.0-alpha has been activated".to_string()];
    
    if !ollama_running || !has_model {
        let red = "\x1b[31m";
        let yellow = "\x1b[33m";
        let reset = "\x1b[0m";
        let gray = "\x1b[90m";

        intro_lines.push(format!("{}⚠️  AI features disabled{}", red, reset));
        intro_lines.push("".to_string());
        intro_lines.push(format!("{}HOW TO ENABLE AI:{}", yellow, reset));
        
        if !ollama_running {
            intro_lines.push(format!("  1. Install Ollama: {}https://ollama.com{}", gray, reset));
            intro_lines.push(format!("  2. Start the Ollama application."));
        }

        // Add a check for protoc as it's needed for the local vector DB
        intro_lines.push(format!("  3. Install {}protobuf{} (required for LanceDB): {}brew install protobuf{}", yellow, reset, gray, reset));
        
        if !has_model {
            intro_lines.push(format!("  4. Run {}ai setup{} to download the model.", yellow, reset));
        }
        intro_lines.push("".to_string());
    }

    intro_lines.extend(vec![
        "WHAT IS CHEV?".to_string(),
        "  Chev (named after my girlfriend) is a next-generation, AI-native shell built in Rust.".to_string(),
        "  It transforms your terminal into a GPU-accelerated co-processor.".to_string(),
        "".to_string(),
        "KEY FEATURES:".to_string(),
        "  ⚡ High-Performance: Built in Rust for zero-latency execution.".to_string(),
        "  🧠 AI-Native: Integrated with Ollama & qwen2.5-coder:7b for terminal advice.".to_string(),
        "  🐕 Modern Aliases: ls -> eza, cd -> zoxide, grep -> rg, and more.".to_string(),
        "  🎨 Visual UX: GPU-accelerated rendering & advanced prompt states.".to_string(),
        "".to_string(),
        "QUICK START:".to_string(),
        "  type 'exit' to quit. use standard commands to see the magic.".to_string(),
        "".to_string(),
    ]);

    ui::effects::display_parallel_intro(intro_lines).await;
    
    let mut env_mgmt = env_manager.lock().unwrap();
    let _isolated_bin = env_mgmt.setup_isolated_bin()?;
    drop(env_mgmt);

    // Initial tool scan for the "Ultimate" experience
    let tools_to_check = vec![
        "eza", "zoxide", "fd", "dust", "rip", "xcp", "broot", "lfs", 
        "miniserve", "bat", "mdcat", "rg", "sd", "delta", "jql", "qsv", 
        "tldr", "heh", "lemmeknow", "kibi", "btm", "procs", "hyperfine", 
        "just", "hwatch", "doggo", "gping", "xh", "fend", "ouch"
    ];
    let installed_count = tools_to_check.iter().filter(|t| which::which(t).is_ok()).count();
    let total_count = tools_to_check.len();
    
    let blue = "\x1b[38;2;67;147;255m";
    let teal = "\x1b[38;2;110;209;195m";
    let reset = "\x1b[0m";
    
    // Detect Semantic Support (OSC 133)
    let term = std::env::var("TERM_PROGRAM").unwrap_or_default();
    let semantic_active = match term.to_lowercase().as_str() {
        "rio" | "wezterm" | "ghostty" | "warp" => true,
        _ => false,
    };

    println!("{}🔋 Power-up Status: {}/{} tools active.{}", blue, installed_count, total_count, reset);
    if semantic_active {
        println!("{}🧊 Semantic Blocks: ACTIVE ({}){}", teal, term, reset);
    } else {
        println!("{}🧊 Semantic Blocks: EMULATED (Using standard sequences){}", "\x1b[90m", reset);
    }
    
    if installed_count < total_count {
        println!("  {}Tip: Run 'ai setup' to activate missing tools internally.{}", "\x1b[90m", reset);
    }
    println!();

    let ghost_state = Arc::new(Mutex::new(ui::suggestions::GhostState::default()));
    
    // Background task for Ghost Text (AI Suggestions)
    let ghost_state_clone = Arc::clone(&ghost_state);
    let _algo_jobs = Arc::clone(&jobs); // Jobs manager available if needed context
    let _algo_env = Arc::clone(&env_manager);
    let semantic_active_clone = semantic_active;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(200));
        let model_name = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
        
        loop {
            interval.tick().await;
            
            let (buffer, should_trigger) = {
                if let Ok(state) = ghost_state_clone.lock() {
                    if let Some(last) = state.last_typing {
                        if state.ghost_text.is_none() && !state.current_buffer.is_empty() && last.elapsed().as_millis() > 1000 {
                             (state.current_buffer.clone(), true)
                        } else {
                            (String::new(), false)
                        }
                    } else {
                        (String::new(), false)
                    }
                } else {
                    (String::new(), false)
                }
            };
            
            if should_trigger {
                // Generate suggestion
                 let suggestion = chev_shell::ai::mimic::generate_ghost_suggestion(&model_name, &buffer).await;

                 if let Some(sugg) = suggestion {
                     if let Ok(mut state) = ghost_state_clone.lock() {
                         // Double check we haven't typed in the meantime
                         if state.current_buffer == buffer {
                            state.ghost_text = Some(sugg.clone());
                            if semantic_active_clone {
                                use std::io::Write;
                                // Send side-channel command
                                print!("\x1b]1338;ghost;{}\x07", sugg);
                                let _ = std::io::stdout().flush();
                            }
                         }
                     }
                 }
            }
        }
    });

    let mut rl = rustyline::Editor::<ui::suggestions::ShellHelper, rustyline::history::FileHistory>::new()?;
    rl.set_helper(Some(ui::suggestions::ShellHelper::new(Arc::clone(&macro_manager), Arc::clone(&ghost_state), semantic_active)));

    // Key Bindings: Accept hint with Tab or Right Arrow
    rl.bind_sequence(KeyEvent(KeyCode::Tab, Modifiers::NONE), Cmd::Complete);
    rl.bind_sequence(KeyEvent(KeyCode::Right, Modifiers::NONE), Cmd::Move(Movement::EndOfLine));
    
    // Additional bindings for Mac-like behavior
    rl.bind_sequence(KeyEvent(KeyCode::Left, Modifiers::ALT), Cmd::Move(Movement::BackwardWord(1, Word::Emacs)));
    rl.bind_sequence(KeyEvent(KeyCode::Right, Modifiers::ALT), Cmd::Move(Movement::ForwardWord(1, At::AfterEnd, Word::Emacs)));
    // Also support Control+U as a fallback for delete line
    rl.bind_sequence(KeyEvent(KeyCode::Char('u'), Modifiers::CTRL), Cmd::Kill(Movement::BeginningOfLine));

    let home = dirs::home_dir().expect("Home dir not found");
    let chev_dir = home.join(".chev");
    let history_path = chev_dir.join("history.txt");
    let suggestions_path = chev_dir.join("suggestions.json");

    if rl.load_history(&history_path).is_err() {
        // Silently continue if no history
    } else {
        if let Some(helper) = rl.helper_mut() {
            helper.trie.load(suggestions_path.to_str().unwrap());
        }
    }

    loop {
        let prompt_parts = ui::prompt::get_prompt_parts();
        if let Some(helper) = rl.helper_mut() {
            helper.prompt_parts = prompt_parts.clone();
        }
        let prompt = prompt_parts.to_colored_string(semantic_active);
        
        let readline = rl.readline(&prompt);
        
        // Mark Command Start after prompt ends (semantic)
        if semantic_active {
            print!("\x1b]133;C\x07"); 
            let _ = std::io::stdout().flush();
        }
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() { continue; }
                if input == "exit" || input == "quit" { break; }

                let _ = rl.add_history_entry(input);
                let _ = rl.save_history(&history_path);
                if let Some(helper) = rl.helper_mut() {
                    helper.trie.add(input);
                }

                // Handle abbreviations built-in
                if input.starts_with("abbr ") {
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    if parts.len() >= 3 {
                        macro_manager.lock().unwrap().set_abbreviation(parts[1].to_string(), parts[2..].join(" "));
                        println!("Abbreviation set.");
                        continue;
                    }
                }

                // Execute via our engine
                let result = engine::executor::execute_command(input, &jobs, &env_manager, &macro_manager).await;
                
                // Semantic: Block End (assuming exit code 0 on success, 1 on error for now)
                if semantic_active {
                    let exit_code = if result.is_ok() { 0 } else { 1 };
                    println!("\x1b]133;D;{}\x07", exit_code);
                }

                if let Err(e) = result {
                    let err_str = e.to_string();
                    if err_str.contains("os error 2") {
                         eprintln!("\x1b[31m❌ Command not found: {}\x1b[0m", input.split_whitespace().next().unwrap_or(""));
                         eprintln!("\x1b[90m   Tip: Ask AI with 'ai ask \"{}\"'\x1b[0m", input);
                    } else {
                        eprintln!("\x1b[31mChev Error: {}\x1b[0m", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                if semantic_active { println!("\x1b]133;D;1\x07"); }
                println!("SIGINT");
                continue;
            }
            Err(ReadlineError::Eof) => {
                if semantic_active { println!("\x1b]133;D\x07"); }
                println!("Exiting...");
                // Terminate all active jobs
                {
                    let jobs = jobs.lock().unwrap();
                    for job in jobs.get_jobs() {
                        #[cfg(unix)]
                        {
                            use nix::sys::signal::{kill, Signal};
                            let _ = kill(job.pgid, Signal::SIGTERM);
                        }
                    }
                }
                // Save suggestions
                if let Some(helper) = rl.helper_mut() {
                    helper.trie.save(suggestions_path.to_str().unwrap());
                }
                // Save macros
                let _ = macro_manager.lock().unwrap().save();
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(&history_path)?;
    Ok(())
}