pub(crate) fn to_kebab_case(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}
