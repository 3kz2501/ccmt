/// Clean up the AI-generated commit message
pub fn format_message(raw: &str) -> String {
    let mut msg = raw.trim().to_string();

    // Remove surrounding quotes
    if (msg.starts_with('"') && msg.ends_with('"'))
        || (msg.starts_with('\'') && msg.ends_with('\''))
    {
        msg = msg[1..msg.len() - 1].to_string();
    }

    // Remove markdown code block wrapping
    if msg.starts_with("```") {
        let lines: Vec<&str> = msg.lines().collect();
        if lines.len() >= 2 && lines.last().is_some_and(|l| l.trim() == "```") {
            msg = lines[1..lines.len() - 1].join("\n");
        }
    }

    // Remove "commit message:" prefix
    let lower = msg.to_lowercase();
    for prefix in &["commit message:", "commit:"] {
        if lower.starts_with(prefix) {
            msg = msg[prefix.len()..].trim().to_string();
            break;
        }
    }

    // Ensure title line is not too long (72 chars max for first line)
    let lines: Vec<&str> = msg.lines().collect();
    if let Some(first) = lines.first()
        && first.len() > 72
    {
        // Try to break at a word boundary
        let truncated: String = first.chars().take(69).collect();
        if let Some(last_space) = truncated.rfind(' ') {
            let title = &first[..last_space];
            let rest = first[last_space..].trim();
            let mut new_msg = title.to_string();
            if !rest.is_empty() {
                new_msg.push_str("\n\n");
                new_msg.push_str(rest);
            }
            if lines.len() > 1 {
                for line in &lines[1..] {
                    new_msg.push('\n');
                    new_msg.push_str(line);
                }
            }
            msg = new_msg;
        }
    }

    msg.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_message() {
        assert_eq!(format_message("feat: add login"), "feat: add login");
    }

    #[test]
    fn strips_surrounding_quotes() {
        assert_eq!(format_message("\"feat: add login\""), "feat: add login");
        assert_eq!(format_message("'fix: typo'"), "fix: typo");
    }

    #[test]
    fn strips_code_block() {
        let input = "```\nfeat: add login\n```";
        assert_eq!(format_message(input), "feat: add login");
    }

    #[test]
    fn strips_code_block_with_lang() {
        let input = "```text\nfeat: add login\n```";
        assert_eq!(format_message(input), "feat: add login");
    }

    #[test]
    fn strips_commit_message_prefix() {
        assert_eq!(
            format_message("Commit message: feat: add login"),
            "feat: add login"
        );
        assert_eq!(format_message("commit: fix: typo"), "fix: typo");
    }

    #[test]
    fn preserves_multiline() {
        let input = "feat: add login\n\nImplement JWT auth";
        assert_eq!(format_message(input), input);
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(format_message("  feat: add login  \n"), "feat: add login");
    }

    #[test]
    fn wraps_long_title() {
        let input = "feat: this is a very long commit message title that definitely exceeds the seventy two character limit for git";
        let result = format_message(input);
        let first_line = result.lines().next().unwrap();
        assert!(first_line.len() <= 72);
    }
}
