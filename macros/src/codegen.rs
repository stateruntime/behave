//! Code generation from the parsed behave DSL AST.
//!
//! Transforms [`BehaveNode`] trees into Rust `#[test]` functions
//! organized inside nested modules.

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::Expr;

use crate::parse::{
    BehaveInput, BehaveNode, EachCase, EachNode, EachTypeNode, GroupNode, MatrixNode, PendingNode,
    TestNode,
};
use crate::slug::{is_rust_keyword, slugify};

/// Creates an identifier, using a raw identifier for Rust keywords.
fn make_ident(name: &str) -> Ident {
    if is_rust_keyword(name) {
        Ident::new_raw(name, Span::call_site())
    } else {
        Ident::new(name, Span::call_site())
    }
}

/// Builds a prefixed name from a slug, focus flag, and tag list.
///
/// Prefix order: `__FOCUS__` → `__TAG_xxx__` (per tag, slugified) → slug.
fn build_prefixed_name(slug: &str, focused: bool, tags: &[String]) -> Ident {
    let mut prefixed = String::new();
    if focused {
        prefixed.push_str("__FOCUS__");
    }
    for tag in tags {
        let tag_slug = slugify(tag);
        prefixed.push_str("__TAG_");
        prefixed.push_str(&tag_slug);
        prefixed.push_str("__");
    }
    prefixed.push_str(slug);
    format_ident!("{prefixed}")
}

/// Context threaded through code generation for inherited state.
struct GenContext<'a> {
    setups: Vec<&'a TokenStream>,
    teardowns: Vec<&'a TokenStream>,
    is_async: bool,
    timeout_ms: Option<u64>,
}

/// Generates Rust test code from a parsed [`BehaveInput`].
///
/// # Errors
///
/// Returns `syn::Error` if code generation encounters an invalid AST state.
pub fn generate(input: BehaveInput) -> syn::Result<TokenStream> {
    let ctx = GenContext {
        setups: Vec::new(),
        teardowns: Vec::new(),
        is_async: false,
        timeout_ms: None,
    };
    let mut tokens = TokenStream::new();
    for node in input.nodes {
        tokens.extend(generate_node(&node, &ctx)?);
    }
    Ok(tokens)
}

fn generate_node(node: &BehaveNode, ctx: &GenContext<'_>) -> syn::Result<TokenStream> {
    match node {
        BehaveNode::Group(group) => generate_group(group, ctx),
        BehaveNode::Test(test) => Ok(generate_test(test, ctx)),
        BehaveNode::Pending(pending) => Ok(generate_pending(pending)),
        BehaveNode::Each(each) => Ok(generate_each(each, ctx)),
        BehaveNode::Matrix(matrix) => Ok(generate_matrix(matrix, ctx)),
        BehaveNode::EachType(each_type) => generate_each_type(each_type, ctx),
    }
}

fn generate_group(group: &GroupNode, ctx: &GenContext<'_>) -> syn::Result<TokenStream> {
    let slug = slugify(&group.label);
    let mod_name = if group.focused || !group.tags.is_empty() {
        build_prefixed_name(&slug, group.focused, &group.tags)
    } else {
        make_ident(&slug)
    };

    let mut child_ctx = GenContext {
        setups: ctx.setups.clone(),
        teardowns: ctx.teardowns.clone(),
        is_async: ctx.is_async || group.async_runtime,
        timeout_ms: group.timeout_ms.or(ctx.timeout_ms),
    };
    if let Some(ref setup) = group.setup {
        child_ctx.setups.push(setup);
    }
    if let Some(ref teardown) = group.teardown {
        child_ctx.teardowns.push(teardown);
    }

    let mut children_tokens = TokenStream::new();
    for child in &group.children {
        children_tokens.extend(generate_node(child, &child_ctx)?);
    }

    Ok(quote! {
        mod #mod_name {
            use super::*;
            #children_tokens
        }
    })
}

fn generate_test(test: &TestNode, ctx: &GenContext<'_>) -> TokenStream {
    let fn_name = if test.focused || !test.tags.is_empty() {
        build_prefixed_name(&slugify(&test.label), test.focused, &test.tags)
    } else {
        make_ident(&slugify(&test.label))
    };

    let empty = TokenStream::new();
    generate_test_fn(&fn_name, &empty, &test.body, ctx, test.xfail)
}

