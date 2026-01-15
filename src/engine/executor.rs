use tokio::process::Command;
use std::process::Stdio;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::convert::TryInto;
use crate::engine::parser::{parse_pipeline, Pipeline, Redirection};
use crate::engine::jobs::{JobManager, JobStatus};
use crate::engine::env::EnvManager;
use crate::engine::macros::MacroManager;
use std::sync::{Arc, Mutex};
use std::os::fd::BorrowedFd;
pub use crate::ai::AiChecker;

pub async fn execute_command(input: &str, jobs: &Arc<Mutex<JobManager>>, env_manager: &Arc<Mutex<EnvManager>>, macro_manager: &Arc<Mutex<MacroManager>>) -> Result<()> {
    // 1. Expand Macros first
    let expanded = {
        let macros = macro_manager.lock().unwrap();
        macros.expand_macro(input).unwrap_or_else(|| input.to_string())
    };

    let (_, pipeline) = parse_pipeline(&expanded)
        .map_err(|e| anyhow!("Parse error: {}", e))?;

    let result = execute_pipeline(pipeline, jobs, env_manager, macro_manager).await;
    
    if result.is_err() {
        let mut macros = macro_manager.lock().unwrap();
        if macros.last_error.is_none() {
            // If it wasn't captured inside (e.g. spawn failure), capture it here
            macros.last_error = Some((expanded.clone(), result.as_ref().err().unwrap().to_string()));
        }
    }
    result
}

