use rustyline::error::ReadlineError;
use clap::Parser;

mod engine;
mod ui;

use std::sync::{Arc, Mutex};
use crate::engine::jobs::JobManager;
use crate::engine::env::EnvManager;

#[derive(Parser, Debug)]
#[command(author, version, about = "🐕 Chev Shell - An AI-native shell built in Rust")]
struct Args {
    /// Command to execute directly (optional)
    #[arg(short, long)]
    command: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let jobs = Arc::new(Mutex::new(JobManager::new()));
    let env_manager = Arc::new(Mutex::new(EnvManager::new()));

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

    if let Some(cmd) = args.command {
        // Execute a single command and exit
        if let Err(e) = engine::executor::execute_command(&cmd, &jobs, &env_manager).await {
            eprintln!("Chev Error: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    let mut intro_lines = vec!["🐚  Chev Shell v0.1.0-alpha has been activated".to_string()];
    intro_lines.extend(vec![
        "".to_string(),
        "WHAT IS CHEV?".to_string(),
        "  Chev (named after my girlfriend) is a next-generation, AI-native shell built in Rust.".to_string(),
        "  It transforms your terminal into a GPU-accelerated co-processor.".to_string(),
        "".to_string(),
        "KEY FEATURES:".to_string(),
        "  ⚡ High-Performance: Built in Rust for zero-latency execution.".to_string(),
        "  🧠 AI-Native: Integrated with Ollama & Devstral for terminal advice.".to_string(),
        "  🐕 Modern Aliases: ls -> eza, cd -> zoxide, grep -> rg, and more.".to_string(),
        "  🎨 Visual UX: GPU-accelerated rendering & advanced prompt states.".to_string(),
        "".to_string(),
        "QUICK START:".to_string(),
        "  type 'exit' to quit. use standard commands to see the magic.".to_string(),
        "".to_string(),
    ]);

    ui::effects::display_parallel_intro(intro_lines).await;
    
    let mut rl = rustyline::Editor::<ui::suggestions::ShellHelper, rustyline::history::FileHistory>::new()?;
    rl.set_helper(Some(ui::suggestions::ShellHelper::new()));

    if rl.load_history("history.txt").is_err() {
        // Silently continue if no history
    } else {
        // Prime the trie with existing history
        let history_cmds: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect();
        if let Some(helper) = rl.helper_mut() {
            for cmd in history_cmds {
                helper.trie.add(&cmd);
            }
        }
    }

    loop {
        ui::prompt::pre_prompt();
        let prompt = ui::prompt::get_prompt();
        
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() { continue; }
                if input == "exit" || input == "quit" { break; }

                let _ = rl.add_history_entry(input);
                if let Some(helper) = rl.helper_mut() {
                    helper.trie.add(input);
                }

                // Execute via our engine
                if let Err(e) = engine::executor::execute_command(input, &jobs, &env_manager).await {
                    eprintln!("Chev Error: {}", e);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("SIGINT");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting...");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")?;
    Ok(())
}