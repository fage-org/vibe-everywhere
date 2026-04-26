//! Simple glob pattern matching with `*` and `?` wildcards.

/// Check if a target matches a pattern with wildcard support.
///
/// Pattern rules:
/// - `*` matches any sequence of characters (including empty)
/// - `?` matches any single character
/// - literal characters match themselves
///
/// Uses iterative dynamic programming (O(pattern_len * target_len)) to avoid
/// the O(2^n) worst case of naive recursive implementations.
pub fn matches_pattern(pattern: &str, target: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = target.chars().collect();
    let m = p.len();
    let n = t.len();

    // dp[i][j] = whether p[0..i] matches t[0..j]
    let mut dp = vec![vec![false; n + 1]; m + 1];
    dp[0][0] = true;

    // Handle leading *'s: they can match empty string
    for i in 1..=m {
        if p[i - 1] == '*' {
            dp[i][0] = dp[i - 1][0];
        } else {
            break;
        }
    }

    for i in 1..=m {
        for j in 1..=n {
            if p[i - 1] == '*' {
                // * matches empty (dp[i-1][j]) or consumes one char (dp[i][j-1])
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if p[i - 1] == '?' || p[i - 1] == t[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }

    dp[m][n]
}
