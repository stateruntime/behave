//! DSL parser for the `behave!` macro.
//!
//! Parses the token stream into a tree of [`BehaveNode`] values representing
//! groups (describe blocks) and tests (leaf blocks).

use syn::parse::{Parse, ParseStream};
use syn::{braced, token, LitStr, Result};

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
}

/// A group node that contains child nodes.
#[derive(Debug)]
pub struct GroupNode {
    pub label: String,
    pub children: Vec<BehaveNode>,
    pub setup: Option<proc_macro2::TokenStream>,
    pub teardown: Option<proc_macro2::TokenStream>,
    pub async_runtime: bool,
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

    if looks_like_group(content) {
        parse_group_body(content, label, span, focused)
    } else {
        parse_test_body(content, label, focused)
    }
}

/// Peeks ahead to determine if this block contains child groups
/// (string literal followed by braces) or test code.
///
/// Skips any combination of `tokio;`, `setup {}`, and `teardown {}`
/// prefixes. Validation of ordering and duplicates happens later in
/// [`parse_group_body`].
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
}
