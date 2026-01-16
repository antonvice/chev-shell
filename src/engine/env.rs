use std::collections::HashMap;
use std::path::PathBuf;
use std::env;
use anyhow::{Result, anyhow};

pub struct EnvManager {
    // Persistent environment variables (inherited + exported)
    vars: HashMap<String, String>,
    // Directory stack for pushd/popd
    dir_stack: Vec<PathBuf>,
}

impl EnvManager {
    pub fn new() -> Self {
        let vars: HashMap<String, String> = env::vars().collect();
        Self {
            vars,
            dir_stack: Vec::new(),
        }
    }

    pub fn set_var(&mut self, key: String, value: String) {
        unsafe {
            env::set_var(&key, &value);
        }
        self.vars.insert(key, value);
    }

    pub fn remove_var(&mut self, key: &str) {
        unsafe {
            env::remove_var(key);
        }
        self.vars.remove(key);
    }

    pub fn get_var(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    pub fn get_all_vars(&self) -> &HashMap<String, String> {
        &self.vars
    }

    // Directory Stack Logic
    pub fn pushd(&mut self, path: PathBuf) -> Result<()> {
        let current = env::current_dir()?;
        env::set_current_dir(&path).map_err(|e| anyhow!("pushd failed: {}", e))?;
        self.dir_stack.push(current);
        Ok(())
    }

    pub fn popd(&mut self) -> Result<PathBuf> {
        if let Some(prev) = self.dir_stack.pop() {
            env::set_current_dir(&prev).map_err(|e| anyhow!("popd failed: {}", e))?;
            Ok(prev)
        } else {
            Err(anyhow!("popd: directory stack empty"))
        }
    }

    pub fn get_stack(&self) -> Vec<String> {
        let mut stack = Vec::new();
        if let Ok(current) = env::current_dir() {
            stack.push(current.to_string_lossy().to_string());
        }
        for path in self.dir_stack.iter().rev() {
            stack.push(path.to_string_lossy().to_string());
        }
        stack
    }

    // Modern feature: Smart Path addition
    pub fn add_to_path(&mut self, new_path: &str, at_front: bool) {
        let current_path = env::var("PATH").unwrap_or_default();
        let mut paths: Vec<PathBuf> = env::split_paths(&current_path).collect();

        // Deduplicate
        let target = PathBuf::from(new_path);
        paths.retain(|p| p != &target);

        if at_front {
            paths.insert(0, target);
        } else {
            paths.push(target);
        }

        if let Ok(joined) = env::join_paths(paths) {
            let joined_str = joined.to_string_lossy().to_string();
            self.set_var("PATH".to_string(), joined_str);
        }
    }

    pub fn setup_isolated_bin(&mut self) -> Result<String> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        let chev_bin = home.join(".chev").join("bin");
        
        if !chev_bin.exists() {
            std::fs::create_dir_all(&chev_bin)?;
        }

        let path_str = chev_bin.to_string_lossy().to_string();
        self.add_to_path(&path_str, true);
        Ok(path_str)
    }
}
