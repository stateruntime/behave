//! DSL parser for the `behave!` macro.
//!
//! Parses the token stream into a tree of [`BehaveNode`] values representing
//! groups (describe blocks) and tests (leaf blocks).

use syn::parse::{Parse, ParseStream};
use syn::{braced, bracketed, token, Expr, Ident, LitInt, LitStr, Result, Token};

/// Root AST node containing all top-level groups.
#[derive(Debug)]
pub struct BehaveInput {
    pub nodes: Vec<BehaveNode>,
}

/// A single node in the behave DSL tree.
#[derive(Debug)]
pub enum BehaveNode {
    /// A group containing other nodes (describe block).
    Group(GroupNode),
    /// A leaf test containing executable code.
    Test(TestNode),
    /// A pending test that should be ignored.
    Pending(PendingNode),
    /// A parameterized test block generating one test per case.
    Each(EachNode),
}

/// A group node that contains child nodes.
#[derive(Debug)]
pub struct GroupNode {
    pub label: String,
    pub children: Vec<BehaveNode>,
    pub setup: Option<proc_macro2::TokenStream>,
    pub teardown: Option<proc_macro2::TokenStream>,
    pub async_runtime: bool,
    pub timeout_ms: Option<u64>,
    pub focused: bool,
}

/// A leaf test node containing code to execute.
#[derive(Debug)]
pub struct TestNode {
    pub label: String,
    pub body: proc_macro2::TokenStream,
    pub focused: bool,
}

/// A pending (ignored) test node.
#[derive(Debug)]
pub struct PendingNode {
    pub label: String,
}

/// A parameterized test node that generates one test per case.
#[derive(Debug)]
pub struct EachNode {
    /// Human-readable label for the generated module.
    pub label: String,
    /// The list of case expressions (one per generated test).
    pub cases: Vec<Expr>,
    /// Parameter names bound from each case expression.
    pub params: Vec<Ident>,
    /// The test body executed for each case.
    pub body: proc_macro2::TokenStream,
    /// Whether this block was marked with `focus`.
    pub focused: bool,
}

impl Parse for BehaveInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut nodes = Vec::new();
        while !input.is_empty() {
            nodes.push(parse_node(input)?);
        }
        if nodes.is_empty() {
            return Err(input.error("behave! block must contain at least one group or test"));
        }
        Ok(Self { nodes })
    }
}

fn parse_node(input: ParseStream<'_>) -> Result<BehaveNode> {
    let focused = try_parse_keyword(input, "focus");
    let pending = try_parse_keyword(input, "pending");

    if focused && pending {
        return Err(input.error("a test cannot be both `focus` and `pending`"));
    }

    let label: LitStr = input.parse()?;
    let label_str = label.value();
    let span = label.span();

    let content;
    braced!(content in input);

    if pending {
        if !content.is_empty() {
            return Err(syn::Error::new(
                span,
                "pending blocks must be empty; remove the body or convert it to a normal test",
            ));
        }
        return Ok(BehaveNode::Pending(PendingNode { label: label_str }));
    }

    classify_block(&content, label_str, span, focused)
}

fn classify_block(
    content: ParseStream<'_>,
    label: String,
    span: proc_macro2::Span,
    focused: bool,
) -> Result<BehaveNode> {
    if content.is_empty() {
        return Err(syn::Error::new(span, "empty blocks are not allowed"));
    }

    if looks_like_each(content) {
        return parse_each_body(content, label, focused);
    }

    if looks_like_group(content) {
        parse_group_body(content, label, span, focused)
    } else {
        parse_test_body(content, label, focused)
    }
}

/// Peeks ahead to determine if this block starts with `each [`.
fn looks_like_each(input: ParseStream<'_>) -> bool {
    let fork = input.fork();
    let Ok(id) = fork.parse::<Ident>() else {
        return false;
    };
    id == "each" && fork.peek(token::Bracket)
}

