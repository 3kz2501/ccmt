use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ccm", about = "AI-powered commit message generator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Context hint for the AI
    #[arg(short, long)]
    pub message: Option<String>,

    /// Generate message only, don't commit
    #[arg(short, long)]
    pub dry_run: bool,

    /// Push after commit
    #[arg(long)]
    pub push: bool,

    /// Skip confirmation prompt
    #[arg(long)]
    pub no_confirm: bool,

    /// Override auth provider (cli or api)
    #[arg(long)]
    pub provider: Option<String>,

    /// Override model
    #[arg(long)]
    pub model: Option<String>,

    /// Override language (en or ja)
    #[arg(long)]
    pub language: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Manage git hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Generate default global config
    Init,
    /// Show merged config
    Show,
}

#[derive(Subcommand, Debug)]
pub enum HookAction {
    /// Install prepare-commit-msg hook
    Install,
    /// Remove installed hook
    Remove,
}
