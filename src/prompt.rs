use crate::config::Config;

pub fn build_system_prompt(config: &Config) -> String {
    let mut parts = Vec::new();

    parts.push(
        "You are a commit message generator. Given a git diff, generate a concise, \
         accurate commit message. Output ONLY the commit message, nothing else. \
         No markdown formatting, no code blocks, no quotes."
            .to_string(),
    );

    if config.commit.conventional {
        parts.push(
            "Use Conventional Commits format: <type>(<optional scope>): <description>\n\n\
             Valid types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert.\n\
             The first line must be under 72 characters.\n\
             If the change is significant, add a blank line followed by a body explaining what and why."
                .to_string(),
        );
    }

    if config.commit.emoji {
        parts.push(
            "Prefix the commit type with an appropriate Gitmoji:\n\
             - feat: ✨  - fix: 🐛  - docs: 📝  - style: 💄  - refactor: ♻️\n\
             - perf: ⚡  - test: ✅  - build: 📦  - ci: 👷  - chore: 🔧  - revert: ⏪"
                .to_string(),
        );
    }

    match config.commit.language.as_str() {
        "ja" => parts.push("Write the commit message in Japanese.".to_string()),
        "en" => {} // default
        lang => parts.push(format!("Write the commit message in {lang}.")),
    }

    if !config.prompt.system.is_empty() {
        parts.push(config.prompt.system.clone());
    }

    parts.join("\n\n")
}

pub fn build_user_prompt(
    diff: &str,
    status: &str,
    hint: Option<&str>,
    max_diff_length: usize,
) -> String {
    let mut parts = Vec::new();

    if let Some(h) = hint {
        parts.push(format!("Context: {h}"));
    }

    let truncated_diff = truncate_diff(diff, max_diff_length);
    parts.push(format!("Git diff:\n```diff\n{truncated_diff}\n```"));

    if !status.is_empty() {
        parts.push(format!("Git status:\n```\n{status}\n```"));
    }

    parts.join("\n\n")
}

pub fn build_edit_prompt(
    diff: &str,
    status: &str,
    previous_message: &str,
    edit_instruction: &str,
    hint: Option<&str>,
    max_diff_length: usize,
) -> String {
    let mut parts = Vec::new();

    if let Some(h) = hint {
        parts.push(format!("Context: {h}"));
    }

    let truncated_diff = truncate_diff(diff, max_diff_length);
    parts.push(format!("Git diff:\n```diff\n{truncated_diff}\n```"));

    if !status.is_empty() {
        parts.push(format!("Git status:\n```\n{status}\n```"));
    }

    parts.push(format!("Previous commit message:\n{previous_message}"));
    parts.push(format!(
        "Revision instruction: {edit_instruction}\n\n\
         Generate a revised commit message based on the instruction above. \
         Output ONLY the commit message."
    ));

    parts.join("\n\n")
}

fn truncate_diff(diff: &str, max_length: usize) -> String {
    if diff.len() <= max_length {
        return diff.to_string();
    }

    // Truncate by file sections, keeping most important parts
    let mut result = String::new();
    let mut current_file = String::new();

    for line in diff.lines() {
        if line.starts_with("diff --git") {
            // If adding this file section would exceed limit, stop
            if !current_file.is_empty() {
                if result.len() + current_file.len() > max_length {
                    result.push_str("\n... (diff truncated, remaining files omitted)\n");
                    return result;
                }
                result.push_str(&current_file);
                current_file.clear();
            }
        }
        current_file.push_str(line);
        current_file.push('\n');
    }

    // Add last file section if it fits
    if result.len() + current_file.len() <= max_length {
        result.push_str(&current_file);
    } else {
        result.push_str("\n... (diff truncated, remaining files omitted)\n");
    }

    result
}
