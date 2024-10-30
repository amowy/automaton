pub fn escape_dot_label(s: &str) -> String {
    s.replace('\"', "\\\"")
}