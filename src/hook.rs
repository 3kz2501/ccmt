use anyhow::{Context, Result, bail};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const HOOK_CONTENT: &str = r#"#!/bin/sh
# Installed by ccmt - AI commit message generator
# This hook generates a commit message using ccmt

# Only run for normal commits (not merge, squash, etc.)
case "$2" in
  merge|squash)
    exit 0
    ;;
esac

# Generate message and write to commit message file
MSG=$(ccmt --dry-run --no-confirm 2>/dev/null)
if [ $? -eq 0 ] && [ -n "$MSG" ]; then
    echo "$MSG" > "$1"
fi
"#;

fn hook_path() -> Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .context("Failed to find git directory")?;

    if !output.status.success() {
        bail!("Not a git repository");
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(git_dir)
        .join("hooks")
        .join("prepare-commit-msg"))
}

pub fn install() -> Result<()> {
    let path = hook_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Backup existing hook
    if path.exists() {
        let backup = path.with_extension("msg.bak");
        fs::copy(&path, &backup)
            .with_context(|| format!("Failed to backup existing hook to {}", backup.display()))?;
        println!("Backed up existing hook to {}", backup.display());
    }

    fs::write(&path, HOOK_CONTENT)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;

    println!("Installed prepare-commit-msg hook at {}", path.display());
    Ok(())
}

pub fn remove() -> Result<()> {
    let path = hook_path()?;

    if !path.exists() {
        println!("No hook found at {}", path.display());
        return Ok(());
    }

    // Check if it's our hook
    let content = fs::read_to_string(&path)?;
    if !content.contains("ccmt") {
        bail!(
            "Hook at {} was not installed by ccmt. Remove manually if intended.",
            path.display()
        );
    }

    fs::remove_file(&path)?;
    println!("Removed hook at {}", path.display());

    // Restore backup if exists
    let backup = path.with_extension("msg.bak");
    if backup.exists() {
        fs::rename(&backup, &path)?;
        println!("Restored previous hook from backup");
    }

    Ok(())
}