/// Parses `each [cases] |params| { body }` into an [`EachNode`].
fn parse_each_body(input: ParseStream<'_>, label: String, focused: bool) -> Result<BehaveNode> {
    // Consume `each`
    let _each: Ident = input.parse()?;

    // Parse `[case1, case2, ...]`
    let cases_content;
    bracketed!(cases_content in input);
    let punctuated =
        syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated(&cases_content)?;
    let cases: Vec<Expr> = punctuated.into_iter().collect();

    if cases.is_empty() {
        return Err(input.error("each block must contain at least one case"));
    }

    // Parse `|param1, param2, ...|`
    let _pipe1: Token![|] = input.parse()?;
    let params_punctuated =
        syn::punctuated::Punctuated::<Ident, Token![,]>::parse_separated_nonempty(input)?;
    let params: Vec<Ident> = params_punctuated.into_iter().collect();
    let _pipe2: Token![|] = input.parse()?;

    // Parse `{ body }`
    let body_content;
    braced!(body_content in input);
    let body: proc_macro2::TokenStream = body_content.parse()?;

    Ok(BehaveNode::Each(EachNode {
        label,
        cases,
        params,
        body,
        focused,
    }))
}

/// Peeks ahead to determine if this block contains child groups
/// (string literal followed by braces) or test code.
///
/// Skips any combination of `tokio;`, `timeout <int>;`, `setup {}`, and
/// `teardown {}` prefixes. Validation of ordering and duplicates happens
/// later in [`parse_group_body`].
fn looks_like_group(input: ParseStream<'_>) -> bool {
    let fork = input.fork();

    loop {
        if !fork.peek(syn::Ident) {
            break;
        }
        let inner = fork.fork();
        let Ok(id) = inner.parse::<syn::Ident>() else {
            return false;
        };

        if id == "tokio" && inner.peek(syn::Token![;]) {
            let _ = fork.parse::<syn::Ident>();
            let _ = fork.parse::<syn::Token![;]>();
            continue;
        }
        if id == "timeout" && inner.peek(LitInt) {
            let _ = fork.parse::<syn::Ident>();
            let _ = fork.parse::<LitInt>();
            let _ = fork.parse::<syn::Token![;]>();
            continue;
        }
        if (id == "setup" || id == "teardown") && inner.peek(token::Brace) {
            let _ = fork.parse::<syn::Ident>();
            if fork.parse::<proc_macro2::Group>().is_err() {
                return false;
            }
            continue;
        }
        if id == "focus" || id == "pending" {
            let _ = fork.parse::<syn::Ident>();
            return fork.peek(LitStr);
        }
        // Unknown ident — treat as test code
        return false;
    }

    fork.peek(LitStr) && {
        let _: std::result::Result<LitStr, _> = fork.parse();
        fork.peek(token::Brace)
    }
}

fn peek_keyword(input: ParseStream<'_>, keyword: &str) -> bool {
    if !input.peek(syn::Ident) {
        return false;
    }
    let fork = input.fork();
    fork.parse::<syn::Ident>()
        .map_or(false, |id| id == keyword && fork.peek(token::Brace))
}

fn parse_group_body(
    content: ParseStream<'_>,
    label: String,
    span: proc_macro2::Span,
    focused: bool,
) -> Result<BehaveNode> {
    let async_runtime = try_parse_async_runtime(content)?;
    let timeout_ms = try_parse_timeout(content)?;
    let setup = try_parse_setup(content)?;
    let teardown = try_parse_teardown(content)?;

    // Reject duplicate or misordered blocks
    if peek_keyword(content, "setup") {
        let msg = if teardown.is_some() {
            "setup must appear before teardown"
        } else {
            "only one setup block is allowed per group"
        };
        return Err(content.error(msg));
    }
    if peek_keyword(content, "teardown") {
        return Err(content.error("only one teardown block is allowed per group"));
    }

    let mut children = Vec::new();

    while !content.is_empty() {
        children.push(parse_node(content)?);
    }

    if children.is_empty() {
        return Err(syn::Error::new(
            span,
            "group must contain at least one test or subgroup",
        ));
    }

    Ok(BehaveNode::Group(GroupNode {
        label,
        children,
        setup,
        teardown,
        async_runtime,
        timeout_ms,
        focused,
    }))
}

