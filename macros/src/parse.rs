//! DSL parser for the `behave!` macro.
//!
//! Parses the token stream into a tree of [`BehaveNode`] values representing
//! groups (describe blocks) and tests (leaf blocks).

use syn::parse::{Parse, ParseStream};
use syn::{braced, bracketed, token, Expr, Ident, LitInt, LitStr, Result, Token, Type};

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
    /// A Cartesian-product parameterized test block.
    Matrix(MatrixNode),
    /// A typed test block generating tests for each type.
    EachType(EachTypeNode),
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
    /// Tags applied to this group (inherited through module path).
    pub tags: Vec<String>,
}

/// A leaf test node containing code to execute.
#[derive(Debug)]
pub struct TestNode {
    pub label: String,
    pub body: proc_macro2::TokenStream,
    pub focused: bool,
    /// Whether this test is expected to fail (`xfail` keyword).
    pub xfail: bool,
    /// Tags applied to this test.
    pub tags: Vec<String>,
}

/// A pending (ignored) test node.
#[derive(Debug)]
pub struct PendingNode {
    pub label: String,
}

/// A single test case in an `each` block, with an optional label.
#[derive(Debug)]
pub struct EachCase {
    /// Optional human-readable label (used as function name instead of `case_N`).
    pub label: Option<String>,
    /// The case expression to bind to parameters.
    pub expr: Expr,
}

/// A parameterized test node that generates one test per case.
#[derive(Debug)]
pub struct EachNode {
    /// Human-readable label for the generated module.
    pub label: String,
    /// The list of cases (one per generated test), each with optional name.
    pub cases: Vec<EachCase>,
    /// Parameter names bound from each case expression.
    pub params: Vec<Ident>,
    /// The test body executed for each case.
    pub body: proc_macro2::TokenStream,
    /// Whether this block was marked with `focus`.
    pub focused: bool,
    /// Whether this block was marked with `xfail` (expected failure).
    pub xfail: bool,
    /// Tags applied to this each block.
    pub tags: Vec<String>,
}

/// A Cartesian-product parameterized test block.
#[derive(Debug)]
pub struct MatrixNode {
    /// Human-readable label for the generated module.
    pub label: String,
    /// Each dimension is a list of expressions.
    pub dimensions: Vec<Vec<Expr>>,
    /// Parameter names bound from the Cartesian product.
    pub params: Vec<Ident>,
    /// The test body executed for each combination.
    pub body: proc_macro2::TokenStream,
    /// Whether this block was marked with `focus`.
    pub focused: bool,
    /// Whether this block was marked with `xfail` (expected failure).
    pub xfail: bool,
    /// Tags applied to this matrix block.
    pub tags: Vec<String>,
}

/// A typed test block that generates tests for each type in the list.
#[derive(Debug)]
pub struct EachTypeNode {
    /// Human-readable label for the generated module.
    pub label: String,
    /// The list of types to generate tests for.
    pub types: Vec<Type>,
    /// Child nodes inside the block (tests, groups, each, matrix, etc.).
    pub children: Vec<BehaveNode>,
    /// Optional setup block.
    pub setup: Option<proc_macro2::TokenStream>,
    /// Optional teardown block.
    pub teardown: Option<proc_macro2::TokenStream>,
    /// Whether async runtime is enabled.
    pub async_runtime: bool,
    /// Optional timeout in milliseconds.
    pub timeout_ms: Option<u64>,
    /// Whether this block was marked with `focus`.
    pub focused: bool,
    /// Tags applied to this block.
    pub tags: Vec<String>,
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
    let xfail = try_parse_keyword(input, "xfail");
    let pending = try_parse_keyword(input, "pending");

    if focused && pending {
        return Err(input.error("a test cannot be both `focus` and `pending`"));
    }
    if xfail && pending {
        return Err(input.error("a test cannot be both `xfail` and `pending`"));
    }

    let label: LitStr = input.parse()?;
    let label_str = label.value();
    let span = label.span();

    let tags = try_parse_tags(input)?;

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

    classify_block(&content, label_str, span, focused, xfail, tags)
}

