mod cli;
mod config;
mod format;
mod git;
mod hook;
mod prompt;
mod provider;

use anyhow::{Result, bail};
use clap::Parser;
use cli::{Cli, Commands, ConfigAction, HookAction};
use colored::Colorize;
use dialoguer::{Input, Select};

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e:#}", "error:".red().bold());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(cmd) = &cli.command {
        return match cmd {
            Commands::Config { action } => match action {
                ConfigAction::Init => config::init_config(),
                ConfigAction::Show => config::show_config(
                    cli.provider.as_deref(),
                    cli.model.as_deref(),
                    cli.language.as_deref(),
                ),
            },
            Commands::Hook { action } => match action {
                HookAction::Install => hook::install(),
                HookAction::Remove => hook::remove(),
            },
        };
    }

    // Main commit flow
    if !git::is_git_repo() {
        bail!("Not a git repository. Run this from inside a git repo.");
    }

    let cfg = config::load_config(
        cli.provider.as_deref(),
        cli.model.as_deref(),
        cli.language.as_deref(),
    )?;

    // Auto-stage if configured
    if cfg.commit.auto_stage {
        git::git_stage_all()?;
    }

    // Get diff
    let diff = git::git_diff_staged()?;
    if diff.is_empty() {
        // Try unstaged diff as fallback info
        let unstaged = git::git_diff_all()?;
        if unstaged.is_empty() {
            bail!("No changes to commit. Stage changes with 'git add' first.");
        }
        bail!(
            "No staged changes. Stage changes with 'git add' first, or set auto_stage = true in config."
        );
    }

    let status = git::git_status().unwrap_or_default();

    // Resolve API key from config or env
    let api_key = if cfg.auth.api_key.is_empty() {
        std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()
    } else {
        cfg.auth.api_key.clone()
    };

    let provider = provider::create_provider(&cfg.auth.provider, &api_key)?;

    let system_prompt = prompt::build_system_prompt(&cfg);
    let user_prompt = prompt::build_user_prompt(
        &diff,
        &status,
        cli.message.as_deref(),
        cfg.prompt.max_diff_length,
    );

    // Generate initial message
    eprintln!("{}", "Generating commit message...".dimmed());
    let raw = provider.generate(&user_prompt, &system_prompt, &cfg.auth.model)?;
    let mut message = format::format_message(&raw);

    // Dry-run mode
    if cli.dry_run {
        println!("{message}");
        return Ok(());
    }

    // Confirmation loop
    let should_confirm = cfg.commit.confirm && !cli.no_confirm;

    if should_confirm {
        loop {
            // Display message
            eprintln!();
            eprintln!("{}", "Generated commit message:".bold());
            eprintln!("{}", "─".repeat(40).dimmed());
            eprintln!("{message}");
            eprintln!("{}", "─".repeat(40).dimmed());
            eprintln!();

            let choices = &[
                "Yes - commit with this message",
                "Edit - revise the message",
                "No - cancel",
            ];
            let selection = Select::new()
                .with_prompt("Commit with this message?")
                .items(choices)
                .default(0)
                .interact_opt()?;

            match selection {
                Some(0) => break, // Yes
                Some(1) => {
                    // Edit
                    let instruction: String = Input::new()
                        .with_prompt("Describe what to change")
                        .interact_text()?;

                    let edit_prompt = prompt::build_edit_prompt(
                        &diff,
                        &status,
                        &message,
                        &instruction,
                        cli.message.as_deref(),
                        cfg.prompt.max_diff_length,
                    );

                    eprintln!("{}", "Regenerating...".dimmed());
                    let raw = provider.generate(&edit_prompt, &system_prompt, &cfg.auth.model)?;
                    message = format::format_message(&raw);
                }
                Some(2) | None => {
                    // No or Ctrl+C
                    eprintln!("{}", "Cancelled.".yellow());
                    return Ok(());
                }
                _ => unreachable!(),
            }
        }
    }

    // Commit
    git::git_commit(&message)?;
    eprintln!(
        "{} {}",
        "Committed:".green().bold(),
        message.lines().next().unwrap_or("")
    );

    // Auto-push
    let should_push = cfg.commit.auto_push || cli.push;
    if should_push {
        eprintln!("{}", "Pushing...".dimmed());
        git::git_push()?;
        eprintln!("{}", "Pushed.".green());
    }

    Ok(())
}
