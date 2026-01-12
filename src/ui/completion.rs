use rustyline::completion::Pair;
use rustyline::Result;
use std::path::Path;

pub struct ChevCompleter;

impl ChevCompleter {
    pub fn complete(line: &str, pos: usize) -> Result<(usize, Vec<Pair>)> {
        let (before, _) = line.split_at(pos);
        let parts: Vec<&str> = before.split_whitespace().collect();

        if parts.is_empty() {
            return Ok((pos, Vec::new()));
        }

        // 1. Git Completion
        if parts[0] == "git" {
            return Ok(Self::complete_git(parts.get(1).copied(), before, pos));
        }

        // 2. Docker Completion
        if parts[0] == "docker" {
            return Ok(Self::complete_docker(parts.get(1).copied(), before, pos));
        }

        // 3. Fallback to File/Path Completion
        Self::complete_path(before, pos)
    }

    fn complete_git(subcmd: Option<&str>, _full: &str, pos: usize) -> (usize, Vec<Pair>) {
        let commands = vec![
            ("add", "Add file contents to the index"),
            ("commit", "Record changes to the repository"),
            ("push", "Update remote refs along with associated objects"),
            ("pull", "Fetch from and integrate with another repository"),
            ("status", "Show the working tree status"),
            ("checkout", "Switch branches or restore working tree files"),
            ("branch", "List, create, or delete branches"),
            ("diff", "Show changes between commits, commit and working tree"),
        ];

        let mut matches = Vec::new();
        let target = subcmd.unwrap_or("");
        
        for (name, desc) in commands {
            if name.starts_with(target) {
                matches.push(Pair {
                    display: format!("{} - {}", name, desc),
                    replacement: name.to_string(),
                });
            }
        }

        let start = pos - target.len();
        (start, matches)
    }

    fn complete_docker(subcmd: Option<&str>, _full: &str, pos: usize) -> (usize, Vec<Pair>) {
        let commands = vec![
            ("run", "Run a command in a new container"),
            ("ps", "List containers"),
            ("images", "List images"),
            ("stop", "Stop one or more running containers"),
            ("rm", "Remove one or more containers"),
            ("build", "Build an image from a Dockerfile"),
            ("exec", "Run a command in a running container"),
        ];

        let mut matches = Vec::new();
        let target = subcmd.unwrap_or("");

        for (name, desc) in commands {
            if name.starts_with(target) {
                matches.push(Pair {
                    display: format!("{} - {}", name, desc),
                    replacement: name.to_string(),
                });
            }
        }

        let start = pos - target.len();
        (start, matches)
    }

    fn complete_path(before: &str, pos: usize) -> Result<(usize, Vec<Pair>)> {
        // Simple path completion using standard library
        let last_word = before.split_whitespace().last().unwrap_or("");
        let path = Path::new(last_word);
        
        let (dir, file_part) = if last_word.ends_with('/') {
            (last_word, "")
        } else {
            let parent = path.parent().and_then(|p| p.to_str()).unwrap_or("");
            let file = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            (parent, file)
        };

        let search_dir = if dir.is_empty() { "." } else { dir };
        let mut matches = Vec::new();

        if let Ok(entries) = std::fs::read_dir(search_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(file_part) {
                    let mut replacement = name.clone();
                    if entry.path().is_dir() {
                        replacement.push('/');
                    }
                    matches.push(Pair {
                        display: name,
                        replacement,
                    });
                }
            }
        }

        let start = pos - file_part.len();
        Ok((start, matches))
    }
}
