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
