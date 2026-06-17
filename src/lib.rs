//! Root test-host crate for the Zira workspace.
//!
//! The repo root is BOTH a `[package]` and (after GREEN) a `[workspace]`, so that
//! `cargo test` at the root compiles and runs the repo-root `tests/`. This crate carries
//! no logic — every member crate lives under `crates/`.
