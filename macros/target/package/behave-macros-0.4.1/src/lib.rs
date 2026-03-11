//! Proc macros for the behave BDD testing framework.
//!
//! This crate is an implementation detail of [`behave`]. Do not depend on it
//! directly - use `behave` instead.
#![allow(unreachable_pub)]

mod codegen;
mod parse;
mod slug;

/// Defines BDD-style test suites using a zero-keyword DSL.
///
/// # Examples
///
/// ```rust,ignore
/// use behave::prelude::*;
///
/// behave! {
///     "addition" {
///         "adds two numbers" {
///             expect!(1 + 1).to_equal(&2)?;
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn behave(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as parse::BehaveInput);
    match codegen::generate(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