fn classify_block(
    content: ParseStream<'_>,
    label: String,
    span: proc_macro2::Span,
    focused: bool,
    xfail: bool,
    tags: Vec<String>,
) -> Result<BehaveNode> {
    if content.is_empty() {
        return Err(syn::Error::new(span, "empty blocks are not allowed"));
    }

    if looks_like_matrix(content) {
        return parse_matrix_body(content, label, focused, xfail, tags);
    }

    if looks_like_each_type(content) {
        if xfail {
            return Err(syn::Error::new(
                span,
                "xfail cannot be applied to each_type blocks, only to tests or `each`/`matrix` blocks",
            ));
        }
        return parse_each_type_body(content, label, focused, tags);
    }

    if looks_like_each(content) {
        return parse_each_body(content, label, focused, xfail, tags);
    }

    if looks_like_group(content) {
        if xfail {
            return Err(syn::Error::new(
                span,
                "xfail cannot be applied to groups, only to tests or `each`/`matrix` blocks",
            ));
        }
        parse_group_body(content, label, span, focused, tags)
    } else {
        parse_test_body(content, label, focused, xfail, tags)
    }
}

/// Tries to consume `tag "name1", "name2"` from the input stream.
///
/// Returns an empty vec if no `tag` keyword is present. Validates that
/// tag strings are non-empty after slugification.
fn try_parse_tags(input: ParseStream<'_>) -> Result<Vec<String>> {
    if !input.peek(syn::Ident) {
        return Ok(Vec::new());
    }

    let fork = input.fork();
    let Ok(ident) = fork.parse::<syn::Ident>() else {
        return Ok(Vec::new());
    };

    if ident != "tag" || !fork.peek(LitStr) {
        return Ok(Vec::new());
    }

    // Consume `tag` from real stream
    let _tag: syn::Ident = input.parse()?;

    let mut tags = Vec::new();
    loop {
        let lit: LitStr = input.parse()?;
        let value = lit.value();
        if value.is_empty() {
            return Err(syn::Error::new(lit.span(), "tag name must not be empty"));
        }
        let slug = crate::slug::slugify(&value);
        if slug == "_unnamed" {
            return Err(syn::Error::new(
                lit.span(),
                "tag name must contain at least one alphanumeric character",
            ));
        }
        tags.push(value);
        if !input.peek(Token![,]) || input.peek2(token::Brace) {
            break;
        }
        let _comma: Token![,] = input.parse()?;
        if !input.peek(LitStr) {
            break;
        }
    }

    Ok(tags)
}

/// Peeks ahead to determine if this block starts with `matrix [`.
fn looks_like_matrix(input: ParseStream<'_>) -> bool {
    let fork = input.fork();
    let Ok(id) = fork.parse::<Ident>() else {
        return false;
    };
    id == "matrix" && fork.peek(token::Bracket)
}

/// Peeks ahead to determine if this block starts with `each [`.
fn looks_like_each(input: ParseStream<'_>) -> bool {
    let fork = input.fork();
    let Ok(id) = fork.parse::<Ident>() else {
        return false;
    };
    id == "each" && fork.peek(token::Bracket)
}

/// Peeks ahead to determine if this block starts with `each_type [`.
fn looks_like_each_type(input: ParseStream<'_>) -> bool {
    let fork = input.fork();
    let Ok(id) = fork.parse::<Ident>() else {
        return false;
    };
    id == "each_type" && fork.peek(token::Bracket)
}

/// Parses `each_type [Type1, Type2] { children }` into an [`EachTypeNode`].
fn parse_each_type_body(
    input: ParseStream<'_>,
    label: String,
    focused: bool,
    tags: Vec<String>,
) -> Result<BehaveNode> {
    let _each_type: Ident = input.parse()?;

    // Parse `[Type1, Type2, ...]`
    let types_content;
    bracketed!(types_content in input);
    let punctuated =
        syn::punctuated::Punctuated::<Type, Token![,]>::parse_terminated(&types_content)?;
    let types: Vec<Type> = punctuated.into_iter().collect();

    if types.is_empty() {
        return Err(input.error("each_type must contain at least one type"));
    }

    // Parse `{ inner group body }`
    let inner;
    braced!(inner in input);

    let async_runtime = try_parse_async_runtime(&inner)?;
    let timeout_ms = try_parse_timeout(&inner)?;
    let setup = try_parse_setup(&inner)?;
    let teardown = try_parse_teardown(&inner)?;

    let mut children = Vec::new();
    while !inner.is_empty() {
        children.push(parse_node(&inner)?);
    }

    if children.is_empty() {
        return Err(input.error("each_type block must contain at least one test or group"));
    }

    Ok(BehaveNode::EachType(EachTypeNode {
        label,
        types,
        children,
        setup,
        teardown,
        async_runtime,
        timeout_ms,
        focused,
        tags,
    }))
}

