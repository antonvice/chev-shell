use chev_shell::ai::{OllamaClient, MimicManager};
use tempfile::tempdir;

#[tokio::test]
async fn test_end_to_end_semantic_history() {
    // 1. Setup temporary LanceDB
    let dir = tempdir().expect("Failed to create temp dir");
    let mimic = MimicManager::new_at_path(dir.path().to_path_buf());
    
    // 2. Setup Ollama Client
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
    let client = OllamaClient::new(model);
    
    // 3. Test Embedding + Storage
    let cmd = "cat README.md | grep usage";
    let vector = client.embeddings(cmd.to_string()).await.expect("Failed to get embeddings from Ollama");
    
    // Ensure we got a non-empty vector
    assert!(!vector.is_empty());
    
    mimic.add_command(cmd, vector).await.expect("Failed to add command to Mimic");
    
    // 4. Test Embedding + Search
    let query = "find usage in documentation";
    let search_vector = client.embeddings(query.to_string()).await.expect("Failed to get search embeddings");
    
    let results = mimic.search(search_vector, 1).await.expect("Failed to search Mimic");
    
    assert!(!results.is_empty());
    assert_eq!(results[0], cmd);
}
