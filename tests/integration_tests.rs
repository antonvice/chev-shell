use chev_shell::ui::protocol::{RioAction, send_rio};
use chev_shell::engine::executor::AiChecker;

#[tokio::test]
async fn test_osc_sequences() {
    // This is more of a "dry run" to ensure types and logic are sound
    // Since send_rio prints to stdout, we can't easily capture it in a test 
    // without redirecting stdout, but we can verify the logic.
    
    let action = RioAction::Notify { 
        title: "Test".to_string(), 
        message: "Message".to_string() 
    };
    
    // Just verify it doesn't panic
    send_rio(action);
}

#[tokio::test]
async fn test_ai_checker_init() {
    let checker = AiChecker::new();
    // We don't want to rely on Ollama running in CI/Tests, 
    // but we can check if the methods exist and are callable.
    let _ = checker.is_ollama_running().await;
}
