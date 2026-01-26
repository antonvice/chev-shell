use std::io::Write;
use std::process::Command;
use std::path::Path;

#[derive(Clone, Default)]
pub struct PromptParts {
    pub user: String,
    pub hostname: String,
    pub path: String,
    pub git: String,
}

impl PromptParts {
    pub fn to_plain_string(&self) -> String {
        let git_text = if self.git.is_empty() { String::new() } else { format!("({}) ", self.git) };
        format!("🐚 {}@{} {} {}> ", self.user, self.hostname, self.path, git_text)
    }

    pub fn to_colored_string(&self, semantic: bool) -> String {
        let p_start = if semantic { "\x01\x1b]133;A\x07\x02" } else { "" };
        let c_start = if semantic { "\x01\x1b]133;B\x07\x02" } else { "" };
        
        let teal = "\x01\x1b[38;2;110;209;195m\x02";
        let gray = "\x01\x1b[90m\x02";
        let reset = "\x01\x1b[0m\x02";
        let bold_teal = "\x01\x1b[1;38;2;110;209;195m\x02";

        let user_host = format!("{}{}@{}{}{}", teal, self.user, gray, self.hostname, reset);
        let path_str = format!("{}{}{}", bold_teal, self.path, reset);
        
        let git_part = if !self.git.is_empty() { 
            format!(" {}({}){}", gray, self.git, reset) 
        } else { 
            String::new() 
        };

        // Construct: [Start]🐚 [User@Host] [Path][Git] [>][Space][Reset][CmdStart]
        // Explicitly managing spaces to ensure no "double space" or "gap" issues
        format!("{}{} {} {}{}{}{} {}{}", 
            p_start, 
            "🐚", 
            user_host, 
            path_str, 
            git_part, 
            teal, 
            ">",
            reset,
            c_start
        )
    }
}

pub fn pre_prompt() {
    // Set Cursor Style & Color
    // \x1b[6 q  -> Blinking Bar (I-Beam) - Note: Google says 5 is Steady Bar, 6 is Blinking Bar
    // \x1b]12;... -> Set cursor color to #6ED1C3
    print!("\x1b[6 q\x1b]12;#6ED1C3\x07"); 
    std::io::stdout().flush().ok();
}

pub fn get_prompt_parts() -> PromptParts {
    let current_dir = std::env::current_dir().unwrap_or_default();
    let home = dirs::home_dir().unwrap_or_default();
    let display_path = shorten_path(&current_dir, &home);

    let hostname = match get_hostname() {
        Some(h) => h,
        None => "Mac".to_string(),
    };
    
    let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
    let git_section = get_git_info().unwrap_or_default();

    PromptParts {
        user,
        hostname,
        path: display_path,
        git: git_section,
    }
}

pub fn get_prompt() -> String {
    get_prompt_parts().to_plain_string()
}

fn get_hostname() -> Option<String> {
    let output = Command::new("hostname").output().ok()?;
    let host = String::from_utf8_lossy(&output.stdout).trim().to_string();
    host.split('.').next().map(|s| s.to_string())
}

fn get_git_info() -> Option<String> {
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if !branch.status.success() {
        return None;
    }

    let branch_name = String::from_utf8_lossy(&branch.stdout).trim().to_string();
    
    let status = Command::new("git").args(["status", "--porcelain"]).output().ok()?;
    let is_dirty = !String::from_utf8_lossy(&status.stdout).trim().is_empty();
    
    let gray = "\x01\x1b[90m\x02";
    let reset = "\x01\x1b[0m\x02";
    let dirty_marker = if is_dirty { "*" } else { "" };
    
    // Format: (branch_name*) 
    Some(format!("{}({}{}){}", gray, branch_name, dirty_marker, reset))
}

fn shorten_path(path: &Path, home: &Path) -> String {
    let path_str = path.to_string_lossy();
    let home_str = home.to_string_lossy();
    
    let relative_path = if path_str.starts_with(&*home_str) {
        path_str.replacen(&*home_str, "~", 1)
    } else {
        path_str.to_string()
    };

    if relative_path == "/" || relative_path == "~" {
        return relative_path;
    }

    let parts: Vec<&str> = relative_path.split('/').collect();
    let mut shortened = Vec::new();

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() && i == 0 {
            // Root
        } else if *part == "~" {
            shortened.push("~".to_string());
        } else if i == parts.len() - 1 {
            shortened.push(part.to_string()); 
        } else {
            if let Some(c) = part.chars().next() {
                shortened.push(c.to_string());
            }
        }
    }

    if relative_path.starts_with('/') {
        format!("/{}", shortened.join("/"))
    } else {
        shortened.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_shorten_path() {
        let home = PathBuf::from("/Users/antonvice");
        let path = PathBuf::from("/Users/antonvice/Documents/programming/chev-shell/chev-shell");
        assert_eq!(shorten_path(&path, &home), "~/D/p/c/chev-shell");
    }
}
