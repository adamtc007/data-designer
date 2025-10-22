/// Shared utilities for DSL parsing and processing
/// This module provides common functionality used across all DSL parsers
///
/// Strip comments from DSL text - removes lines starting with # and inline comments
/// This follows the EBNF grammar: comment = "#" , { any_character - newline } , newline ;
pub fn strip_comments(dsl_text: &str) -> String {
    dsl_text
        .lines()
        .map(|line| {
            // Remove inline comments (everything after #)
            if let Some(comment_pos) = line.find('#') {
                line[..comment_pos].trim_end()
            } else {
                line
            }
        })
        .filter(|line| !line.trim().is_empty()) // Remove empty lines
        .collect::<Vec<&str>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_comments_leading_comments() {
        let dsl_with_comments = r#"# This is a comment
# Another comment
CREATE CBU name="Test""#;

        let cleaned = strip_comments(dsl_with_comments);
        assert_eq!(cleaned, r#"CREATE CBU name="Test""#);
    }

    #[test]
    fn test_strip_comments_inline_comments() {
        let dsl_with_comments = r#"CREATE CBU name="Test" # inline comment
WITH ENTITY "test""#;

        let cleaned = strip_comments(dsl_with_comments);
        assert_eq!(cleaned, "CREATE CBU name=\"Test\"\nWITH ENTITY \"test\"");
    }

    #[test]
    fn test_strip_comments_mixed() {
        let dsl_with_comments = r#"# Leading comment
CREATE CBU name="Test" # inline comment
# Middle comment
WITH ENTITY "test" # another inline
# Trailing comment"#;

        let cleaned = strip_comments(dsl_with_comments);
        assert_eq!(cleaned, "CREATE CBU name=\"Test\"\nWITH ENTITY \"test\"");
    }

    #[test]
    fn test_strip_comments_no_comments() {
        let dsl = "CREATE CBU name=\"Test\"\nWITH ENTITY \"test\"";
        let cleaned = strip_comments(dsl);
        assert_eq!(cleaned, dsl);
    }
}