async fn execute_pipeline(pipeline: Pipeline, jobs_mutex: &Arc<Mutex<JobManager>>, env_mutex: &Arc<Mutex<EnvManager>>, macro_mutex: &Arc<Mutex<MacroManager>>) -> Result<()> {
    let background = pipeline.background;
    let mut prev_stdout: Option<Stdio> = None;
    let commands_len = pipeline.commands.len();
    let mut pipeline_pgid = None;

    // String for job manager representation
    let full_cmd_str: String = pipeline.commands.iter()
        .map(|c| c.args.join(" "))
        .collect::<Vec<_>>()
        .join(" | ");

    for (i, cmd) in pipeline.commands.into_iter().enumerate() {
        let is_last = i == commands_len - 1;

        // Extract command and raw args
        if cmd.args.is_empty() { continue; }
        let original_command = &cmd.args[0];

        // Handle Job Control Built-ins (fg, bg, jobs)
        if i == 0 {
            match original_command.as_str() {
                "jobs" => {
                    let jobs = jobs_mutex.lock().unwrap();
                    let gray = "\x1b[90m";
                    let reset = "\x1b[0m";

                    for job in jobs.get_jobs() {
                        let elapsed = job.start_time.elapsed();
                        let duration_str = if elapsed.as_secs() < 60 {
                            format!("{}s", elapsed.as_secs())
                        } else if elapsed.as_secs() < 3600 {
                            format!("{}m {}s", elapsed.as_secs() / 60, elapsed.as_secs() % 60)
                        } else {
                            format!("{}h {}m", elapsed.as_secs() / 3600, (elapsed.as_secs() % 3600) / 60)
                        };

                        println!(
                            "[{}] {}  \t {} \t {}(active for {}){}", 
                            job.id, job.status, job.cmd, gray, duration_str, reset
                        );
                    }
                    return Ok(());
                }
                "fg" => {
                    let id = cmd.args.get(1).and_then(|s| s.parse::<usize>().ok());
                    let jobs = jobs_mutex.lock().unwrap();
                    if let Some(target_job) = id.and_then(|i| jobs.find_job_by_id(i).cloned()) {
                        println!("Bringing job [{}] to foreground: {}", target_job.id, target_job.cmd);
                        
                        #[cfg(unix)]
                        {
                            use nix::sys::signal::{kill, Signal};
                            use nix::unistd::tcsetpgrp;
                            
                            let shell_pgid = nix::unistd::getpgrp();
                            let job_pgid = target_job.pgid;

                            let stdin = unsafe { BorrowedFd::borrow_raw(libc::STDIN_FILENO) };

                            // 1. Give terminal to job
                            let _ = tcsetpgrp(stdin, job_pgid);
                            
                            // 2. Resume if stopped
                            let _ = kill(job_pgid, Signal::SIGCONT);
                            
                            // 3. Wait for it
                            let _ = nix::sys::wait::waitpid(job_pgid, Some(nix::sys::wait::WaitPidFlag::WUNTRACED));
                            
                            // 4. Take back terminal
                            let _ = tcsetpgrp(stdin, shell_pgid);
                        }
                    } else {
                        println!("fg: job not found");
                    }
                    return Ok(());
                }
                "bg" => {
                    let id = cmd.args.get(1).and_then(|s| s.parse::<usize>().ok());
                    let jobs = jobs_mutex.lock().unwrap();
                    if let Some(target_job) = id.and_then(|i| jobs.find_job_by_id(i)) {
                        println!("Resuming job [{}] in background: {}", target_job.id, target_job.cmd);
                        #[cfg(unix)]
                        {
                            use nix::sys::signal::{kill, Signal};
                            let _ = kill(target_job.pgid, Signal::SIGCONT);
                        }
                    } else {
                        println!("bg: job not found");
                    }
                    return Ok(());
                }
                "set" => {
                    let mut env = env_mutex.lock().unwrap();
                    if cmd.args.len() == 1 {
                        // List variables with nice teal highlights
                        let teal = "\x1b[38;2;110;209;195m";
                        let reset = "\x1b[0m";
                        for (k, v) in env.get_all_vars() {
                            println!("{}{}={} {}", teal, k, reset, v);
                        }
                    } else if let Some(arg) = cmd.args.get(1) {
                        if let Some((k, v)) = arg.split_once('=') {
                            env.set_var(k.to_string(), v.to_string());
                        } else {
                            // Classic style: set KEY VALUE
                            if let Some(v) = cmd.args.get(2) {
                                env.set_var(arg.to_string(), v.to_string());
                            } else {
                                // set KEY (empty value)
                                env.set_var(arg.to_string(), "".to_string());
                            }
                        }
                    }
                    return Ok(());
                }
                "unset" => {
                    let mut env = env_mutex.lock().unwrap();
                    if let Some(arg) = cmd.args.get(1) {
                        env.remove_var(arg);
                    }
                    return Ok(());
                }
                "path" => {
                    // Modern helper: path add /path/to/bin
                    let mut env = env_mutex.lock().unwrap();
                    match cmd.args.get(1).map(|s| s.as_str()) {
                        Some("add") => {
                            if let Some(p) = cmd.args.get(2) {
                                env.add_to_path(p, false);
                                println!("Added to PATH: {}", p);
                            }
                        }
                        Some("prepend") => {
                            if let Some(p) = cmd.args.get(2) {
                                env.add_to_path(p, true);
                                println!("Prepended to PATH: {}", p);
                            }
                        }
                        _ => {
                            println!("Usage: path [add|prepend] <dir>");
                        }
                    }
                    return Ok(());
                }
                "pushd" => {
                    let mut env = env_mutex.lock().unwrap();
                    if let Some(p) = cmd.args.get(1) {
                        env.pushd(PathBuf::from(p))?;
                        println!("{}", env.get_stack().join("  "));
                    }
                    return Ok(());
                }
                "popd" => {
                    let mut env = env_mutex.lock().unwrap();
                    env.popd()?;
                    println!("{}", env.get_stack().join("  "));
                    return Ok(());
                }
                "dirs" => {
                    let env = env_mutex.lock().unwrap();
                    println!("{}", env.get_stack().join("  "));
                    return Ok(());
                }
                "macro" => {
                    let mut macros = macro_mutex.lock().unwrap();
                    let teal = "\x1b[38;2;110;209;195m";
                    let reset = "\x1b[0m";

                    match cmd.args.get(1).map(|s| s.as_str()) {
                        Some("set") => {
                            if cmd.args.len() >= 4 {
                                let name = cmd.args[2].clone();
                                let template = cmd.args[3..].join(" ");
                                macros.set_macro(name, template)?;
                                println!("Macro set successfully.");
                            } else {
                                println!("Usage: macro set <name> <template>");
                            }
                        }
                        Some("unset") => {
                            if let Some(name) = cmd.args.get(2) {
                                macros.unset_macro(name)?;
                                println!("Macro unset.");
                            }
                        }
                        _ => {
                            println!("{}🐚 Chev Macros:{}", teal, reset);
                            for (name, m) in macros.list() {
                                println!("  {} -> {}", name, m.template);
                            }
                        }
                    }
                    return Ok(());
                }
                "ai" => {
                    let teal = "\x1b[38;2;110;209;195m";
                    let reset = "\x1b[0m";
                    let gray = "\x1b[90m";

                    match cmd.args.get(1).map(|s| s.as_str()) {
                        Some("ask") => {
                            let prompt = cmd.args[2..].join(" ");
                            if prompt.is_empty() {
                                println!("Usage: ai ask <your question>");
                                return Ok(());
                            }

                            println!("{}🐕 Chev is thinking...{}", gray, reset);
                            
                            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
                            let client = crate::ai::OllamaClient::new(model);
                            
                            match client.generate(prompt, false).await {
                                Ok(response) => {
                                    println!("{}🤖 AI:{} {}", teal, reset, response);
                                }
                                Err(e) => {
                                    eprintln!("{}❌ Error:{} {}", teal, reset, e);
                                }
                            }
                        }
                        Some("fix") => {
                            let (last_cmd, last_err) = {
                                let macros = macro_mutex.lock().unwrap();
                                match &macros.last_error {
                                    Some(err) => err.clone(),
                                    None => {
                                        println!("{}No failed command found to fix.{}", gray, reset);
                                        return Ok(());
                                    }
                                }
                            };

                            println!("{}🐕 Analyzing last failure...{}", gray, reset);

                            let prompt = format!(
                                "The user ran: `{}`\nIt failed with this error:\n```\n{}\n```\nProvide a fixed command in JSON format: {{\"fixed_command\": \"...\"}}. Only return the JSON.",
                                last_cmd, last_err
                            );

                            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
                            let client = crate::ai::OllamaClient::new(model);

                            match client.generate(prompt, true).await {
                                Ok(response) => {
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                                        if let Some(fixed) = json["fixed_command"].as_str() {
                                            println!("{}💡 Suggestion:{} {}", teal, reset, fixed);
                                            let mut macros = macro_mutex.lock().unwrap();
                                            macros.last_suggestion = Some(fixed.to_string());
                                        }
                                    } else {
                                        println!("{}AI returned an invalid response.{}", gray, reset);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{}❌ Error:{} {}", teal, reset, e);
                                }
                            }
                        }
                        Some("search") => {
                            let query = cmd.args[2..].join(" ");
                            if query.is_empty() {
                                println!("Usage: ai search <description of a past command>");
                                return Ok(());
                            }

                            println!("{}🔍 Searching semantic history...{}", gray, reset);

                            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
                            let client = crate::ai::OllamaClient::new(model);
                            let mimic = crate::ai::MimicManager::new();

                            match client.embeddings(query).await {
                                Ok(vector) => {
                                    match mimic.search(vector, 5).await {
                                        Ok(results) => {
                                            if results.is_empty() {
                                                println!("{}No matching commands found.{}", gray, reset);
                                            } else {
                                                println!("{}🎯 Top matches:{}", teal, reset);
                                                for (i, res) in results.iter().enumerate() {
                                                    println!("  {}. {}", i + 1, res);
                                                }
                                                // Set the first result as suggestion
                                                let mut macros = macro_mutex.lock().unwrap();
                                                macros.last_suggestion = Some(results[0].clone());
                                                println!("{}💡 Hit Tab to use the top match.{}", gray, reset);
                                            }
                                        }
                                        Err(e) => eprintln!("{}❌ Search error:{} {}", teal, reset, e),
                                    }
                                }
                                Err(e) => eprintln!("{}❌ Embedding error:{} {}", teal, reset, e),
                            }
                        }
                        Some("status") => {
                            let checker = AiChecker::new();
                            let model_name = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
                            let running = checker.is_ollama_running().await;
                            
                            println!("{}📊 AI Status:{}", teal, reset);
                            if running {
                                println!("  Ollama: {}Running{}", "\x1b[32m", reset);
                                let has_model = checker.has_model(&model_name).await;
                                if has_model {
                                    println!("  Model ({}): {}Installed{}", model_name, "\x1b[32m", reset);
                                    println!("  Ready: {}YES{}", "\x1b[32m", reset);
                                } else {
                                    println!("  Model ({}): {}Not Found{}", model_name, "\x1b[31m", reset);
                                    println!("  Ready: {}NO (Run 'ai setup'){}", "\x1b[31m", reset);
                                }
                            } else {
                                println!("  Ollama: {}Not Running{}", "\x1b[31m", reset);
                                println!("  Ready: {}NO (Start Ollama application){}", "\x1b[31m", reset);
                            }
                        }
                        Some("setup") => {
                            let checker = AiChecker::new();
                            let model_name = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
                            
                            if !checker.is_ollama_running().await {
                                println!("{}❌ Error:{} Ollama is not running. Please start Ollama application first.", "\x1b[31m", reset);
                                return Ok(());
                            }

                            if checker.has_model(&model_name).await {
                                println!("{}✅ Model '{}' is already installed.{}", "\x1b[32m", model_name, reset);
                                return Ok(());
                            }

                            println!("{}⏳ Pulling model '{}'... this might take a while.{}", teal, model_name, reset);
                            match checker.pull_model(&model_name).await {
                                Ok(_) => println!("{}✅ Model '{}' successfully installed!{}", "\x1b[32m", model_name, reset),
                                Err(e) => eprintln!("{}❌ Failed to pull model:{} {}", "\x1b[31m", reset, e),
                            }
                        }
                        _ => {
                            println!("{}🤖 Chev AI Help:{}", teal, reset);
                            println!("  ai ask <prompt>  - Ask the AI for advice or help.");
                            println!("  ai fix           - Fix the last failed command.");
                            println!("  ai search <desc> - Search history semantically.");
                            println!("  ai status        - Check AI system health.");
                            println!("  ai setup         - Install the required AI model.");
                        }
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        // Handle built-ins for the first/only command
        // Note: Built-ins don't usually pipe well in simple implementations, 
        // but we'll support cd as a special case.
        if original_command == "cd" && commands_len == 1 {
            return handle_cd(cmd.args.iter().skip(1).map(|s| s.as_str()).collect()).await;
        }

        let raw_args: Vec<&str> = cmd.args.iter().skip(1).map(|s| s.as_str()).collect();
        let (real_command, mapped_args) = resolve_command(original_command, raw_args).await?;
        
        let mut tokio_cmd = Command::new(real_command);
        tokio_cmd.args(mapped_args);

        // Process group management for job control
        // On Unix, each pipeline gets a new process group
        #[cfg(unix)]
        {
            if i == 0 {
                // First process in pipeline sets its own pgid
                tokio_cmd.process_group(0);
            } else if let Some(pgid) = pipeline_pgid {
                // Subsequent processes join the first one's group
                tokio_cmd.process_group(pgid);
            }
        }

        // Handle Input from pipe
        if let Some(stdout) = prev_stdout.take() {
            tokio_cmd.stdin(stdout);
        }

        // Handle Output to pipe
        if !is_last {
            tokio_cmd.stdout(Stdio::piped());
        }

        // Handle stderr capture for the last command to support 'ai fix'
        if is_last {
            tokio_cmd.stderr(Stdio::piped());
        }

        // Handle Redirections
        for red in cmd.redirections {
            match red {
                Redirection::Stdout(path) => {
                    let file = File::create(path).map_err(|e| anyhow!("Failed to create output file: {}", e))?;
                    tokio_cmd.stdout(Stdio::from(file));
                }
                Redirection::Stderr(path) => {
                    let file = File::create(path).map_err(|e| anyhow!("Failed to create error file: {}", e))?;
                    tokio_cmd.stderr(Stdio::from(file));
                }
                Redirection::Append(path) => {
                    let file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path).map_err(|e| anyhow!("Failed to open append file: {}", e))?;
                    tokio_cmd.stdout(Stdio::from(file));
                }
                Redirection::StderrToStdout => {
                    // Simplified: We'll leave it for now
                }
            }
        }

        let mut child = tokio_cmd.spawn()
            .map_err(|e| anyhow!("Failed to spawn {}: {}", cmd.args[0], e))?;

        // Capture PGID of the first process
        if i == 0 {
            if let Some(id) = child.id() {
                pipeline_pgid = Some(id as i32);
            }
        }

        if is_last {
            let mut captured_stderr = String::new();
            if let Some(mut stderr) = child.stderr.take() {
                use tokio::io::AsyncReadExt;
                let mut buffer = [0u8; 1024];
                while let Ok(n) = stderr.read(&mut buffer).await {
                    if n == 0 { break; }
                    let s = String::from_utf8_lossy(&buffer[..n]);
                    captured_stderr.push_str(&s);
                    eprint!("{}", s); // Still show it to the user
                }
            }

            let status = if background {
                if let Some(pgid) = pipeline_pgid {
                    let mut jobs = jobs_mutex.lock().unwrap();
                    let id = jobs.add_job(nix::unistd::Pid::from_raw(pgid), full_cmd_str.clone(), JobStatus::Running);
                    println!("[{}] {}", id, pgid);
                }
                Ok(())
            } else {
                #[cfg(unix)]
                {
                    if let Some(pgid) = pipeline_pgid {
                        let shell_pgid = nix::unistd::getpgrp();
                        let job_pgid = nix::unistd::Pid::from_raw(pgid);
                        let stdin = unsafe { BorrowedFd::borrow_raw(libc::STDIN_FILENO) };
                        let is_tty = unsafe { libc::isatty(libc::STDIN_FILENO) != 0 };
                        
                        if is_tty {
                            let _ = nix::unistd::tcsetpgrp(stdin, job_pgid);
                        }
                        
                        // Wait for child using nix to catch Stopped status
                        let wait_res = loop {
                            use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};
                            match waitpid(job_pgid, Some(WaitPidFlag::WUNTRACED)) {
                                Ok(WaitStatus::Stopped(pid, _)) => {
                                    let mut jobs = jobs_mutex.lock().unwrap();
                                    let id = jobs.add_job(pid, full_cmd_str.clone(), JobStatus::Suspended);
                                    println!("\n[{}] {} \t Stopped", id, full_cmd_str);
                                    break Ok(());
                                }
                                Ok(WaitStatus::Exited(_, status)) => {
                                    if status == 0 { break Ok(()); }
                                    else { break Err(anyhow!("Command exited with code {}", status)); }
                                }
                                Ok(WaitStatus::Signaled(_, sig, _)) => break Err(anyhow!("Command killed by signal {:?}", sig)),
                                Ok(WaitStatus::Continued(_)) => continue,
                                Err(e) => break Err(anyhow!("Wait error: {}", e)),
                                _ => continue,
                            }
                        };
                        
                        if is_tty {
                            let _ = nix::unistd::tcsetpgrp(stdin, shell_pgid);
                        }
                        wait_res
                    } else {
                        let status = child.wait().await?;
                        if status.success() { Ok(()) } else { Err(anyhow!("Exited with status {}", status)) }
                    }
                }
                #[cfg(not(unix))]
                {
                    let status = child.wait().await?;
                    if status.success() { Ok(()) } else { Err(anyhow!("Exited with status {}", status)) }
                }
            };

            // Store error context if command failed
            if status.is_err() {
                let mut macros = macro_mutex.lock().unwrap();
                macros.last_error = Some((full_cmd_str.clone(), captured_stderr));
            } else {
                // Clear context on success
                let mut macros = macro_mutex.lock().unwrap();
                macros.last_error = None;
                macros.last_suggestion = None;

                // Semantic history recording (background task)
                if !full_cmd_str.starts_with("ai ") {
                    let cmd_to_record = full_cmd_str.clone();
                    tokio::spawn(async move {
                        let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
                        let client = crate::ai::OllamaClient::new(model);
                        let mimic = crate::ai::MimicManager::new();
                        if let Ok(vector) = client.embeddings(cmd_to_record.clone()).await {
                            let _ = mimic.add_command(&cmd_to_record, vector).await;
                        }
                    });
                }
            }

            status?;
            prev_stdout = None;
        } else {
            if let Some(stdout) = child.stdout.take() {
                let std_stdout: Stdio = stdout.try_into()
                    .map_err(|e| anyhow!("Failed to convert stdout: {}", e))?;
                prev_stdout = Some(std_stdout);
            }
        }
    }

    Ok(())
}

async fn resolve_command<'a>(command: &'a str, args: Vec<&'a str>) -> Result<(String, Vec<&'a str>)> {
    let mapped = match command {
        // Navigation & Files
        "ls" => "eza",
        "find" => "fd",
        "du" => "dust",
        "rm" => "rip",
        "cp" => "xcp",
        "tree" => "broot",
        
        // Text & Data
        "cat" => "bat",
        "grep" | "rg" => "rg",
        "sed" => "sd",
        "diff" => "delta",
        "cut" | "awk" if args.iter().any(|a| a.contains(':')) => "choose",
        "jq" => "jql",
        "tldr" | "man" if !args.is_empty() => "tldr",

        // System & Monitoring
        "top" | "htop" => "btm",
        "ps" => "procs",
        "time" => "hyperfine",
        "make" => "just",
        "watch" => "hwatch",
        "dig" => "doggo",
        "sudo" => "sudo-rs",

        _ => command,
    };
    Ok((mapped.to_string(), args))
}

async fn handle_cd(args: Vec<&str>) -> Result<()> {
    let target = args.get(0).map(|&s| s).unwrap_or("~");
    
    // Resolve path (handle ~)
    let path_str = if target == "~" {
        dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?
            .to_string_lossy()
            .to_string()
    } else if target.starts_with("~/") {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        home.join(&target[2..]).to_string_lossy().to_string()
    } else {
        target.to_string()
    };

    let path = Path::new(&path_str);

    // 1. Try direct cd
    if path.exists() && path.is_dir() {
        std::env::set_current_dir(path)
            .map_err(|e| anyhow!("cd failed: {}", e))?;
        // Update zoxide history
        let _ = Command::new("zoxide").arg("add").arg(path).spawn()?.wait().await;
        return Ok(());
    }

    // 2. Smart jump via zoxide if target is not a valid path
    let output = Command::new("zoxide")
        .arg("query")
        .arg(target)
        .output()
        .await?;

    if output.status.success() {
        let new_path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !new_path_str.is_empty() {
            let new_path = Path::new(&new_path_str);
            if new_path.exists() {
                std::env::set_current_dir(new_path)
                    .map_err(|e| anyhow!("cd failed: {}", e))?;
                // Confirm visit to zoxide
                let _ = Command::new("zoxide").arg("add").arg(new_path).spawn()?.wait().await;
                return Ok(());
            }
        }
    }

    Err(anyhow!("cd: no such file or directory: {}", target))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command_mapping() {
        let cases = vec![
            ("ls", "eza"),
            ("cat", "bat"),
            ("find", "fd"),
            ("du", "dust"),
            ("rm", "rip"),
            ("grep", "rg"),
            ("rg", "rg"),
            ("sed", "sd"),
            ("diff", "delta"),
            ("top", "btm"),
            ("htop", "btm"),
            ("ps", "procs"),
            ("time", "hyperfine"),
            ("make", "just"),
            ("cp", "xcp"),
            ("tree", "broot"),
            ("jq", "jql"),
            ("watch", "hwatch"),
            ("dig", "doggo"),
            ("sudo", "sudo-rs"),
            ("git", "git"), // No mapping
        ];

        for (cmd, expected) in cases {
            let (real, _) = resolve_command(cmd, vec![]).await.unwrap();
            assert_eq!(real, expected, "Mapping for {} failed", cmd);
        }
    }

    #[tokio::test]
    async fn test_conditional_mapping() {
        // Test choose mapping
        let (real, _) = resolve_command("cut", vec!["0:3"]).await.unwrap();
        assert_eq!(real, "choose");

        // Test tldr mapping
        let (real, _) = resolve_command("man", vec!["ls"]).await.unwrap();
        assert_eq!(real, "tldr");
    }
}