fn parse_test_body(content: ParseStream<'_>, label: String, focused: bool) -> Result<BehaveNode> {
    let body: proc_macro2::TokenStream = content.parse()?;
    Ok(BehaveNode::Test(TestNode {
        label,
        body,
        focused,
    }))
}

fn try_parse_setup(input: ParseStream<'_>) -> Result<Option<proc_macro2::TokenStream>> {
    if !input.peek(syn::Ident) {
        return Ok(None);
    }

    let fork = input.fork();
    let ident: syn::Ident = fork.parse()?;
    if ident != "setup" || !fork.peek(token::Brace) {
        return Ok(None);
    }

    // Consume from real stream
    let _ident: syn::Ident = input.parse()?;
    let inner;
    braced!(inner in input);
    let tokens: proc_macro2::TokenStream = inner.parse()?;
    Ok(Some(tokens))
}

fn try_parse_teardown(input: ParseStream<'_>) -> Result<Option<proc_macro2::TokenStream>> {
    if !input.peek(syn::Ident) {
        return Ok(None);
    }

    let fork = input.fork();
    let ident: syn::Ident = fork.parse()?;
    if ident != "teardown" || !fork.peek(token::Brace) {
        return Ok(None);
    }

    // Consume from real stream
    let _ident: syn::Ident = input.parse()?;
    let inner;
    braced!(inner in input);
    let tokens: proc_macro2::TokenStream = inner.parse()?;
    Ok(Some(tokens))
}

fn try_parse_async_runtime(input: ParseStream<'_>) -> Result<bool> {
    if !input.peek(syn::Ident) {
        return Ok(false);
    }

    let fork = input.fork();
    let ident: syn::Ident = fork.parse()?;
    if ident != "tokio" || !fork.peek(syn::Token![;]) {
        return Ok(false);
    }

    // Consume from real stream
    let _ident: syn::Ident = input.parse()?;
    let _semi: syn::Token![;] = input.parse()?;
    Ok(true)
}

/// Tries to consume `timeout <integer>;` from the input stream.
///
/// Returns `Some(ms)` if the keyword was present, `None` otherwise.
/// Rejects `timeout 0;` at parse time with a compile error.
fn try_parse_timeout(input: ParseStream<'_>) -> Result<Option<u64>> {
    if !input.peek(syn::Ident) {
        return Ok(None);
    }

    let fork = input.fork();
    let ident: syn::Ident = fork.parse()?;
    if ident != "timeout" || !fork.peek(LitInt) {
        return Ok(None);
    }

    // Consume from real stream
    let _ident: syn::Ident = input.parse()?;
    let lit: LitInt = input.parse()?;
    let ms: u64 = lit.base10_parse()?;
    if ms == 0 {
        return Err(syn::Error::new(
            lit.span(),
            "timeout must be greater than 0",
        ));
    }
    let _semi: syn::Token![;] = input.parse()?;
    Ok(Some(ms))
}

