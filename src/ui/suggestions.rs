use rustyline::hint::{Hint, Hinter};
use rustyline::Context;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use rustyline::Helper;
use std::borrow::Cow;

/// Simple but fast history search for suggestions
pub struct CommandTrie {
    commands: Vec<String>,
}

impl CommandTrie {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn add(&mut self, cmd: &str) {
        if cmd.is_empty() { return; }
        // Remove old occurrences to keep it fresh
        self.commands.retain(|x| x != cmd);
        self.commands.push(cmd.to_string());
    }

    pub fn suggest(&self, input: &str) -> Option<String> {
        if input.is_empty() { return None; }
        // Exact match prefix search from most recent
        self.commands.iter().rev()
            .find(|c| c.starts_with(input) && *c != input)
            .map(|c| c[input.len()..].to_string())
    }
}

pub struct StringHint(String);
impl Hint for StringHint {
    fn display(&self) -> &str { &self.0 }
    fn completion(&self) -> Option<&str> { Some(&self.0) }
}

pub struct ShellHelper {
    pub trie: CommandTrie,
}

impl ShellHelper {
    pub fn new() -> Self {
        Self { trie: CommandTrie::new() }
    }
}

impl Helper for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = String;
}

impl Hinter for ShellHelper {
    type Hint = StringHint;
    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
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
