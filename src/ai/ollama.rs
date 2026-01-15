use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use reqwest::Client;

#[allow(dead_code)]
#[derive(Serialize, Debug)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub message: Message,
    pub done: bool,
}

#[derive(Serialize, Debug)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GenerateResponse {
    pub response: String,
    pub done: bool,
}

#[derive(Serialize, Debug)]
pub struct EmbeddingsRequest {
    pub model: String,
    pub prompt: String,
}

#[derive(Deserialize, Debug)]
pub struct EmbeddingsResponse {
    pub embedding: Vec<f32>,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    pub fn new(model: String) -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:11434".to_string(),
            model,
        }
    }

    pub async fn chat(&self, messages: Vec<Message>, json_format: bool) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            format: if json_format { Some("json".to_string()) } else { None },
        };

        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Ollama error: {}", error_text));
        }

        let resp: ChatResponse = response.json().await?;
        Ok(resp.message.content)
    }

    pub async fn generate(&self, prompt: String, json_format: bool) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);
        let request = GenerateRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            format: if json_format { Some("json".to_string()) } else { None },
        };

        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Ollama error: {}", error_text));
        }

        let resp: GenerateResponse = response.json().await?;
        Ok(resp.response)
    }

    pub async fn embeddings(&self, prompt: String) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);
        let request = EmbeddingsRequest {
            model: self.model.clone(),
            prompt,
        };

        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Ollama error: {}", error_text));
        }

        let resp: EmbeddingsResponse = response.json().await?;
        Ok(resp.embedding)
    }
}
