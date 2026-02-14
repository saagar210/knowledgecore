pub fn heading_marker(level: usize, title: &str) -> String {
    format!("[[H{}:{}]]", level, title.trim())
}

pub fn page_marker(page: usize) -> String {
    format!("[[PAGE:{:04}]]", page)
}
