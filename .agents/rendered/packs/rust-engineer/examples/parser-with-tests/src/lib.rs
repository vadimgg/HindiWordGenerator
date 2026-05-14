/// @intent Parse comma-separated labels while preserving empty input as no labels.
pub fn parse_labels(input: &str) -> Vec<&str> {
    input
        .split(',')
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_labels_ignores_empty_segments() {
        assert_eq!(parse_labels("alpha, , beta"), vec!["alpha", "beta"]);
    }
}
