use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs;
use anyhow::{Result, Context};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Macro {
    pub name: String,
    pub template: String,
}

pub struct MacroManager {
    macros: HashMap<String, Macro>,
    abbreviations: HashMap<String, String>,
    config_path: std::path::PathBuf,
    pub last_suggestion: Option<String>,
    pub last_error: Option<(String, String)>, // (command, stderr)
}

impl MacroManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let chev_dir = home.join(".chev");
        if !chev_dir.exists() {
            let _ = std::fs::create_dir_all(&chev_dir);
        }
        let config_path = chev_dir.join("macros.json");
        
        let mut manager = Self {
            macros: HashMap::new(),
            abbreviations: HashMap::new(),
            config_path: config_path.clone(),
            last_suggestion: None,
            last_error: None,
        };

        let _ = manager.load();
        manager
    }

    pub fn set_macro(&mut self, name: String, template: String) -> Result<()> {
        self.macros.insert(name.clone(), Macro { name, template });
        self.save()
    }

    pub fn unset_macro(&mut self, name: &str) -> Result<()> {
        self.macros.remove(name);
        self.save()
    }

    pub fn set_abbreviation(&mut self, name: String, expansion: String) {
        self.abbreviations.insert(name, expansion);
    }

    pub fn get_abbreviation(&self, name: &str) -> Option<&String> {
        self.abbreviations.get(name)
    }

    pub fn expand_macro(&self, input: &str) -> Option<String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() { return None; }

        if let Some(m) = self.macros.get(parts[0]) {
            let mut result = m.template.clone();
            let args = &parts[1..];
            
            // Modern feature: Smart placeholder replacement. 
            // Replace $1, $2 with specific args, or $ with all remaining args joined.
            if result.contains('$') {
                for (i, arg) in args.iter().enumerate() {
                    let placeholder = format!("${}", i + 1);
                    result = result.replace(&placeholder, arg);
                }
                // Universal placeholder $ for "the rest"
                result = result.replace("$", &args.join(" "));
            } else {
                // If no placeholders, just append args
                if !args.is_empty() {
                    result.push(' ');
                    result.push_str(&args.join(" "));
                }
            }
            return Some(result);
        }
        None
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&(&self.macros, &self.abbreviations))?;
        fs::write(&self.config_path, json).context("Failed to save macros")
    }

    fn load(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            let (macros, abbreviations): (HashMap<String, Macro>, HashMap<String, String>) = serde_json::from_str(&content)?;
            self.macros = macros;
            self.abbreviations = abbreviations;
        }
        Ok(())
    }

    pub fn list(&self) -> &HashMap<String, Macro> {
        &self.macros
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_expansion() {
        let mut manager = MacroManager::new();
        manager.set_macro("g".to_string(), "git".to_string()).unwrap();
        
        let expanded = manager.expand_macro("g commit -m 'feat'").unwrap();
        assert_eq!(expanded, "git commit -m 'feat'");

        manager.set_macro("grep-src".to_string(), "rg $1 src/".to_string()).unwrap();
        let expanded = manager.expand_macro("grep-src hello").unwrap();
        assert_eq!(expanded, "rg hello src/");
    }

    #[test]
    fn test_macro_placeholders() {
        let mut manager = MacroManager::new();
        manager.set_macro("dual".to_string(), "echo $1 $2".to_string()).unwrap();
        let expanded = manager.expand_macro("dual first second").unwrap();
        assert_eq!(expanded, "echo first second");

        manager.set_macro("rest".to_string(), "echo all: $".to_string()).unwrap();
        let expanded = manager.expand_macro("rest one two three").unwrap();
        assert_eq!(expanded, "echo all: one two three");
    }

    #[test]
    fn test_abbreviations() {
        let mut manager = MacroManager::new();
        manager.set_abbreviation("gco".to_string(), "git checkout".to_string());
        assert_eq!(manager.get_abbreviation("gco"), Some(&"git checkout".to_string()));
    }
}