/// Parses `matrix [a, b] x [c, d] |p1, p2| { body }` into a [`MatrixNode`].
///
/// Each dimension is a bracket-delimited list of expressions separated by the
/// `x` identifier. The parameter count must equal the dimension count.
fn parse_matrix_body(
    input: ParseStream<'_>,
    label: String,
    focused: bool,
    xfail: bool,
    tags: Vec<String>,
) -> Result<BehaveNode> {
    let _matrix: Ident = input.parse()?;
    let mut dimensions = Vec::new();

    // Parse first dimension `[a, b, ...]`
    dimensions.push(parse_dimension(input)?);

    // Parse remaining `x [c, d, ...]` dimensions
    while input.peek(Ident) {
        let fork = input.fork();
        let id: Ident = fork.parse()?;
        if id != "x" {
            break;
        }
        let _x: Ident = input.parse()?;
        dimensions.push(parse_dimension(input)?);
    }

    if dimensions.len() < 2 {
        return Err(input.error("matrix requires at least 2 dimensions separated by `x`"));
    }

    // Parse `|param1, param2, ...|`
    let _pipe1: Token![|] = input.parse()?;
    let params_punctuated =
        syn::punctuated::Punctuated::<Ident, Token![,]>::parse_separated_nonempty(input)?;
    let params: Vec<Ident> = params_punctuated.into_iter().collect();
    let _pipe2: Token![|] = input.parse()?;

    if params.len() != dimensions.len() {
        return Err(input.error(format!(
            "matrix has {} dimensions but {} parameters",
            dimensions.len(),
            params.len(),
        )));
    }

    // Parse `{ body }`
    let body_content;
    braced!(body_content in input);
    let body: proc_macro2::TokenStream = body_content.parse()?;

    Ok(BehaveNode::Matrix(MatrixNode {
        label,
        dimensions,
        params,
        body,
        focused,
        xfail,
        tags,
    }))
}

/// Parses a single `[expr, expr, ...]` dimension for matrix blocks.
fn parse_dimension(input: ParseStream<'_>) -> Result<Vec<Expr>> {
    let content;
    bracketed!(content in input);
    let punctuated = syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;
    let exprs: Vec<Expr> = punctuated.into_iter().collect();
    if exprs.is_empty() {
        return Err(input.error("matrix dimension must contain at least one value"));
    }
    Ok(exprs)
}

/// Parses `each [cases] |params| { body }` into an [`EachNode`].
///
/// Cases can optionally have a string label as the first tuple element:
/// `each [("ok", 200, true), ("not_found", 404, false)] |name, code, ok| { ... }`
/// The label becomes the test function name instead of `case_N`.
fn parse_each_body(
    input: ParseStream<'_>,
    label: String,
    focused: bool,
    xfail: bool,
    tags: Vec<String>,
) -> Result<BehaveNode> {
    // Consume `each`
    let _each: Ident = input.parse()?;

    // Parse `[case1, case2, ...]`
    let cases_content;
    bracketed!(cases_content in input);
    let punctuated =
        syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated(&cases_content)?;
    let raw_cases: Vec<Expr> = punctuated.into_iter().collect();

    if raw_cases.is_empty() {
        return Err(input.error("each block must contain at least one case"));
    }

    let cases = raw_cases
        .into_iter()
        .map(extract_named_case)
        .collect::<Result<Vec<_>>>()?;

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
        xfail,
        tags,
    }))
}

