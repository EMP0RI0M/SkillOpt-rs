use crate::skillopt_sleep::types::SessionDigest;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const NEGATIVE_FEEDBACK: &[&str] = &[
    "still broken", "still not", "still wrong", "doesn't work", "does not work",
    "not working", "that's wrong", "thats wrong", "incorrect", "wrong",
    "no,", "nope", "fix it", "didn't", "did not", "broken", "error again",
    "still failing", "still fails", "not fixed", "revert", "undo",
];

const POSITIVE_FEEDBACK: &[&str] = &[
    "thanks", "thank you", "perfect", "great", "works now", "fixed",
    "that works", "lgtm", "looks good", "nice", "awesome", "correct",
];

/// Detect explicit user feedback signals from message text.
pub fn detect_feedback(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut signals = Vec::new();

    for &ph in NEGATIVE_FEEDBACK {
        if lower.contains(ph) {
            signals.push(format!("neg:{}", ph));
        }
    }
    for &ph in POSITIVE_FEEDBACK {
        if lower.contains(ph) {
            signals.push(format!("pos:{}", ph));
        }
    }

    signals
}

/// Extract clean text from Claude message content (string or list of content blocks).
pub fn text_from_content(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            let mut parts = Vec::new();
            for block in arr {
                if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                    if let Some(txt) = block.get("text").and_then(|v| v.as_str()) {
                        parts.push(txt);
                    }
                }
            }
            parts.join("\n")
        }
        _ => String::new(),
    }
}

/// Extract tool names used in an assistant turn.
pub fn tool_names_from_content(content: &Value) -> Vec<String> {
    let mut names = Vec::new();
    if let Value::Array(arr) = content {
        for block in arr {
            if block.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                if let Some(name) = block.get("name").and_then(|v| v.as_str()) {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

/// Parse Claude Code project transcript (.jsonl) into a normalized SessionDigest.
pub fn harvest_session_file<P: AsRef<Path>>(path: P) -> anyhow::Result<SessionDigest> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);

    let mut session_id = String::new();
    let mut project = String::new();
    let mut git_branch = String::new();
    let mut user_prompts = Vec::new();
    let mut assistant_finals = Vec::new();
    let mut tools_used = Vec::new();
    let mut feedback_signals = Vec::new();
    let mut n_user_turns = 0;
    let mut n_assistant_turns = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let record: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if session_id.is_empty() {
            if let Some(sid) = record.get("sessionId").and_then(|v| v.as_str()) {
                session_id = sid.to_string();
            }
        }
        if project.is_empty() {
            if let Some(p) = record.get("cwd").and_then(|v| v.as_str()) {
                project = p.to_string();
            }
        }
        if git_branch.is_empty() {
            if let Some(b) = record.get("gitBranch").and_then(|v| v.as_str()) {
                git_branch = b.to_string();
            }
        }

        let rec_type = record.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(msg) = record.get("message") {
            let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or(rec_type);
            let content = msg.get("content").unwrap_or(&Value::Null);

            if role == "user" {
                n_user_turns += 1;
                let txt = text_from_content(content);
                if !txt.is_empty() {
                    feedback_signals.extend(detect_feedback(&txt));
                    user_prompts.push(txt);
                }
            } else if role == "assistant" {
                n_assistant_turns += 1;
                tools_used.extend(tool_names_from_content(content));
                let txt = text_from_content(content);
                if !txt.is_empty() {
                    assistant_finals.push(txt);
                }
            }
        }
    }

    tools_used.sort();
    tools_used.dedup();
    feedback_signals.sort();
    feedback_signals.dedup();

    Ok(SessionDigest {
        session_id,
        project,
        git_branch,
        user_prompts,
        assistant_finals,
        tools_used,
        feedback_signals,
        n_user_turns,
        n_assistant_turns,
        raw_path: path.as_ref().to_string_lossy().to_string(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_feedback() {
        let msg = "That's wrong, please fix it.";
        let sigs = detect_feedback(msg);
        assert!(sigs.contains(&"neg:fix it".to_string()));
        assert!(sigs.contains(&"neg:wrong".to_string()));

        let good = "Thanks, works now!";
        let sigs_good = detect_feedback(good);
        assert!(sigs_good.contains(&"pos:thanks".to_string()));
        assert!(sigs_good.contains(&"pos:works now".to_string()));
    }

    #[test]
    fn test_content_extraction() {
        let content = serde_json::json!([
            {"type": "text", "text": "Analyzing repo..."},
            {"type": "tool_use", "name": "Bash"},
            {"type": "text", "text": "Done."}
        ]);
        assert_eq!(text_from_content(&content), "Analyzing repo...\nDone.");
        assert_eq!(tool_names_from_content(&content), vec!["Bash"]);
    }
}
