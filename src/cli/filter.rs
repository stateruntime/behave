//! Filter expression parser and evaluator for `--filter` flag.
//!
//! Supports boolean expressions over test tags and names:
//!
//! ```text
//! cargo behave --filter 'tag(slow) and not tag(flaky)'
//! cargo behave --filter 'name(checkout) or tag(integration)'
//! ```

use super::error::CliError;

/// A parsed filter expression AST.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::filter::FilterExpr;
///
/// let expr = FilterExpr::Tag("slow".to_string());
/// assert!(expr.matches("__TAG_slow__my_test"));
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FilterExpr {
    /// Matches tests whose full name contains `__TAG_{tag}__`.
    Tag(String),
    /// Matches tests whose full name contains the substring.
    Name(String),
    /// Both sides must match.
    And(Box<FilterExpr>, Box<FilterExpr>),
    /// At least one side must match.
    Or(Box<FilterExpr>, Box<FilterExpr>),
    /// Inverts the inner expression.
    Not(Box<FilterExpr>),
}

impl FilterExpr {
    /// Evaluates this expression against a full test name.
    ///
    /// Tags are encoded as `__TAG_xxx__` prefixes in test names.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "cli")]
    /// # {
    /// use behave::cli::filter::FilterExpr;
    ///
    /// let expr = FilterExpr::Tag("slow".to_string());
    /// assert!(expr.matches("__TAG_slow__suite::test_a"));
    /// assert!(!expr.matches("suite::test_b"));
    /// # }
    /// ```
    pub fn matches(&self, test_name: &str) -> bool {
        match self {
            Self::Tag(tag) => {
                let marker = format!("__TAG_{tag}__");
                test_name.contains(&marker)
            }
            Self::Name(substr) => test_name.contains(substr.as_str()),
            Self::And(a, b) => a.matches(test_name) && b.matches(test_name),
            Self::Or(a, b) => a.matches(test_name) || b.matches(test_name),
            Self::Not(inner) => !inner.matches(test_name),
        }
    }
}

/// Parses a filter expression string into an AST.
///
/// # Errors
///
/// Returns [`CliError::FilterParse`] if the expression is malformed.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::filter::parse_filter;
///
/// let expr = parse_filter("tag(slow) and not tag(flaky)").unwrap();
/// assert!(expr.matches("__TAG_slow__my_test"));
/// # }
/// ```
pub fn parse_filter(input: &str) -> Result<FilterExpr, CliError> {
    let tokens = tokenize(input);
    let (expr, rest) = parse_or(&tokens)?;
    if !rest.is_empty() {
        return Err(filter_err(format!("unexpected token: {}", rest[0])));
    }
    Ok(expr)
}

// --- Tokenizer ---

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else if ch == '(' {
            tokens.push("(".to_string());
            chars.next();
        } else if ch == ')' {
            tokens.push(")".to_string());
            chars.next();
        } else {
            tokens.push(collect_word(&mut chars));
        }
    }

    tokens
}

/// Collects an alphanumeric word. If the word contains `(`, reads through
/// the matching `)` so that `tag(slow)` stays as one token.
fn collect_word(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut word = String::new();
    let mut depth = 0u32;
    while let Some(&ch) = chars.peek() {
        if ch == '(' {
            depth += 1;
            word.push(ch);
            chars.next();
        } else if ch == ')' && depth > 0 {
            depth -= 1;
            word.push(ch);
            chars.next();
        } else if depth == 0 && (ch.is_whitespace() || ch == ')' || ch == '(') {
            break;
        } else {
            word.push(ch);
            chars.next();
        }
    }
    word
}

// --- Recursive-descent parser ---

fn parse_or(tokens: &[String]) -> Result<(FilterExpr, &[String]), CliError> {
    let (mut left, mut rest) = parse_and(tokens)?;
    while rest.first().is_some_and(|t| t == "or") {
        let (right, r) = parse_and(&rest[1..])?;
        left = FilterExpr::Or(Box::new(left), Box::new(right));
        rest = r;
    }
    Ok((left, rest))
}

fn parse_and(tokens: &[String]) -> Result<(FilterExpr, &[String]), CliError> {
    let (mut left, mut rest) = parse_not(tokens)?;
    while rest.first().is_some_and(|t| t == "and") {
        let (right, r) = parse_not(&rest[1..])?;
        left = FilterExpr::And(Box::new(left), Box::new(right));
        rest = r;
    }
    Ok((left, rest))
}

fn parse_not(tokens: &[String]) -> Result<(FilterExpr, &[String]), CliError> {
    if tokens.first().is_some_and(|t| t == "not") {
        let (inner, rest) = parse_not(&tokens[1..])?;
        return Ok((FilterExpr::Not(Box::new(inner)), rest));
    }
    parse_primary(tokens)
}

