// LLM provider abstraction — calls real AI models

/// Provider configuration from environment
#[derive(Debug, Clone, Default)]
pub struct ProviderConfig {
    pub anthropic_key: Option<String>,
    pub openai_key: Option<String>,
    pub kimi_key: Option<String>,
    pub goose_key: Option<String>,
    pub pi_key: Option<String>,
    pub opencode_key: Option<String>,
}

impl ProviderConfig {
    pub fn from_env() -> Self {
        Self {
            anthropic_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            openai_key: std::env::var("OPENAI_API_KEY").ok(),
            kimi_key: std::env::var("KIMI_API_KEY").ok()
                .or_else(|| std::env::var("MOONSHOT_API_KEY").ok()),
            goose_key: std::env::var("GOOSE_API_KEY").ok(),
            pi_key: std::env::var("PI_API_KEY").ok(),
            opencode_key: std::env::var("OPENCODE_API_KEY").ok(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.anthropic_key.is_some() || self.openai_key.is_some()
            || self.kimi_key.is_some() || self.goose_key.is_some()
            || self.pi_key.is_some() || self.opencode_key.is_some()
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

// ── Anthropic Claude ────────────────────────────────────────────────────

pub struct AnthropicProvider {
    api_key: String,
    http: reqwest::blocking::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key, http: reqwest::blocking::Client::new() }
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
            .json(&body)
            .send()
            .map_err(|e| format!("Anthropic error: {}", e))?;
        let status = resp.status();
        let body: serde_json::Value = resp.json().map_err(|e| format!("Parse: {}", e))?;
        if !status.is_success() {
            return Err(format!("Anthropic ({}): {}", status.as_u16(),
                body["error"]["message"].as_str().unwrap_or("unknown")));
        }
        Ok(LlmResponse {
            content: body["content"][0]["text"].as_str().unwrap_or("").to_string(),
            model: model.to_string(),
            input_tokens: body["usage"]["input_tokens"].as_u64().unwrap_or(0),
            output_tokens: body["usage"]["output_tokens"].as_u64().unwrap_or(0),
        })
    }
}

// ── OpenAI Direct ───────────────────────────────────────────────────────

pub struct OpenAIDirectProvider {
    api_key: String,
    http: reqwest::blocking::Client,
}

impl OpenAIDirectProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key, http: reqwest::blocking::Client::new() }
    }
}

impl LlmProvider for OpenAIDirectProvider {
    fn name(&self) -> &'static str { "openai" }

    fn complete(&self, prompt: &str, model: &str) -> Result<LlmResponse, String> {
        call_openai_compatible(&self.http, "https://api.openai.com/v1", &self.api_key, prompt, model)
    }
}

// ── OpenAI-Compatible Generic ───────────────────────────────────────────

/// Generic provider for any OpenAI-compatible chat completions API.
pub struct OpenAICompatibleProvider {
    name: &'static str,
    base_url: &'static str,
    api_key: String,
    http: reqwest::blocking::Client,
}

impl OpenAICompatibleProvider {
    pub fn new(name: &'static str, base_url: &'static str, api_key: String) -> Self {
        Self { name, base_url, api_key, http: reqwest::blocking::Client::new() }
    }
}

impl LlmProvider for OpenAICompatibleProvider {
    fn name(&self) -> &'static str { self.name }

    fn complete(&self, prompt: &str, model: &str) -> Result<LlmResponse, String> {
        call_openai_compatible(&self.http, self.base_url, &self.api_key, prompt, model)
    }
}

fn call_openai_compatible(
    http: &reqwest::blocking::Client,
    base_url: &str,
    api_key: &str,
    prompt: &str,
    model: &str,
) -> Result<LlmResponse, String> {
    let model = if model.is_empty() || model.contains('/') {
        match model.split('/').nth(1).filter(|m| !m.is_empty()) {
            Some(m) => m,
            None => "gpt-4o",
        }
    } else { model };

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "messages": [{"role": "user", "content": prompt}]
    });

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let resp = http.post(&url)
        .header("authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .map_err(|e| format!("{} error: {}", base_url, e))?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().map_err(|e| format!("Parse: {}", e))?;
    if !status.is_success() {
        return Err(format!("{} ({}): {}",
            base_url, status.as_u16(),
            body["error"]["message"].as_str().unwrap_or("unknown")));
    }

    let content = body["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
    let usage = &body["usage"];
    Ok(LlmResponse {
        content,
        model: model.to_string(),
        input_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0),
        output_tokens: usage["completion_tokens"].as_u64().unwrap_or(0),
    })
}

// ── Provider Profiles ───────────────────────────────────────────────────

/// Known OpenAI-compatible provider profiles
pub const PROFILES: &[(&str, &str, &str)] = &[
    // (name, base_url, env_var_for_api_key)
    ("kimi",       "https://api.moonshot.cn/v1",                    "KIMI_API_KEY"),
    ("moonshot",   "https://api.moonshot.cn/v1",                    "MOONSHOT_API_KEY"),
    ("goose",      "https://api.goose.ai/v1",                       "GOOSE_API_KEY"),
    ("pi",         "https://api.pi.ai/v1",                          "PI_API_KEY"),
    ("opencode",   "http://127.0.0.1:8081/v1",                     "OPENCODE_API_KEY"),
    ("deepseek",   "https://api.deepseek.com/v1",                   "DEEPSEEK_API_KEY"),
    ("groq",       "https://api.groq.com/openai/v1",               "GROQ_API_KEY"),
    ("openrouter", "https://openrouter.ai/api/v1",                  "OPENROUTER_API_KEY"),
    ("xai",        "https://api.x.ai/v1",                           "XAI_API_KEY"),
    ("cerebras",   "https://api.cerebras.ai/v1",                    "CEREBRAS_API_KEY"),
    ("fireworks",  "https://api.fireworks.ai/inference/v1",        "FIREWORKS_API_KEY"),
];

/// Select a provider based on model reference (e.g. "anthropic/claude-sonnet-4")
pub fn provider_for_model(config: &ProviderConfig, model: &str) -> Option<Box<dyn LlmProvider>> {
    // Anthropic
    if model.contains("anthropic") || model.contains("claude") {
        return config.anthropic_key.clone()
            .map(AnthropicProvider::new).map(|p| Box::new(p) as Box<dyn LlmProvider>);
    }
    // Direct OpenAI
    if model.contains("openai") || model.contains("gpt-") || model.contains("o1") || model.contains("o3") {
        return config.openai_key.clone()
            .map(OpenAIDirectProvider::new).map(|p| Box::new(p) as Box<dyn LlmProvider>);
    }
    // Check known profiles
    let model_lower = model.to_lowercase();
    for &(name, base_url, env_var) in PROFILES {
        if model_lower.contains(name) {
            let key = match env_var {
                "KIMI_API_KEY" => config.kimi_key.clone(),
                "MOONSHOT_API_KEY" => config.kimi_key.clone().or_else(|| config.kimi_key.clone()),
                "GOOSE_API_KEY" => config.goose_key.clone(),
                "PI_API_KEY" => config.pi_key.clone(),
                "OPENCODE_API_KEY" => config.opencode_key.clone(),
                _ => std::env::var(env_var).ok(),
            };
            if let Some(k) = key {
                return Some(Box::new(OpenAICompatibleProvider::new(name, base_url, k)));
            }
        }
    }
    // Fallback: try the first configured provider
    if let Some(key) = &config.anthropic_key {
        return Some(Box::new(AnthropicProvider::new(key.clone())));
    }
    if let Some(key) = &config.openai_key {
        return Some(Box::new(OpenAIDirectProvider::new(key.clone())));
    }
    None
}