fn try_parse_keyword(input: ParseStream<'_>, keyword: &str) -> bool {
    if input.peek(syn::Ident) {
        let fork = input.fork();
        if let Ok(ident) = fork.parse::<syn::Ident>() {
            if ident == keyword && fork.peek(LitStr) {
                // Advance the real stream
                let _ = input.parse::<syn::Ident>();
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_input(tokens: proc_macro2::TokenStream) -> Result<BehaveInput> {
        syn::parse2(tokens)
    }

    /// Unwrap a successful parse and return the first node as a group.
    /// Panics with a clear message if parsing failed or the node is wrong.
    fn first_group(input: Result<BehaveInput>) -> GroupNode {
        let parsed = input.expect("parse should succeed");
        match parsed.nodes.into_iter().next() {
            Some(BehaveNode::Group(g)) => g,
            other => panic!("expected Group node, got {other:?}"),
        }
    }

    /// Unwrap a successful parse and return the first node as a test.
    fn first_test(input: Result<BehaveInput>) -> TestNode {
        let parsed = input.expect("parse should succeed");
        match parsed.nodes.into_iter().next() {
            Some(BehaveNode::Test(t)) => t,
            other => panic!("expected Test node, got {other:?}"),
        }
    }

    #[test]
    fn parse_simple_test() {
        let input = quote::quote! {
            "my test" {
                let x = 1;
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_nested_group() {
        let input = quote::quote! {
            "outer" {
                "inner" {
                    let x = 1;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_pending_test() {
        let input = quote::quote! {
            pending "not yet" {}
        };
        let parsed = parse_input(input);
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_pending_with_body_errors() {
        let input = quote::quote! {
            pending "not yet" {
                let x = 1;
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_empty_block_errors() {
        let input = quote::quote! {
            "empty" {}
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_focus_keyword() {
        let input = quote::quote! {
            focus "focused test" {
                let x = 1;
            }
        };
        let test = first_test(parse_input(input));
        assert!(test.focused);
    }

    #[test]
    fn parse_setup_block() {
        let input = quote::quote! {
            "suite" {
                setup {
                    let x = 1;
                }

                "test" {
                    let _ = x;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.setup.is_some());
    }

    #[test]
    fn parse_nested_three_levels() {
        let input = quote::quote! {
            "level1" {
                "level2" {
                    "level3" {
                        let x = 1;
                    }
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_unicode_labels() {
        let input = quote::quote! {
            "日本語テスト" {
                let x = 1;
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_teardown_block() {
        let input = quote::quote! {
            "suite" {
                teardown {
                    drop(x);
                }

                "test" {
                    let x = 1;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.teardown.is_some());
    }

    #[test]
    fn parse_setup_and_teardown() {
        let input = quote::quote! {
            "suite" {
                setup {
                    let x = 1;
                }

                teardown {
                    drop(x);
                }

                "test" {
                    let _ = x;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.setup.is_some());
        assert!(group.teardown.is_some());
    }

    #[test]
    fn parse_teardown_without_setup() {
        let input = quote::quote! {
            "suite" {
                teardown {
                    cleanup();
                }

                "test" {
                    let x = 1;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.setup.is_none());
        assert!(group.teardown.is_some());
    }

    #[test]
    fn parse_async_runtime() {
        let input = quote::quote! {
            "suite" {
                tokio;

                "test" {
                    let x = 1;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.async_runtime);
    }

    #[test]
    fn parse_async_with_setup() {
        let input = quote::quote! {
            "suite" {
                tokio;

                setup {
                    let x = 1;
                }

                "test" {
                    let _ = x;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.async_runtime);
        assert!(group.setup.is_some());
    }

    #[test]
    fn parse_async_with_setup_and_teardown() {
        let input = quote::quote! {
            "suite" {
                tokio;

                setup {
                    let x = 1;
                }

                teardown {
                    drop(x);
                }

                "test" {
                    let _ = x;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.async_runtime);
        assert!(group.setup.is_some());
        assert!(group.teardown.is_some());
    }

    #[test]
    fn parse_duplicate_setup_errors() {
        let input = quote::quote! {
            "suite" {
                setup { let x = 1; }
                setup { let y = 2; }
                "test" { let _ = x; }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("only one setup"),
            "expected 'only one setup' error, got: {msg}"
        );
    }

    #[test]
    fn parse_duplicate_teardown_errors() {
        let input = quote::quote! {
            "suite" {
                teardown { cleanup1(); }
                teardown { cleanup2(); }
                "test" { let x = 1; }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("only one teardown"),
            "expected 'only one teardown' error, got: {msg}"
        );
    }

    #[test]
    fn parse_teardown_before_setup_errors() {
        let input = quote::quote! {
            "suite" {
                teardown { cleanup(); }
                setup { let x = 1; }
                "test" { let _ = x; }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("setup must appear before teardown"),
            "expected ordering error, got: {msg}"
        );
    }

    #[test]
    fn parse_focus_and_pending_exclusive() {
        let input = quote::quote! {
            focus pending "both" {
                let x = 1;
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_multiple_siblings() {
        let input = quote::quote! {
            "suite" {
                "test a" {
                    let a = 1;
                }

                "test b" {
                    let b = 2;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert_eq!(group.children.len(), 2);
    }

    #[test]
    fn parse_focus_group() {
        let input = quote::quote! {
            focus "suite" {
                "test" {
                    let x = 1;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.focused);
    }

    #[test]
    fn parse_nested_async_groups() {
        let input = quote::quote! {
            "outer" {
                tokio;

                "inner" {
                    tokio;

                    "test" {
                        let x = 1;
                    }
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.async_runtime);
        if let BehaveNode::Group(inner) = &group.children[0] {
            assert!(inner.async_runtime);
        } else {
            panic!("expected inner Group node");
        }
    }

    /// Unwrap a successful parse and return the first node as an Each.
    fn first_each(input: Result<BehaveInput>) -> EachNode {
        let parsed = input.expect("parse should succeed");
        match parsed.nodes.into_iter().next() {
            Some(BehaveNode::Each(e)) => e,
            other => panic!("expected Each node, got {other:?}"),
        }
    }

    #[test]
    fn parse_each_basic() {
        let input = quote::quote! {
            "addition" {
                each [
                    (2, 2, 4),
                    (0, 0, 0),
                    (-1, 1, 0),
                ] |a, b, expected| {
                    let _ = a + b == expected;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_ok());
        let group = first_group(parsed);
        match &group.children[0] {
            BehaveNode::Each(each) => {
                assert_eq!(each.label, "addition");
                assert_eq!(each.cases.len(), 3);
                assert_eq!(each.params.len(), 3);
                assert_eq!(each.params[0], "a");
                assert_eq!(each.params[1], "b");
                assert_eq!(each.params[2], "expected");
                assert!(!each.focused);
            }
            other => panic!("expected Each node, got {other:?}"),
        }
    }

    #[test]
    fn parse_each_single_param() {
        let input = quote::quote! {
            "positives" {
                each [1, 2, 3, 5, 8] |n| {
                    let _ = n;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_ok());
        let group = first_group(parsed);
        match &group.children[0] {
            BehaveNode::Each(each) => {
                assert_eq!(each.cases.len(), 5);
                assert_eq!(each.params.len(), 1);
                assert_eq!(each.params[0], "n");
            }
            other => panic!("expected Each node, got {other:?}"),
        }
    }

    #[test]
    fn parse_each_empty_cases_errors() {
        let input = quote::quote! {
            "empty" {
                each [] |n| {
                    let _ = n;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("at least one case"),
            "expected 'at least one case' error, got: {msg}"
        );
    }

    #[test]
    fn parse_each_focused() {
        let input = quote::quote! {
            focus "cases" {
                each [1, 2] |n| {
                    let _ = n;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_ok());
        let group = first_group(parsed);
        assert!(group.focused);
        match &group.children[0] {
            BehaveNode::Each(each) => {
                assert_eq!(each.cases.len(), 2);
            }
            other => panic!("expected Each node, got {other:?}"),
        }
    }

    #[test]
    fn parse_each_top_level() {
        let input = quote::quote! {
            "values" {
                each [10, 20] |v| {
                    let _ = v;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_timeout() {
        let input = quote::quote! {
            "suite" {
                timeout 5000;

                "test" {
                    let x = 1;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert_eq!(group.timeout_ms, Some(5000));
    }

    #[test]
    fn parse_timeout_with_tokio() {
        let input = quote::quote! {
            "suite" {
                tokio;
                timeout 1000;

                "test" {
                    let x = 1;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert!(group.async_runtime);
        assert_eq!(group.timeout_ms, Some(1000));
    }

    #[test]
    fn parse_timeout_zero_errors() {
        let input = quote::quote! {
            "suite" {
                timeout 0;

                "test" {
                    let x = 1;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("greater than 0"),
            "expected timeout > 0 error, got: {msg}"
        );
    }

    #[test]
    fn parse_timeout_with_setup() {
        let input = quote::quote! {
            "suite" {
                timeout 2000;

                setup {
                    let x = 1;
                }

                "test" {
                    let _ = x;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert_eq!(group.timeout_ms, Some(2000));
        assert!(group.setup.is_some());
    }

    #[test]
    fn parse_no_timeout() {
        let input = quote::quote! {
            "suite" {
                "test" {
                    let x = 1;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert_eq!(group.timeout_ms, None);
    }
}