fn parse_primary(tokens: &[String]) -> Result<(FilterExpr, &[String]), CliError> {
    let first = tokens
        .first()
        .ok_or_else(|| filter_err("unexpected end of expression"))?;

    if first == "(" {
        return parse_grouped(&tokens[1..]);
    }
    if let Some(inner) = extract_function_arg(first, "tag(") {
        return Ok((FilterExpr::Tag(inner), &tokens[1..]));
    }
    if let Some(inner) = extract_function_arg(first, "name(") {
        return Ok((FilterExpr::Name(inner), &tokens[1..]));
    }
    Err(filter_err(format!(
        "expected tag(...) or name(...), got: {first}"
    )))
}

fn parse_grouped(tokens: &[String]) -> Result<(FilterExpr, &[String]), CliError> {
    let (expr, rest) = parse_or(tokens)?;
    if rest.first().is_some_and(|t| t == ")") {
        return Ok((expr, &rest[1..]));
    }
    Err(filter_err("expected closing ')'"))
}

fn extract_function_arg(token: &str, prefix: &str) -> Option<String> {
    token
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(')'))
        .map(|s| {
            s.strip_prefix('"')
                .and_then(|inner| inner.strip_suffix('"'))
                .or_else(|| {
                    s.strip_prefix('\'')
                        .and_then(|inner| inner.strip_suffix('\''))
                })
                .unwrap_or(s)
                .to_string()
        })
}

fn filter_err(message: impl Into<String>) -> CliError {
    let msg = message.into();
    CliError::FilterParse {
        message: format!("{msg}\n  example: tag(slow) and not tag(flaky)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_tag() {
        let expr = parse_filter("tag(slow)").ok();
        assert!(expr.is_some());
        let expr = expr.unwrap_or_else(|| FilterExpr::Tag(String::new()));
        assert!(expr.matches("__TAG_slow__test"));
        assert!(!expr.matches("test"));
    }

    #[test]
    fn parse_single_name() {
        let expr = parse_filter("name(checkout)").ok();
        assert!(expr.is_some());
        let expr = expr.unwrap_or_else(|| FilterExpr::Tag(String::new()));
        assert!(expr.matches("suite::checkout::test"));
        assert!(!expr.matches("suite::other::test"));
    }

    #[test]
    fn parse_and_expression() {
        let expr = parse_filter("tag(slow) and tag(integration)").ok();
        assert!(expr.is_some());
        let expr = expr.unwrap_or_else(|| FilterExpr::Tag(String::new()));
        assert!(expr.matches("__TAG_slow____TAG_integration__test"));
        assert!(!expr.matches("__TAG_slow__test"));
    }

    #[test]
    fn parse_or_expression() {
        let expr = parse_filter("tag(slow) or tag(fast)").ok();
        assert!(expr.is_some());
        let expr = expr.unwrap_or_else(|| FilterExpr::Tag(String::new()));
        assert!(expr.matches("__TAG_slow__test"));
        assert!(expr.matches("__TAG_fast__test"));
        assert!(!expr.matches("test"));
    }

    #[test]
    fn parse_not_expression() {
        let expr = parse_filter("not tag(flaky)").ok();
        assert!(expr.is_some());
        let expr = expr.unwrap_or_else(|| FilterExpr::Tag(String::new()));
        assert!(expr.matches("test"));
        assert!(!expr.matches("__TAG_flaky__test"));
    }

    #[test]
    fn parse_complex_expression() {
        let expr = parse_filter("tag(slow) and not tag(flaky)").ok();
        assert!(expr.is_some());
        let expr = expr.unwrap_or_else(|| FilterExpr::Tag(String::new()));
        assert!(expr.matches("__TAG_slow__test"));
        assert!(!expr.matches("__TAG_slow____TAG_flaky__test"));
    }

    #[test]
    fn parse_grouped_expression() {
        let expr = parse_filter("(tag(a) or tag(b)) and tag(c)").ok();
        assert!(expr.is_some());
        let expr = expr.unwrap_or_else(|| FilterExpr::Tag(String::new()));
        assert!(expr.matches("__TAG_a____TAG_c__test"));
        assert!(expr.matches("__TAG_b____TAG_c__test"));
        assert!(!expr.matches("__TAG_a__test"));
    }

    #[test]
    fn parse_empty_input_fails() {
        assert!(parse_filter("").is_err());
    }

    #[test]
    fn parse_invalid_token_fails() {
        assert!(parse_filter("unknown").is_err());
    }

    #[test]
    fn parse_unclosed_paren_fails() {
        assert!(parse_filter("(tag(a)").is_err());
    }

    #[test]
    fn parse_quoted_tag_double() {
        let expr = parse_filter("tag(\"slow\")").ok();
        assert!(expr.is_some());
        let expr = expr.unwrap_or_else(|| FilterExpr::Tag(String::new()));
        assert!(expr.matches("__TAG_slow__test"));
    }

    #[test]
    fn parse_quoted_tag_single() {
        let expr = parse_filter("tag('slow')").ok();
        assert!(expr.is_some());
        let expr = expr.unwrap_or_else(|| FilterExpr::Tag(String::new()));
        assert!(expr.matches("__TAG_slow__test"));
    }

    #[test]
    fn parse_error_includes_syntax_hint() {
        let result = parse_filter("unknown");
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("example:"));
        }
    }
}