/// Checks if a case expression is a tuple whose first element is a string
/// literal. If so, extracts the label and rebuilds the expression without it.
fn extract_named_case(expr: Expr) -> Result<EachCase> {
    if let Expr::Tuple(ref tuple) = expr {
        if let Some(Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        })) = tuple.elems.first()
        {
            let label = lit_str.value();
            let mut rest = tuple.elems.iter().skip(1).cloned();
            let Some(first_value) = rest.next() else {
                return Err(syn::Error::new_spanned(
                    tuple,
                    "named each case must have at least one value after the label",
                ));
            };

            // Single remaining value: unwrap from tuple.
            let Some(second_value) = rest.next() else {
                return Ok(EachCase {
                    label: Some(label),
                    expr: first_value,
                });
            };

            // Multiple remaining values: rebuild as tuple.
            let mut elems = syn::punctuated::Punctuated::new();
            elems.push(first_value);
            elems.push(second_value);
            for val in rest {
                elems.push(val);
            }
            let new_tuple = syn::ExprTuple {
                attrs: tuple.attrs.clone(),
                paren_token: tuple.paren_token,
                elems,
            };
            return Ok(EachCase {
                label: Some(label),
                expr: Expr::Tuple(new_tuple),
            });
        }
    }
    Ok(EachCase { label: None, expr })
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
        if id == "focus" || id == "pending" || id == "xfail" {
            let _ = fork.parse::<syn::Ident>();
            return fork.peek(LitStr);
        }
        // Unknown ident — treat as test code
        return false;
    }

    // After optional prefixes, expect `"label"` then optional `tag "...", "..."` then `{`
    if !fork.peek(LitStr) {
        return false;
    }
    let _: std::result::Result<LitStr, _> = fork.parse();

    // Skip past optional `tag "...", "..."` sequence
    if fork.peek(syn::Ident) {
        let inner = fork.fork();
        if let Ok(id) = inner.parse::<syn::Ident>() {
            if id == "tag" && inner.peek(LitStr) {
                let _ = fork.parse::<syn::Ident>();
                // consume tag string literals separated by commas
                while fork.peek(LitStr) {
                    let _ = fork.parse::<LitStr>();
                    if fork.peek(Token![,]) && !fork.peek2(token::Brace) {
                        let _ = fork.parse::<Token![,]>();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fork.peek(token::Brace)
}

fn peek_keyword(input: ParseStream<'_>, keyword: &str) -> bool {
    if !input.peek(syn::Ident) {
        return false;
    }
    let fork = input.fork();
    fork.parse::<syn::Ident>()
        .is_ok_and(|id| id == keyword && fork.peek(token::Brace))
}

fn parse_group_body(
    content: ParseStream<'_>,
    label: String,
    span: proc_macro2::Span,
    focused: bool,
    tags: Vec<String>,
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
        tags,
    }))
}

fn parse_test_body(
    content: ParseStream<'_>,
    label: String,
    focused: bool,
    xfail: bool,
    tags: Vec<String>,
) -> Result<BehaveNode> {
    let body: proc_macro2::TokenStream = content.parse()?;
    Ok(BehaveNode::Test(TestNode {
        label,
        body,
        focused,
        xfail,
        tags,
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
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
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
        let each = first_each(parse_input(input));
        assert_eq!(each.label, "addition");
        assert_eq!(each.cases.len(), 3);
        assert_eq!(each.params.len(), 3);
        assert_eq!(each.params[0], "a");
        assert_eq!(each.params[1], "b");
        assert_eq!(each.params[2], "expected");
        assert!(!each.focused);
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
        let each = first_each(parse_input(input));
        assert_eq!(each.cases.len(), 5);
        assert_eq!(each.params.len(), 1);
        assert_eq!(each.params[0], "n");
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
    fn parse_each_named_cases() {
        let input = quote::quote! {
            "http status" {
                each [
                    ("ok", 200, true),
                    ("not_found", 404, false),
                ] |name, code, success| {
                    let _ = (code, success);
                }
            }
        };
        let each = first_each(parse_input(input));
        assert_eq!(each.cases.len(), 2);
        assert_eq!(each.cases[0].label.as_deref(), Some("ok"));
        assert_eq!(each.cases[1].label.as_deref(), Some("not_found"));
        assert_eq!(each.params.len(), 3);
    }

    #[test]
    fn parse_each_named_single_value() {
        let input = quote::quote! {
            "items" {
                each [
                    ("small", 1),
                    ("large", 100),
                ] |name, n| {
                    let _ = n;
                }
            }
        };
        let each = first_each(parse_input(input));
        assert_eq!(each.cases[0].label.as_deref(), Some("small"));
        assert_eq!(each.cases[1].label.as_deref(), Some("large"));
    }

    #[test]
    fn parse_each_named_case_without_value_errors() {
        let input = quote::quote! {
            "items" {
                each [
                    ("empty",),
                ] |name| {
                    let _ = name;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("at least one value"),
            "expected named-case missing value error, got: {msg}"
        );
    }

    #[test]
    fn parse_each_unnamed_cases_preserved() {
        let input = quote::quote! {
            "values" {
                each [
                    (1, 2),
                    (3, 4),
                ] |a, b| {
                    let _ = a + b;
                }
            }
        };
        let each = first_each(parse_input(input));
        assert!(each.cases[0].label.is_none());
        assert!(each.cases[1].label.is_none());
    }

    #[test]
    fn parse_each_mixed_named_unnamed() {
        let input = quote::quote! {
            "items" {
                each [
                    ("labeled", 1, true),
                    (2, false),
                ] |n, flag| {
                    let _ = (n, flag);
                }
            }
        };
        let each = first_each(parse_input(input));
        assert_eq!(each.cases[0].label.as_deref(), Some("labeled"));
        assert!(each.cases[1].label.is_none());
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
        let each = first_each(parse_input(input));
        assert!(each.focused);
        assert_eq!(each.cases.len(), 2);
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

    // --- tag tests ---

    #[test]
    fn parse_single_tag_on_test() {
        let input = quote::quote! {
            "my test" tag "slow" {
                let x = 1;
            }
        };
        let test = first_test(parse_input(input));
        assert_eq!(test.tags, vec!["slow"]);
    }

    #[test]
    fn parse_multiple_tags() {
        let input = quote::quote! {
            "my test" tag "slow", "integration" {
                let x = 1;
            }
        };
        let test = first_test(parse_input(input));
        assert_eq!(test.tags, vec!["slow", "integration"]);
    }

    #[test]
    fn parse_tag_on_group() {
        let input = quote::quote! {
            "suite" tag "integration" {
                "test" {
                    let x = 1;
                }
            }
        };
        let group = first_group(parse_input(input));
        assert_eq!(group.tags, vec!["integration"]);
    }

    #[test]
    fn parse_tag_on_each() {
        let input = quote::quote! {
            "cases" tag "unit" {
                each [1, 2] |n| {
                    let _ = n;
                }
            }
        };
        let each = first_each(parse_input(input));
        assert_eq!(each.tags, vec!["unit"]);
    }

    #[test]
    fn parse_tag_on_matrix() {
        let input = quote::quote! {
            "combos" tag "slow" {
                matrix [1, 2] x [3, 4] |a, b| {
                    let _ = a + b;
                }
            }
        };
        let matrix = first_matrix(parse_input(input));
        assert_eq!(matrix.tags, vec!["slow"]);
    }

    #[test]
    fn parse_focus_with_tag() {
        let input = quote::quote! {
            focus "important" tag "critical" {
                let x = 1;
            }
        };
        let test = first_test(parse_input(input));
        assert!(test.focused);
        assert_eq!(test.tags, vec!["critical"]);
    }

    #[test]
    fn parse_xfail_with_tag() {
        let input = quote::quote! {
            xfail "broken" tag "known_issue" {
                let x = 1;
            }
        };
        let test = first_test(parse_input(input));
        assert!(test.xfail);
        assert_eq!(test.tags, vec!["known_issue"]);
    }

    #[test]
    fn parse_no_tags_backward_compat() {
        let input = quote::quote! {
            "simple test" {
                let x = 1;
            }
        };
        let test = first_test(parse_input(input));
        assert!(test.tags.is_empty());
    }

    #[test]
    fn parse_empty_tag_errors() {
        let input = quote::quote! {
            "test" tag "" {
                let x = 1;
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("must not be empty"),
            "expected empty tag error, got: {msg}"
        );
    }

    // --- xfail tests ---

    #[test]
    fn parse_xfail_test() {
        let input = quote::quote! {
            xfail "expected failure" {
                let x = 1;
            }
        };
        let test = first_test(parse_input(input));
        assert!(test.xfail);
        assert!(!test.focused);
        assert_eq!(test.label, "expected failure");
    }

    #[test]
    fn parse_xfail_each() {
        let input = quote::quote! {
            xfail "broken cases" {
                each [1, 2] |n| {
                    let _ = n;
                }
            }
        };
        let each = first_each(parse_input(input));
        assert!(each.xfail);
        assert_eq!(each.cases.len(), 2);
    }

    #[test]
    fn parse_xfail_group_errors() {
        let input = quote::quote! {
            xfail "suite" {
                "test" {
                    let x = 1;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("xfail cannot be applied to groups"),
            "expected xfail-group error, got: {msg}"
        );
    }

    #[test]
    fn parse_xfail_pending_errors() {
        let input = quote::quote! {
            xfail pending "both" {
                let x = 1;
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_non_xfail_test_has_xfail_false() {
        let input = quote::quote! {
            "normal test" {
                let x = 1;
            }
        };
        let test = first_test(parse_input(input));
        assert!(!test.xfail);
    }

    // --- matrix tests ---

    /// Unwrap a successful parse and return the first node as a Matrix.
    fn first_matrix(input: Result<BehaveInput>) -> MatrixNode {
        let parsed = input.expect("parse should succeed");
        match parsed.nodes.into_iter().next() {
            Some(BehaveNode::Matrix(m)) => m,
            other => panic!("expected Matrix node, got {other:?}"),
        }
    }

    #[test]
    fn parse_matrix_basic() {
        let input = quote::quote! {
            "combos" {
                matrix [1, 2] x [10, 20] |a, b| {
                    let _ = a + b;
                }
            }
        };
        let matrix = first_matrix(parse_input(input));
        assert_eq!(matrix.label, "combos");
        assert_eq!(matrix.dimensions.len(), 2);
        assert_eq!(matrix.dimensions[0].len(), 2);
        assert_eq!(matrix.dimensions[1].len(), 2);
        assert_eq!(matrix.params.len(), 2);
        assert_eq!(matrix.params[0], "a");
        assert_eq!(matrix.params[1], "b");
        assert!(!matrix.focused);
        assert!(!matrix.xfail);
    }

    #[test]
    fn parse_matrix_three_dimensions() {
        let input = quote::quote! {
            "3d" {
                matrix [1, 2] x ["a", "b"] x [true, false] |n, s, b| {
                    let _ = (n, s, b);
                }
            }
        };
        let matrix = first_matrix(parse_input(input));
        assert_eq!(matrix.dimensions.len(), 3);
        assert_eq!(matrix.params.len(), 3);
    }

    #[test]
    fn parse_matrix_single_dimension_errors() {
        let input = quote::quote! {
            "single" {
                matrix [1, 2] |n| {
                    let _ = n;
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("at least 2 dimensions"),
            "expected dimension count error, got: {msg}"
        );
    }

    #[test]
    fn parse_matrix_param_mismatch_errors() {
        let input = quote::quote! {
            "mismatch" {
                matrix [1, 2] x [10, 20] |a, b, c| {
                    let _ = (a, b, c);
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("2 dimensions but 3 parameters"),
            "expected param mismatch error, got: {msg}"
        );
    }

    #[test]
    fn parse_matrix_empty_dimension_errors() {
        let input = quote::quote! {
            "empty" {
                matrix [] x [1] |a, b| {
                    let _ = (a, b);
                }
            }
        };
        let parsed = parse_input(input);
        assert!(parsed.is_err());
        let msg = parsed.unwrap_err().to_string();
        assert!(
            msg.contains("at least one value"),
            "expected empty dimension error, got: {msg}"
        );
    }

    #[test]
    fn parse_matrix_focused() {
        let input = quote::quote! {
            focus "focused combos" {
                matrix [1, 2] x [10, 20] |a, b| {
                    let _ = a + b;
                }
            }
        };
        let matrix = first_matrix(parse_input(input));
        assert!(matrix.focused);
    }

    #[test]
    fn parse_matrix_xfail() {
        let input = quote::quote! {
            xfail "broken combos" {
                matrix [1, 2] x [10, 20] |a, b| {
                    let _ = a + b;
                }
            }
        };
        let matrix = first_matrix(parse_input(input));
        assert!(matrix.xfail);
    }
}
