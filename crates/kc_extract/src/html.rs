use crate::markers::heading_marker;
use regex::Regex;

pub fn canonicalize_html(input: &str) -> String {
    let heading_re = Regex::new(r"(?is)<h([1-6])[^>]*>(.*?)</h[1-6]>").expect("valid regex");
    let tag_re = Regex::new(r"(?is)<[^>]+>").expect("valid regex");

    let mut with_markers = input.to_string();
    for cap in heading_re.captures_iter(input) {
        let level = cap
            .get(1)
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(1);
        let raw = cap.get(2).map(|m| m.as_str()).unwrap_or_default();
        let title = tag_re.replace_all(raw, "").to_string();
        let marker = format!("{}\n", heading_marker(level, &title));
        with_markers = with_markers.replacen(cap.get(0).map(|m| m.as_str()).unwrap_or_default(), &(marker + raw), 1);
    }

    tag_re.replace_all(&with_markers, "").to_string()
}
