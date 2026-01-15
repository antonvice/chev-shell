use chev_shell::engine::executor::execute_command;
use chev_shell::engine::jobs::JobManager;
use chev_shell::engine::env::EnvManager;
use chev_shell::engine::macros::MacroManager;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn test_error_capture() {
    let jobs = Arc::new(Mutex::new(JobManager::new()));
    let env_manager = Arc::new(Mutex::new(EnvManager::new()));
    let macro_manager = Arc::new(Mutex::new(MacroManager::new()));

    // Run a command that definitely fails
    let result = execute_command("nonexistentcommand", &jobs, &env_manager, &macro_manager).await;
    
    // It should return an error
    assert!(result.is_err());

    // Check if MacroManager captured the error
    let macros = macro_manager.lock().unwrap();
    assert!(macros.last_error.is_some());
    let (cmd, err) = macros.last_error.as_ref().unwrap();
    assert_eq!(cmd, "nonexistentcommand");
    // Depending on the OS, the error message might vary, but it should contain somewhat of a "no such file" message
    println!("Captured Stderr: {}", err);
}

#[tokio::test]
async fn test_success_clears_error() {
    let jobs = Arc::new(Mutex::new(JobManager::new()));
    let env_manager = Arc::new(Mutex::new(EnvManager::new()));
    let macro_manager = Arc::new(Mutex::new(MacroManager::new()));

    // 1. Fail first
    let _ = execute_command("sh -c \"exit 1\"", &jobs, &env_manager, &macro_manager).await;
    {
        let macros = macro_manager.lock().unwrap();
        assert!(macros.last_error.is_some());
    }

    // 2. Succeed then
    let _ = execute_command("sh -c \"exit 0\"", &jobs, &env_manager, &macro_manager).await;
    {
        let macros = macro_manager.lock().unwrap();
        assert!(macros.last_error.is_none());
    }
}
