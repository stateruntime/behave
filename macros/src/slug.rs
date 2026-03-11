//! Label slugification for converting human-readable strings to Rust identifiers.

/// Converts a human-readable label into a valid Rust identifier slug.
///
/// Lowercases, replaces non-alphanumeric characters with underscores,
/// collapses runs of underscores, and trims leading/trailing underscores.
/// Prepends `_` if the result starts with a digit.
///
/// Keyword escaping (`r#`) is handled by the codegen layer, not here.
pub fn slugify(input: &str) -> String {
    let slug: String = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();

    let collapsed = collapse_underscores(&slug);
    let trimmed = collapsed.trim_matches('_');

    if trimmed.is_empty() {
        return String::from("_unnamed");
    }

    if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// Returns `true` if the identifier is a Rust keyword requiring `r#`.
pub fn is_rust_keyword(ident: &str) -> bool {
    const RUST_KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while", "yield",
    ];
    RUST_KEYWORDS.contains(&ident)
}

fn collapse_underscores(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_underscore = false;
    for c in input.chars() {
        if c == '_' {
            if !prev_underscore {
                result.push('_');
            }
            prev_underscore = true;
        } else {
            result.push(c);
            prev_underscore = false;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic_label() {
        assert_eq!(slugify("adds two numbers"), "adds_two_numbers");
    }

    #[test]
    fn slugify_special_characters() {
        assert_eq!(slugify("it's a test!"), "it_s_a_test");
    }

    #[test]
    fn slugify_leading_digit() {
        assert_eq!(slugify("1st test"), "_1st_test");
    }

    #[test]
    fn slugify_keyword_returns_plain() {
        // Keywords are no longer escaped here; codegen handles it.
        assert_eq!(slugify("type"), "type");
    }

    #[test]
    fn slugify_collapses_underscores() {
        assert_eq!(slugify("a   b"), "a_b");
    }

    #[test]
    fn slugify_empty_after_strip() {
        assert_eq!(slugify("!!!"), "_unnamed");
    }

    #[test]
    fn slugify_mixed_case() {
        assert_eq!(slugify("Hello World"), "hello_world");
    }

    #[test]
    fn is_keyword_true() {
        assert!(is_rust_keyword("type"));
        assert!(is_rust_keyword("fn"));
    }

    #[test]
    fn is_keyword_false() {
        assert!(!is_rust_keyword("hello"));
        assert!(!is_rust_keyword("_type"));
    }
}