/// Wraps a test body for `xfail`: catches `Err` → passes, `Ok` → fails.
fn wrap_xfail_sync(body: &TokenStream) -> TokenStream {
    quote! {
        let __behave_xfail: Result<(), Box<dyn std::error::Error>> = (|| {
            #body
            Ok(())
        })();
        match __behave_xfail {
            Ok(()) => {
                return Err("xfail: expected test to fail, but it passed".into());
            }
            Err(_) => {}
        }
    }
}

/// Wraps an async test body for `xfail`: catches `Err` → passes, `Ok` → fails.
fn wrap_xfail_async(body: &TokenStream) -> TokenStream {
    quote! {
        let __behave_xfail: Result<(), Box<dyn std::error::Error>> = async {
            #body
            Ok(())
        }.await;
        match __behave_xfail {
            Ok(()) => {
                return Err("xfail: expected test to fail, but it passed".into());
            }
            Err(_) => {}
        }
    }
}

/// Shared helper that emits a single `#[test]` (or `#[tokio::test]`) function.
///
/// `extra_setup` is prepended after inherited setups — used by `each` to bind
/// case parameters. Regular tests pass an empty stream.
fn generate_test_fn(
    fn_name: &Ident,
    extra_setup: &TokenStream,
    body: &TokenStream,
    ctx: &GenContext<'_>,
    xfail: bool,
) -> TokenStream {
    let xfail_body;
    let effective_body = if xfail {
        xfail_body = if ctx.is_async {
            wrap_xfail_async(body)
        } else {
            wrap_xfail_sync(body)
        };
        &xfail_body
    } else {
        body
    };

    let setup_tokens: TokenStream = ctx.setups.iter().map(|s| quote! { #s }).collect();
    let has_teardown = !ctx.teardowns.is_empty();
    let teardown_tokens: TokenStream = ctx.teardowns.iter().rev().map(|t| quote! { #t }).collect();

    match (ctx.is_async, has_teardown, ctx.timeout_ms) {
        (false, false, None) => gen_sync(fn_name, &setup_tokens, extra_setup, effective_body),
        (false, true, None) => gen_sync_teardown(
            fn_name,
            &setup_tokens,
            extra_setup,
            effective_body,
            &teardown_tokens,
        ),
        (true, false, None) => gen_async(fn_name, &setup_tokens, extra_setup, effective_body),
        (true, true, None) => gen_async_teardown(
            fn_name,
            &setup_tokens,
            extra_setup,
            effective_body,
            &teardown_tokens,
        ),
        (false, false, Some(ms)) => {
            gen_sync_timeout(fn_name, &setup_tokens, extra_setup, effective_body, ms)
        }
        (false, true, Some(ms)) => gen_sync_teardown_timeout(
            fn_name,
            &setup_tokens,
            extra_setup,
            effective_body,
            &teardown_tokens,
            ms,
        ),
        (true, false, Some(ms)) => {
            gen_async_timeout(fn_name, &setup_tokens, extra_setup, effective_body, ms)
        }
        (true, true, Some(ms)) => gen_async_teardown_timeout(
            fn_name,
            &setup_tokens,
            extra_setup,
            effective_body,
            &teardown_tokens,
            ms,
        ),
    }
}

/// Sync test, no teardown, no timeout.
fn gen_sync(
    fn_name: &Ident,
    setup: &TokenStream,
    extra: &TokenStream,
    body: &TokenStream,
) -> TokenStream {
    quote! {
        #[test]
        fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
            #setup
            #extra
            #body
            Ok(())
        }
    }
}

/// Sync test with teardown (`catch_unwind` to guarantee cleanup), no timeout.
fn gen_sync_teardown(
    fn_name: &Ident,
    setup: &TokenStream,
    extra: &TokenStream,
    body: &TokenStream,
    teardown: &TokenStream,
) -> TokenStream {
    quote! {
        #[test]
        fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
            #setup
            #extra
            let __behave_test_result = std::panic::catch_unwind(
                std::panic::AssertUnwindSafe(|| -> Result<(), Box<dyn std::error::Error>> {
                    #body
                    Ok(())
                })
            );
            #teardown
            match __behave_test_result {
                Ok(result) => result,
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
    }
}

/// Async test, no teardown, no timeout.
fn gen_async(
    fn_name: &Ident,
    setup: &TokenStream,
    extra: &TokenStream,
    body: &TokenStream,
) -> TokenStream {
    quote! {
        #[tokio::test]
        async fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
            #setup
            #extra
            #body
            Ok(())
        }
    }
}

/// Async test with teardown (runs after async body completes), no timeout.
fn gen_async_teardown(
    fn_name: &Ident,
    setup: &TokenStream,
    extra: &TokenStream,
    body: &TokenStream,
    teardown: &TokenStream,
) -> TokenStream {
    quote! {
        #[tokio::test]
        async fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
            #setup
            #extra
            let __behave_test_result: Result<(), Box<dyn std::error::Error>> = async {
                #body
                Ok(())
            }.await;
            #teardown
            __behave_test_result
        }
    }
}

/// Sync test with timeout, no teardown. Spawns a thread and waits with `recv_timeout`.
fn gen_sync_timeout(
    fn_name: &Ident,
    setup: &TokenStream,
    extra: &TokenStream,
    body: &TokenStream,
    ms: u64,
) -> TokenStream {
    let err_msg = format!("test timed out after {ms}ms");
    quote! {
        #[test]
        fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
            let (__behave_tx, __behave_rx) = std::sync::mpsc::channel::<Result<(), String>>();
            std::thread::spawn(move || {
                let __behave_r: Result<(), Box<dyn std::error::Error>> = (|| {
                    #setup
                    #extra
                    #body
                    Ok(())
                })();
                let _ = __behave_tx.send(__behave_r.map_err(|e| e.to_string()));
            });
            match __behave_rx.recv_timeout(std::time::Duration::from_millis(#ms)) {
                Ok(Ok(())) => Ok(()),
                Ok(Err(msg)) => Err(msg.into()),
                Err(_) => Err(#err_msg.into()),
            }
        }
    }
}

/// Sync test with teardown and timeout. Setup + body + teardown all run inside
/// the spawned thread so teardown can access setup variables.
fn gen_sync_teardown_timeout(
    fn_name: &Ident,
    setup: &TokenStream,
    extra: &TokenStream,
    body: &TokenStream,
    teardown: &TokenStream,
    ms: u64,
) -> TokenStream {
    let err_msg = format!("test timed out after {ms}ms");
    quote! {
        #[test]
        fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
            let (__behave_tx, __behave_rx) = std::sync::mpsc::channel::<Result<(), String>>();
            std::thread::spawn(move || {
                #setup
                #extra
                let __behave_r = std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(|| -> Result<(), Box<dyn std::error::Error>> {
                        #body
                        Ok(())
                    })
                );
                #teardown
                let __behave_final = match __behave_r {
                    Ok(result) => result.map_err(|e| e.to_string()),
                    Err(_) => Err("test panicked".to_string()),
                };
                let _ = __behave_tx.send(__behave_final);
            });
            match __behave_rx.recv_timeout(std::time::Duration::from_millis(#ms)) {
                Ok(Ok(())) => Ok(()),
                Ok(Err(msg)) => Err(msg.into()),
                Err(_) => Err(#err_msg.into()),
            }
        }
    }
}

/// Async test with timeout, no teardown. Wraps body in `tokio::time::timeout`.
fn gen_async_timeout(
    fn_name: &Ident,
    setup: &TokenStream,
    extra: &TokenStream,
    body: &TokenStream,
    ms: u64,
) -> TokenStream {
    let err_msg = format!("test timed out after {ms}ms");
    quote! {
        #[tokio::test]
        async fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
            #setup
            #extra
            match tokio::time::timeout(
                std::time::Duration::from_millis(#ms),
                async {
                    #body
                    Ok::<(), Box<dyn std::error::Error>>(())
                }
            ).await {
                Ok(result) => result,
                Err(_) => Err(#err_msg.into()),
            }
        }
    }
}

/// Async test with teardown and timeout. Teardown runs after the timeout
/// wrapper so cleanup happens regardless of whether the body timed out.
fn gen_async_teardown_timeout(
    fn_name: &Ident,
    setup: &TokenStream,
    extra: &TokenStream,
    body: &TokenStream,
    teardown: &TokenStream,
    ms: u64,
) -> TokenStream {
    let err_msg = format!("test timed out after {ms}ms");
    quote! {
        #[tokio::test]
        async fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
            #setup
            #extra
            let __behave_test_result = match tokio::time::timeout(
                std::time::Duration::from_millis(#ms),
                async {
                    #body
                    Ok::<(), Box<dyn std::error::Error>>(())
                }
            ).await {
                Ok(result) => result,
                Err(_) => Err(#err_msg.into()),
            };
            #teardown
            __behave_test_result
        }
    }
}

fn generate_each(each: &EachNode, ctx: &GenContext<'_>) -> TokenStream {
    let slug = slugify(&each.label);
    let mod_name = if each.focused || !each.tags.is_empty() {
        build_prefixed_name(&slug, each.focused, &each.tags)
    } else {
        make_ident(&slug)
    };

    let case_fns: TokenStream = each
        .cases
        .iter()
        .enumerate()
        .map(|(i, case)| {
            let fn_name = each_case_fn_name(case, i);
            let case_expr = &case.expr;
            let params = &each.params;

            // Single param: `let p = expr;`  Multi param: `let (p1, p2) = expr;`
            let binding = if params.len() == 1 {
                let p = &params[0];
                quote! { let #p = #case_expr; }
            } else {
                quote! { let (#(#params),*) = #case_expr; }
            };

            generate_test_fn(&fn_name, &binding, &each.body, ctx, each.xfail)
        })
        .collect();

    quote! {
        mod #mod_name {
            use super::*;
            #case_fns
        }
    }
}

fn generate_matrix(matrix: &MatrixNode, ctx: &GenContext<'_>) -> TokenStream {
    let slug = slugify(&matrix.label);
    let mod_name = if matrix.focused || !matrix.tags.is_empty() {
        build_prefixed_name(&slug, matrix.focused, &matrix.tags)
    } else {
        make_ident(&slug)
    };

    let combos = cartesian_product(&matrix.dimensions);
    let case_fns: TokenStream = combos
        .iter()
        .enumerate()
        .map(|(i, combo)| {
            let fn_name = matrix_case_fn_name(&matrix.dimensions, combo, i);
            let params = &matrix.params;
            let bindings: TokenStream = params
                .iter()
                .zip(combo.iter())
                .map(|(p, expr)| quote! { let #p = #expr; })
                .collect();
            generate_test_fn(&fn_name, &bindings, &matrix.body, ctx, matrix.xfail)
        })
        .collect();

    quote! {
        mod #mod_name {
            use super::*;
            #case_fns
        }
    }
}

fn generate_each_type(each_type: &EachTypeNode, ctx: &GenContext<'_>) -> syn::Result<TokenStream> {
    let slug = slugify(&each_type.label);
    let mod_name = if each_type.focused || !each_type.tags.is_empty() {
        build_prefixed_name(&slug, each_type.focused, &each_type.tags)
    } else {
        make_ident(&slug)
    };

    let mut child_ctx = GenContext {
        setups: ctx.setups.clone(),
        teardowns: ctx.teardowns.clone(),
        is_async: ctx.is_async || each_type.async_runtime,
        timeout_ms: each_type.timeout_ms.or(ctx.timeout_ms),
    };
    if let Some(ref setup) = each_type.setup {
        child_ctx.setups.push(setup);
    }
    if let Some(ref teardown) = each_type.teardown {
        child_ctx.teardowns.push(teardown);
    }

    let mut children_tokens = TokenStream::new();
    for child in &each_type.children {
        children_tokens.extend(generate_node(child, &child_ctx)?);
    }

    let type_modules: TokenStream = each_type
        .types
        .iter()
        .map(|ty| {
            let type_slug = slugify(&quote!(#ty).to_string());
            let type_mod = make_ident(&type_slug);
            quote! {
                mod #type_mod {
                    use super::*;
                    #[allow(dead_code)]
                    type T = #ty;
                    #children_tokens
                }
            }
        })
        .collect();

    Ok(quote! {
        mod #mod_name {
            use super::*;
            #type_modules
        }
    })
}

/// Computes the Cartesian product of all dimensions as index-expression tuples.
fn cartesian_product(dimensions: &[Vec<Expr>]) -> Vec<Vec<&Expr>> {
    let mut result: Vec<Vec<&Expr>> = vec![vec![]];
    for dim in dimensions {
        let mut next = Vec::new();
        for combo in &result {
            for expr in dim {
                let mut extended = combo.clone();
                extended.push(expr);
                next.push(extended);
            }
        }
        result = next;
    }
    result
}

/// Generates `case_I_J` names from dimension indices.
fn matrix_case_fn_name(dimensions: &[Vec<Expr>], combo: &[&Expr], fallback: usize) -> Ident {
    let indices: Vec<usize> = combo
        .iter()
        .zip(dimensions.iter())
        .map(|(expr, dim)| {
            dim.iter()
                .position(|e| token_eq(e, expr))
                .unwrap_or(fallback)
        })
        .collect();
    let name = indices
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join("_");
    format_ident!("case_{name}")
}

/// Compares two expressions by their token representation.
fn token_eq(a: &Expr, b: &Expr) -> bool {
    quote!(#a).to_string() == quote!(#b).to_string()
}

/// Determines the function name for an `each` case.
///
/// If the case has a string label, slugify it. Otherwise, use `case_N`.
fn each_case_fn_name(case: &EachCase, index: usize) -> Ident {
    case.label.as_ref().map_or_else(
        || format_ident!("case_{index}"),
        |label| make_ident(&slugify(label)),
    )
}

fn generate_pending(pending: &PendingNode) -> TokenStream {
    let fn_name = format_ident!("__PENDING__{}", slugify(&pending.label));

    quote! {
        #[test]
        #[ignore = "pending"]
        fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::parse::BehaveInput;

    fn parse_and_generate(input: TokenStream) -> syn::Result<TokenStream> {
        let parsed: BehaveInput = syn::parse2(input)?;
        generate(parsed)
    }

    #[test]
    fn generates_simple_test() {
        let input = quote::quote! {
            "math" {
                "adds numbers" {
                    let x = 1 + 1;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
    }

    #[test]
    fn generates_pending_test() {
        let input = quote::quote! {
            "suite" {
                pending "not done" {}
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("ignore"));
    }

    #[test]
    fn generates_setup_inheritance() {
        let input = quote::quote! {
            "outer" {
                setup {
                    let base = 10;
                }

                "inner" {
                    setup {
                        let extra = 5;
                    }

                    "test" {
                        let _ = base + extra;
                    }
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("base"));
        assert!(code.contains("extra"));
    }

    #[test]
    fn generates_focus_prefix() {
        let input = quote::quote! {
            "suite" {
                focus "important" {
                    let x = 1;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("__FOCUS__"));
    }

    #[test]
    fn generates_teardown_with_catch_unwind() {
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
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("catch_unwind"),
            "expected catch_unwind in: {code}"
        );
        assert!(
            code.contains("resume_unwind"),
            "expected resume_unwind in: {code}"
        );
        assert!(code.contains("cleanup"), "expected cleanup in: {code}");
    }

    #[test]
    fn generates_no_catch_unwind_without_teardown() {
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
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(!code.contains("catch_unwind"));
    }

    #[test]
    fn generates_async_test() {
        let input = quote::quote! {
            "suite" {
                tokio;

                "test" {
                    let x = 1;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("async fn"));
        assert!(code.contains("tokio :: test"));
    }

    #[test]
    fn generates_async_with_teardown() {
        let input = quote::quote! {
            "suite" {
                tokio;

                teardown {
                    cleanup();
                }

                "test" {
                    let x = 1;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("async fn"));
        // Async teardown uses async block, not catch_unwind
        assert!(!code.contains("catch_unwind"));
        assert!(code.contains("__behave_test_result"));
    }

    #[test]
    fn generates_teardown_without_setup() {
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
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("catch_unwind"));
        assert!(code.contains("cleanup"));
    }

    #[test]
    fn generates_nested_modules() {
        let input = quote::quote! {
            "a" {
                "b" {
                    "c" {
                        let x = 1;
                    }
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("mod a"));
        assert!(code.contains("mod b"));
    }

    #[test]
    fn generates_each_tests() {
        let input = quote::quote! {
            "math" {
                "addition" {
                    each [
                        (2, 2, 4),
                        (0, 0, 0),
                    ] |a, b, expected| {
                        let _ = a + b == expected;
                    }
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("mod addition"),
            "expected mod addition in: {code}"
        );
        assert!(code.contains("fn case_0"), "expected fn case_0 in: {code}");
        assert!(code.contains("fn case_1"), "expected fn case_1 in: {code}");
    }

    #[test]
    fn generates_each_single_param() {
        let input = quote::quote! {
            "values" {
                each [1, 2, 3] |n| {
                    let _ = n;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("fn case_0"), "expected fn case_0 in: {code}");
        assert!(code.contains("fn case_1"), "expected fn case_1 in: {code}");
        assert!(code.contains("fn case_2"), "expected fn case_2 in: {code}");
    }

    #[test]
    fn generates_each_with_focus() {
        let input = quote::quote! {
            focus "cases" {
                each [1, 2] |n| {
                    let _ = n;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__FOCUS__"),
            "expected __FOCUS__ prefix in: {code}"
        );
    }

    #[test]
    fn generates_sync_timeout() {
        let input = quote::quote! {
            "suite" {
                timeout 5000;

                "test" {
                    let x = 1;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("recv_timeout"),
            "expected recv_timeout in: {code}"
        );
        assert!(code.contains("5000"), "expected timeout value in: {code}");
    }

    #[test]
    fn generates_async_timeout() {
        let input = quote::quote! {
            "suite" {
                tokio;
                timeout 1000;

                "test" {
                    let x = 1;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("tokio :: time :: timeout"),
            "expected tokio::time::timeout in: {code}"
        );
        assert!(code.contains("1000"), "expected timeout value in: {code}");
    }

    #[test]
    fn generates_sync_timeout_with_teardown() {
        let input = quote::quote! {
            "suite" {
                timeout 3000;

                teardown {
                    cleanup();
                }

                "test" {
                    let x = 1;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("recv_timeout"),
            "expected recv_timeout in: {code}"
        );
        assert!(code.contains("cleanup"), "expected teardown in: {code}");
    }

    #[test]
    fn generates_timeout_inherits_to_children() {
        let input = quote::quote! {
            "outer" {
                timeout 2000;

                "inner" {
                    "test" {
                        let x = 1;
                    }
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("recv_timeout"),
            "expected inherited timeout in: {code}"
        );
        assert!(code.contains("2000"), "expected timeout value in: {code}");
    }

    #[test]
    fn generates_each_named_cases() {
        let input = quote::quote! {
            "http" {
                "status" {
                    each [
                        ("ok", 200, true),
                        ("not_found", 404, false),
                    ] |name, code, success| {
                        let _ = (code, success);
                    }
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("fn ok"), "expected fn ok in: {code}");
        assert!(
            code.contains("fn not_found"),
            "expected fn not_found in: {code}"
        );
        assert!(
            !code.contains("fn case_0"),
            "should not have case_0 with named cases: {code}"
        );
    }

    #[test]
    fn generates_each_named_single_value() {
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
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("fn small"), "expected fn small in: {code}");
        assert!(code.contains("fn large"), "expected fn large in: {code}");
    }

    #[test]
    fn generates_each_with_inherited_setup() {
        let input = quote::quote! {
            "suite" {
                setup {
                    let base = 10;
                }

                "offset" {
                    each [
                        (1, 11),
                        (5, 15),
                    ] |n, expected| {
                        let _ = base + n == expected;
                    }
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("base"), "expected setup binding in: {code}");
        assert!(code.contains("fn case_0"), "expected fn case_0 in: {code}");
    }

    // --- tag codegen tests ---

    #[test]
    fn generates_tag_prefix_on_test() {
        let input = quote::quote! {
            "my test" tag "slow" {
                let x = 1;
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__TAG_slow__"),
            "expected __TAG_slow__ in: {code}"
        );
    }

    #[test]
    fn generates_multiple_tag_prefixes() {
        let input = quote::quote! {
            "my test" tag "slow", "integration" {
                let x = 1;
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__TAG_slow__"),
            "expected __TAG_slow__ in: {code}"
        );
        assert!(
            code.contains("__TAG_integration__"),
            "expected __TAG_integration__ in: {code}"
        );
    }

    #[test]
    fn generates_focus_and_tag_combo() {
        let input = quote::quote! {
            focus "important" tag "critical" {
                let x = 1;
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__FOCUS____TAG_critical__"),
            "expected FOCUS then TAG prefix in: {code}"
        );
    }

    #[test]
    fn generates_tag_on_each_module() {
        let input = quote::quote! {
            "cases" tag "unit" {
                each [1, 2] |n| {
                    let _ = n;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__TAG_unit__"),
            "expected __TAG_unit__ in: {code}"
        );
    }

    #[test]
    fn generates_tag_on_matrix_module() {
        let input = quote::quote! {
            "combos" tag "slow" {
                matrix [1, 2] x [3, 4] |a, b| {
                    let _ = a + b;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__TAG_slow__"),
            "expected __TAG_slow__ in: {code}"
        );
    }

    // --- xfail codegen tests ---

    #[test]
    fn generates_xfail_test() {
        let input = quote::quote! {
            xfail "expected failure" {
                let x = 1;
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__behave_xfail"),
            "expected xfail wrapper in: {code}"
        );
        assert!(
            code.contains("expected test to fail"),
            "expected xfail error message in: {code}"
        );
    }

    #[test]
    fn generates_xfail_each() {
        let input = quote::quote! {
            xfail "broken" {
                each [1, 2] |n| {
                    let _ = n;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__behave_xfail"),
            "expected xfail wrapper in each: {code}"
        );
    }

    #[test]
    fn generates_xfail_async() {
        let input = quote::quote! {
            "suite" {
                tokio;

                xfail "async expected failure" {
                    let x = 1;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(code.contains("async fn"), "expected async fn in: {code}");
        assert!(
            code.contains("__behave_xfail"),
            "expected xfail wrapper in async: {code}"
        );
    }

    #[test]
    fn generates_non_xfail_has_no_wrapper() {
        let input = quote::quote! {
            "normal test" {
                let x = 1;
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            !code.contains("__behave_xfail"),
            "normal test should not have xfail wrapper: {code}"
        );
    }

    // --- matrix codegen tests ---

    #[test]
    fn generates_matrix_tests() {
        let input = quote::quote! {
            "combos" {
                matrix [1, 2] x [10, 20] |a, b| {
                    let _ = a + b;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("mod combos"),
            "expected mod combos in: {code}"
        );
        assert!(
            code.contains("fn case_0_0"),
            "expected fn case_0_0 in: {code}"
        );
        assert!(
            code.contains("fn case_0_1"),
            "expected fn case_0_1 in: {code}"
        );
        assert!(
            code.contains("fn case_1_0"),
            "expected fn case_1_0 in: {code}"
        );
        assert!(
            code.contains("fn case_1_1"),
            "expected fn case_1_1 in: {code}"
        );
    }

    #[test]
    fn generates_matrix_with_focus() {
        let input = quote::quote! {
            focus "combos" {
                matrix [1, 2] x [3, 4] |a, b| {
                    let _ = a + b;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__FOCUS__"),
            "expected __FOCUS__ prefix in: {code}"
        );
    }

    #[test]
    fn generates_matrix_three_dimensions() {
        let input = quote::quote! {
            "3d" {
                matrix [1, 2] x [10, 20] x [true] |a, b, c| {
                    let _ = (a, b, c);
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        // 2 * 2 * 1 = 4 cases
        assert!(
            code.contains("fn case_0_0_0"),
            "expected fn case_0_0_0 in: {code}"
        );
        assert!(
            code.contains("fn case_1_1_0"),
            "expected fn case_1_1_0 in: {code}"
        );
    }

    #[test]
    fn generates_matrix_xfail() {
        let input = quote::quote! {
            xfail "broken combos" {
                matrix [1, 2] x [10, 20] |a, b| {
                    let _ = a + b;
                }
            }
        };
        let result = parse_and_generate(input);
        assert!(result.is_ok());
        let code = result.map(|t| t.to_string()).unwrap_or_default();
        assert!(
            code.contains("__behave_xfail"),
            "expected xfail wrapper in matrix: {code}"
        );
    }
}
