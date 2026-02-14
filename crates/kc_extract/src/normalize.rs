pub fn normalize_text_v1(input: &str) -> String {
    let normalized_line_endings = input.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed_per_line = normalized_line_endings
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    let mut out = trimmed_per_line;
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}
