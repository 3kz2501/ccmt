pub mod api;
pub mod claude_cli;

use anyhow::Result;

pub trait Provider {
    fn generate(&self, prompt: &str, system: &str, model: &str) -> Result<String>;
}

pub fn create_provider(name: &str, api_key: &str) -> Result<Box<dyn Provider>> {
    match name {
        "cli" => Ok(Box::new(claude_cli::ClaudeCliProvider)),
        "api" => Ok(Box::new(api::ApiProvider::new(api_key)?)),
        other => anyhow::bail!("Unknown provider: {other}. Use 'cli' or 'api'."),
    }
}
