use crate::markers::heading_marker;

pub fn canonicalize_markdown(input: &str) -> String {
    let mut out = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            let title = trimmed[level..].trim();
            if (1..=6).contains(&level) && !title.is_empty() {
                out.push(heading_marker(level, title));
            }
        }
        out.push(line.to_string());
    }
    out.join("\n")
}
