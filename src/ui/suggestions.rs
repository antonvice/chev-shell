use rustyline::hint::{Hint, Hinter};
use rustyline::Context;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use rustyline::Helper;
use std::borrow::Cow;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandMetadata {
    pub cmd: String,
    pub cwd: String,
    pub count: u32,
    pub last_used: std::time::SystemTime,
}

/// Smarter history search using Contextual Frecency
pub struct CommandTrie {
    entries: Vec<CommandMetadata>,
}

impl CommandTrie {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, cmd: &str) {
        if cmd.is_empty() { return; }
        let cwd = std::env::current_dir().unwrap_or_default().to_string_lossy().to_string();
        
        // Update existing entry or create new one
        if let Some(entry) = self.entries.iter_mut().find(|e| e.cmd == cmd && e.cwd == cwd) {
            entry.count += 1;
            entry.last_used = std::time::SystemTime::now();
        } else {
            self.entries.push(CommandMetadata {
                cmd: cmd.to_string(),
                cwd,
                count: 1,
                last_used: std::time::SystemTime::now(),
            });
        }
    }

    pub fn load(&mut self, path: &str) {
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(entries) = serde_json::from_str(&data) {
                self.entries = entries;
            }
        }
    }

    pub fn save(&self, path: &str) {
        if let Ok(data) = serde_json::to_string_pretty(&self.entries) {
            let _ = std::fs::write(path, data);
        }
    }

    pub fn suggest(&self, input: &str) -> Option<String> {
        if input.is_empty() { return None; }
        let current_cwd = std::env::current_dir().unwrap_or_default().to_string_lossy().to_string();

        // 1. Exact History Match (Frecency ranked)
        let mut candidates: Vec<_> = self.entries.iter()
            .filter(|e| e.cmd.starts_with(input) && e.cmd != input)
            .collect();

        if !candidates.is_empty() {
            candidates.sort_by(|a, b| {
                let a_is_cwd = a.cwd == current_cwd;
                let b_is_cwd = b.cwd == current_cwd;

                if a_is_cwd != b_is_cwd {
                    return b_is_cwd.cmp(&a_is_cwd);
                }
                
                if b.count != a.count {
                    return b.count.cmp(&a.count);
                }

                b.last_used.cmp(&a.last_used)
            });

            return Some(candidates.first().unwrap().cmd[input.len()..].to_string());
        }

        // 2. Subcommand Hints (Static context)
        // If user typed 'git ', suggest 'commit', 'push', etc.
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() == 1 && input.ends_with(' ') {
            match parts[0] {
                "git" => return Some("status".to_string()),
                "docker" => return Some("ps".to_string()),
                "npm" => return Some("run".to_string()),
                "cargo" => return Some("build".to_string()),
                _ => {}
            }
        }

        None
    }
}

pub struct StringHint(String);
impl Hint for StringHint {
    fn display(&self) -> &str { &self.0 }
    fn completion(&self) -> Option<&str> { Some(&self.0) }
}

pub struct ShellHelper {
    pub trie: CommandTrie,
    pub macro_manager: std::sync::Arc<std::sync::Mutex<crate::engine::macros::MacroManager>>,
}

impl ShellHelper {
    pub fn new(macro_manager: std::sync::Arc<std::sync::Mutex<crate::engine::macros::MacroManager>>) -> Self {
        Self { 
            trie: CommandTrie::new(),
            macro_manager,
        }
    }
}

impl Helper for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = rustyline::completion::Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        crate::ui::completion::ChevCompleter::complete(line, pos, &self.macro_manager)
    }
}

impl Hinter for ShellHelper {
    type Hint = StringHint;
    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        // 1. Check for AI Suggestions (if line is empty or starts a fix)
        {
            let macros = self.macro_manager.lock().unwrap();
            if let Some(suggestion) = &macros.last_suggestion {
                if line.is_empty() || suggestion.starts_with(line) {
                    return Some(StringHint(suggestion[line.len()..].to_string()));
                }
            }
        }

        if line.is_empty() { return None; }

        // 2. Check for Abbreviations (Shadow expansion hint)
        {
            let macros = self.macro_manager.lock().unwrap();
            if let Some(expansion) = macros.get_abbreviation(line.trim()) {
                return Some(StringHint(format!(" ({})", expansion)));
            }
        }

        // 3. Fallback to History
        self.trie.suggest(line).map(StringHint)
    }
}

impl Highlighter for ShellHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        // Ghost text in Dim Gray
        Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }
}

impl Validator for ShellHelper {}
