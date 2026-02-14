use crate::app_error::AppResult;
use regex::Regex;

pub fn render_snippet_display_only(text: &str) -> AppResult<String> {
    let marker_re = Regex::new(r"^\[\[(PAGE:[0-9]{4}|H[1-6]:.*)\]\]$").expect("valid regex");

    let mut lines = Vec::new();
    let mut prev_blank = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if marker_re.is_match(trimmed) {
            continue;
        }
        if trimmed.is_empty() {
            if prev_blank {
                continue;
            }
            prev_blank = true;
            lines.push(String::new());
        } else {
            prev_blank = false;
            lines.push(trimmed.to_string());
        }
    }

    Ok(lines.join("\n").trim().to_string())
}
