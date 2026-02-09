use super::Provider;
use anyhow::{Result, bail};
use serde_json::json;

pub struct ApiProvider {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl ApiProvider {
    pub fn new(api_key: &str) -> Result<Self> {
        if api_key.is_empty() {
            bail!(
                "API key is required for 'api' provider. Set auth.api_key in config or use ANTHROPIC_API_KEY env var."
            );
        }
        Ok(Self {
            api_key: api_key.to_string(),
            client: reqwest::blocking::Client::new(),
        })
    }
}

impl Provider for ApiProvider {
    fn generate(&self, prompt: &str, system: &str, model: &str) -> Result<String> {
        let model_id = resolve_model(model);

        let messages = vec![json!({
            "role": "user",
            "content": prompt,
        })];

        let mut body = json!({
            "model": model_id,
            "max_tokens": 1024,
            "messages": messages,
        });

        if !system.is_empty() {
            body["system"] = json!(system);
        }

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            bail!("Anthropic API error ({}): {}", status, text);
        }

        let json: serde_json::Value = resp.json()?;
        let text = json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Unexpected API response format"))?
            .trim()
            .to_string();

        if text.is_empty() {
            bail!("API returned empty response");
        }

        Ok(text)
    }
}

fn resolve_model(alias: &str) -> &str {
    match alias {
        "sonnet" => "claude-sonnet-4-5-20250929",
        "haiku" => "claude-haiku-4-5-20251001",
        "opus" => "claude-opus-4-6",
        other => other,
    }
}
