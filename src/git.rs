use anyhow::{Context, Result, bail};
use std::process::Command;

fn run_git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn is_git_repo() -> bool {
    run_git(&["rev-parse", "--is-inside-work-tree"]).is_ok()
}

pub fn git_diff_staged() -> Result<String> {
    run_git(&["diff", "--cached"])
}

pub fn git_diff_all() -> Result<String> {
    run_git(&["diff"])
}

pub fn git_status() -> Result<String> {
    run_git(&["status", "--porcelain"])
}

pub fn git_stage_all() -> Result<String> {
    run_git(&["add", "."])
}

pub fn git_commit(msg: &str) -> Result<String> {
    run_git(&["commit", "-m", msg])
}

pub fn git_push() -> Result<String> {
    run_git(&["push"])
}
