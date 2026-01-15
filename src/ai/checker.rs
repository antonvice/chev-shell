use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Model {
    name: String,
}

#[derive(Deserialize, Debug)]
struct ModelsResponse {
    models: Vec<Model>,
}

pub struct AiChecker {
    client: Client,
    base_url: String,
}

impl AiChecker {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    pub async fn is_ollama_running(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        self.client.get(&url).send().await.is_ok()
    }

    pub async fn has_model(&self, model_name: &str) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(tags) = resp.json::<ModelsResponse>().await {
                    tags.models.iter().any(|m| m.name.contains(model_name))
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    pub async fn pull_model(&self, model_name: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.base_url);
        let payload = serde_json::json!({
            "name": model_name,
            "stream": false
        });

        let resp = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to pull model: {}", resp.status()))
        }
    }
}
