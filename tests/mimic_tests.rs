use chev_shell::ai::MimicManager;

#[tokio::test]
async fn test_mimic_store_and_search() {
    let mimic = MimicManager::new();
    
    // Create dummy vectors (mocking embeddings)
    // Most embedding models use 1024, 1536, 4096 etc.
    // For test, we'll use a small dimension if we initialize the table first.
    let vector_a = vec![1.0, 0.0, 0.0];
    let vector_b = vec![0.0, 1.0, 0.0];
    
    // In a real scenario, we don't know the dim until first add.
    // MimicManager initializes dim on first create_table.
    
    let result_a = mimic.add_command("git pull origin master", vector_a.clone()).await;
    assert!(result_a.is_ok());
    
    let result_b = mimic.add_command("docker-compose up -d", vector_b.clone()).await;
    assert!(result_b.is_ok());
    
    // Search for something close to vector_a
    let search_vector = vec![0.9, 0.1, 0.0];
    let results = mimic.search(search_vector, 1).await.unwrap();
    
    assert!(!results.is_empty());
    assert_eq!(results[0], "git pull origin master");
}
