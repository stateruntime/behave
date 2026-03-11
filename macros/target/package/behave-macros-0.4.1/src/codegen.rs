//! Code generation from the parsed behave DSL AST.
//!
//! Transforms [`BehaveNode`] trees into Rust `#[test]` functions
//! organized inside nested modules.

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

use crate::parse::{BehaveInput, BehaveNode, EachNode, GroupNode, PendingNode, TestNode};
use crate::slug::{is_rust_keyword, slugify};

/// Creates an identifier, using a raw identifier for Rust keywords.
fn make_ident(name: &str) -> Ident {
    if is_rust_keyword(name) {
        Ident::new_raw(name, Span::call_site())
    } else {
        Ident::new(name, Span::call_site())
    }
}

/// Context threaded through code generation for inherited state.
struct GenContext<'a> {
    setups: Vec<&'a TokenStream>,
    teardowns: Vec<&'a TokenStream>,
    is_async: bool,
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
    }
}

fn generate_group(group: &GroupNode, ctx: &GenContext<'_>) -> syn::Result<TokenStream> {
    let slug = slugify(&group.label);
    let mod_name = if group.focused {
        format_ident!("__FOCUS__{slug}")
    } else {
        make_ident(&slug)
    };

    let mut child_ctx = GenContext {
        setups: ctx.setups.clone(),
        teardowns: ctx.teardowns.clone(),
        is_async: ctx.is_async || group.async_runtime,
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
    let fn_name = if test.focused {
        format_ident!("__FOCUS__{}", slugify(&test.label))
    } else {
        make_ident(&slugify(&test.label))
    };

    let empty = TokenStream::new();
    generate_test_fn(fn_name, &empty, &test.body, ctx)
}

/// Shared helper that emits a single `#[test]` (or `#[tokio::test]`) function.
///
/// `extra_setup` is prepended after inherited setups — used by `each` to bind
/// case parameters. Regular tests pass an empty stream.
fn generate_test_fn(
    fn_name: Ident,
    extra_setup: &TokenStream,
    body: &TokenStream,
    ctx: &GenContext<'_>,
) -> TokenStream {
    let setup_tokens: TokenStream = ctx.setups.iter().map(|s| quote! { #s }).collect();
    let has_teardown = !ctx.teardowns.is_empty();

    // Teardowns run in reverse order (innermost first, like destructors)
    let teardown_tokens: TokenStream = ctx.teardowns.iter().rev().map(|t| quote! { #t }).collect();

    match (ctx.is_async, has_teardown) {
        (false, false) => quote! {
            #[test]
            fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
                #setup_tokens
                #extra_setup
                #body
                Ok(())
            }
        },
        (false, true) => quote! {
            #[test]
            fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
                #setup_tokens
                #extra_setup
                let __behave_test_result = std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(|| -> Result<(), Box<dyn std::error::Error>> {
                        #body
                        Ok(())
                    })
                );
                #teardown_tokens
                match __behave_test_result {
                    Ok(result) => result,
                    Err(panic) => std::panic::resume_unwind(panic),
                }
            }
        },
        (true, false) => quote! {
            #[tokio::test]
            async fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
                #setup_tokens
                #extra_setup
                #body
                Ok(())
            }
        },
        (true, true) => quote! {
            #[tokio::test]
            async fn #fn_name() -> Result<(), Box<dyn std::error::Error>> {
                #setup_tokens
                #extra_setup
                let __behave_test_result: Result<(), Box<dyn std::error::Error>> = async {
                    #body
                    Ok(())
                }.await;
                #teardown_tokens
                __behave_test_result
            }
        },
    }
}

fn generate_each(each: &EachNode, ctx: &GenContext<'_>) -> TokenStream {
    let slug = slugify(&each.label);
    let mod_name = if each.focused {
        format_ident!("__FOCUS__{slug}")
    } else {
        make_ident(&slug)
    };

    let case_fns: TokenStream = each
        .cases
        .iter()
        .enumerate()
        .map(|(i, case_expr)| {
            let fn_name = format_ident!("case_{i}");
            let params = &each.params;

            // Single param: `let p = expr;`  Multi param: `let (p1, p2) = expr;`
            let binding = if params.len() == 1 {
                let p = &params[0];
                quote! { let #p = #case_expr; }
            } else {
                quote! { let (#(#params),*) = #case_expr; }
            };

            generate_test_fn(fn_name, &binding, &each.body, ctx)
        })
        .collect();

    quote! {
        mod #mod_name {
            use super::*;
            #case_fns
        }
    }
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
}
