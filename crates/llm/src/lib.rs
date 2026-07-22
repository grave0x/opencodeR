// LLM provider abstraction — calls real AI models

use opencode_r_schema::session_message::{MessageContent, MessageRole, SessionMessage};
use opencode_r_schema::session_id::SessionID;

/// Provider configuration from environment
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub anthropic_key: Option<String>,
    pub openai_key: Option<String>,
}

impl ProviderConfig {
    pub fn from_env() -> Self {
        Self {
            anthropic_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            openai_key: std::env::var("OPENAI_API_KEY").ok(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.anthropic_key.is_some() || self.openai_key.is_some()
    }
}

/// A single LLM response
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Provider trait — implemented by each AI provider
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn complete(&self, prompt: &str, model: &str) -> Result<LlmResponse, String>;
}

/// Anthropic Claude provider
pub struct AnthropicProvider {
    api_key: String,
    http: reqwest::blocking::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: reqwest::blocking::Client::new(),
        }
    }
}

impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &'static str { "anthropic" }

    fn complete(&self, prompt: &str, model: &str) -> Result<LlmResponse, String> {
        let model = if model.is_empty() { "claude-sonnet-4-20250514" } else { model };

        let body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "messages": [{"role": "user", "content": prompt}]
        });

        let resp = self.http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("Anthropic API error: {}", e))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().map_err(|e| format!("Parse error: {}", e))?;

        if !status.is_success() {
            let err = body["error"]["message"].as_str().unwrap_or("unknown");
            return Err(format!("Anthropic API ({}): {}", status.as_u16(), err));
        }

        let content = body["content"][0]["text"].as_str().unwrap_or("").to_string();
        let input_tokens = body["usage"]["input_tokens"].as_u64().unwrap_or(0);
        let output_tokens = body["usage"]["output_tokens"].as_u64().unwrap_or(0);

        Ok(LlmResponse { content, model: model.to_string(), input_tokens, output_tokens })
    }
}

/// OpenAI GPT provider
pub struct OpenAIProvider {
    api_key: String,
    http: reqwest::blocking::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: reqwest::blocking::Client::new(),
        }
    }
}

impl LlmProvider for OpenAIProvider {
    fn name(&self) -> &'static str { "openai" }

    fn complete(&self, prompt: &str, model: &str) -> Result<LlmResponse, String> {
        let model = if model.is_empty() { "gpt-4o" } else { model };

        let body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "messages": [{"role": "user", "content": prompt}]
        });

        let resp = self.http
            .post("https://api.openai.com/v1/chat/completions")
            .header("authorization", format!("Bearer {}", &self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("OpenAI API error: {}", e))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().map_err(|e| format!("Parse error: {}", e))?;

        if !status.is_success() {
            let err = body["error"]["message"].as_str().unwrap_or("unknown");
            return Err(format!("OpenAI API ({}): {}", status.as_u16(), err));
        }

        let content = body["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
        let usage = &body["usage"];
        let input_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0);
        let output_tokens = usage["completion_tokens"].as_u64().unwrap_or(0);

        Ok(LlmResponse { content, model: model.to_string(), input_tokens, output_tokens })
    }
}

/// Select a provider based on the model reference (e.g. "anthropic/claude-sonnet-4")
pub fn provider_for_model(config: &ProviderConfig, model: &str) -> Option<Box<dyn LlmProvider>> {
    if model.contains("anthropic") || model.contains("claude") {
        config.anthropic_key.clone().map(AnthropicProvider::new).map(|p| Box::new(p) as Box<dyn LlmProvider>)
    } else if model.contains("openai") || model.contains("gpt-") {
        config.openai_key.clone().map(OpenAIProvider::new).map(|p| Box::new(p) as Box<dyn LlmProvider>)
    } else {
        // Fallback: try anthropic first, then openai
        config.anthropic_key.clone()
            .map(AnthropicProvider::new)
            .map(|p| Box::new(p) as Box<dyn LlmProvider>)
            .or_else(|| config.openai_key.clone().map(OpenAIProvider::new).map(|p| Box::new(p) as Box<dyn LlmProvider>))
    }
}
