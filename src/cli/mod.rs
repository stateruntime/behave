//! CLI runner for `cargo-behave`.
//!
//! Provides a hierarchical test output renderer that replaces the default
//! `cargo test` output with a tree-structured, colored display.
//!
//! This module is only available when the `cli` feature is enabled.

pub mod config;
pub mod context;
pub mod error;
pub mod filter;
pub mod history;
pub mod output;
pub mod parser;
pub mod render;
pub mod runner;
pub mod tree;
pub mod watch;
