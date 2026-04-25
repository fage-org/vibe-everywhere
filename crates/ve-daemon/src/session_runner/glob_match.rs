//! Simple glob pattern matching with `*` and `?` wildcards.

/// Check if a target matches a pattern with wildcard support
///
/// Pattern rules:
/// - `*` matches any sequence of characters
/// - `?` matches any single character
/// - literal characters match themselves
pub fn matches_pattern(pattern: &str, target: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    fn match_helper(pattern: &[char], target: &[char]) -> bool {
        match (pattern.first(), target.first()) {
            (None, None) => true,
            (Some('*'), _) => {
                match_helper(&pattern[1..], target)
                    || (!target.is_empty() && match_helper(pattern, &target[1..]))
            }
            (Some('?'), Some(_)) => match_helper(&pattern[1..], &target[1..]),
            (Some(p), Some(t)) if *p == *t => match_helper(&pattern[1..], &target[1..]),
            _ => false,
        }
    }

    match_helper(&pattern_chars, &target_chars)
}
