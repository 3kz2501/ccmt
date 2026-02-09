use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Config {
    pub auth: AuthConfig,
    pub commit: CommitConfig,
    pub prompt: PromptConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommitConfig {
    pub conventional: bool,
    pub emoji: bool,
    pub language: String,
    pub auto_stage: bool,
    pub auto_push: bool,
    pub confirm: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptConfig {
    pub system: String,
    pub max_diff_length: usize,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            provider: "cli".to_string(),
            api_key: String::new(),
            model: "sonnet".to_string(),
        }
    }
}

impl Default for CommitConfig {
    fn default() -> Self {
        Self {
            conventional: true,
            emoji: false,
            language: "en".to_string(),
            auto_stage: false,
            auto_push: false,
            confirm: true,
        }
    }
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            system: String::new(),
            max_diff_length: 8000,
        }
    }
}

// Partial config for TOML deserialization (all fields optional)
#[derive(Debug, Deserialize, Default)]
struct PartialConfig {
    auth: Option<PartialAuthConfig>,
    commit: Option<PartialCommitConfig>,
    prompt: Option<PartialPromptConfig>,
}

#[derive(Debug, Deserialize)]
struct PartialAuthConfig {
    provider: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PartialCommitConfig {
    conventional: Option<bool>,
    emoji: Option<bool>,
    language: Option<String>,
    auto_stage: Option<bool>,
    auto_push: Option<bool>,
    confirm: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PartialPromptConfig {
    system: Option<String>,
    max_diff_length: Option<usize>,
}

impl Config {
    fn apply_partial(&mut self, partial: PartialConfig) {
        if let Some(auth) = partial.auth {
            if let Some(v) = auth.provider {
                self.auth.provider = v;
            }
            if let Some(v) = auth.api_key {
                self.auth.api_key = v;
            }
            if let Some(v) = auth.model {
                self.auth.model = v;
            }
        }
        if let Some(commit) = partial.commit {
            if let Some(v) = commit.conventional {
                self.commit.conventional = v;
            }
            if let Some(v) = commit.emoji {
                self.commit.emoji = v;
            }
            if let Some(v) = commit.language {
                self.commit.language = v;
            }
            if let Some(v) = commit.auto_stage {
                self.commit.auto_stage = v;
            }
            if let Some(v) = commit.auto_push {
                self.commit.auto_push = v;
            }
            if let Some(v) = commit.confirm {
                self.commit.confirm = v;
            }
        }
        if let Some(prompt) = partial.prompt {
            if let Some(v) = prompt.system {
                self.prompt.system = v;
            }
            if let Some(v) = prompt.max_diff_length {
                self.prompt.max_diff_length = v;
            }
        }
    }
}

/// Get the global config file path: ~/.config/ccmt/config.toml
pub fn global_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not determine config directory")?;
    Ok(config_dir.join("ccmt").join("config.toml"))
}

/// Search for .ccmt.toml starting from `start_dir` and walking up to /
fn find_project_config(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(".ccmt.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn load_partial(path: &Path) -> Result<PartialConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let partial: PartialConfig =
        toml::from_str(&content).with_context(|| format!("Failed to parse: {}", path.display()))?;
    Ok(partial)
}

/// Load merged config: defaults ← global ← project ← CLI overrides
pub fn load_config(
    provider_override: Option<&str>,
    model_override: Option<&str>,
    language_override: Option<&str>,
) -> Result<Config> {
    let mut config = Config::default();

    // Global config
    let global_path = global_config_path()?;
    if global_path.is_file() {
        let partial = load_partial(&global_path)?;
        config.apply_partial(partial);
    }

    // Project config
    let cwd = std::env::current_dir()?;
    if let Some(project_path) = find_project_config(&cwd) {
        let partial = load_partial(&project_path)?;
        config.apply_partial(partial);
    }

    // CLI overrides
    if let Some(p) = provider_override {
        config.auth.provider = p.to_string();
    }
    if let Some(m) = model_override {
        config.auth.model = m.to_string();
    }
    if let Some(l) = language_override {
        config.commit.language = l.to_string();
    }

    Ok(config)
}

/// Generate default global config file
pub fn init_config() -> Result<()> {
    let path = global_config_path()?;
    if path.is_file() {
        anyhow::bail!(
            "Config already exists at {}. Delete it first to re-initialize.",
            path.display()
        );
    }
    let config = Config::default();
    let content = toml::to_string_pretty(&config)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    println!("Created config at {}", path.display());
    Ok(())
}

/// Show the fully merged config
pub fn show_config(
    provider_override: Option<&str>,
    model_override: Option<&str>,
    language_override: Option<&str>,
) -> Result<()> {
    let config = load_config(provider_override, model_override, language_override)?;
    let content = toml::to_string_pretty(&config)?;
    println!("{content}");
    Ok(())
}
