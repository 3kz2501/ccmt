use super::Provider;
use anyhow::{Context, Result, bail};
use std::process::Command;

pub struct ClaudeCliProvider;

impl Provider for ClaudeCliProvider {
    fn generate(&self, prompt: &str, system: &str, model: &str) -> Result<String> {
        let mut cmd = Command::new("claude");
        cmd.args(["-p", prompt, "--output-format", "text"]);

        if !system.is_empty() {
            cmd.args(["--system-prompt", system]);
        }

        if !model.is_empty() {
            cmd.args(["--model", model]);
        }

        let output = cmd
            .output()
            .context("Failed to run 'claude' CLI. Is it installed? Install with: npm install -g @anthropic-ai/claude-code")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("claude CLI failed: {}", stderr.trim());
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            bail!("claude CLI returned empty response");
        }

        Ok(text)
    }
}
