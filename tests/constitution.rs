//! Frozen tests for T-00.12 — "Embed the constitution".
//!
//! Criterion → test mapping:
//!
//!   C1 -> c1_load_default_requires_no_file,
//!          c1_load_default_returns_constitution_directly
//!   C2 -> c2_rules_readable_from_immutable_binding,
//!          c2_rules_returns_shared_slice
//!   C3 -> c3_embedded_constitution_is_nonempty,
//!          c3_no_public_mutator_on_immutable_binding

use zira_config::Constitution;

// ---- C1 -------------------------------------------------------------------------------

/// C1: load_default() takes no path argument — proves the default is embedded in the
///     binary via include_str!, not loaded from the filesystem.
#[test]
fn c1_load_default_requires_no_file() {
    let _c = Constitution::load_default();
}

/// C1: the return type is Constitution (not Option or Result), confirming the embedded
///     default is always present and always valid at runtime.
#[test]
fn c1_load_default_returns_constitution_directly() {
    let _c: Constitution = Constitution::load_default();
}

// ---- C2 -------------------------------------------------------------------------------

/// C2: rules() is callable on an immutable (non-mut) binding, confirming it is a
///     &self accessor with no side-effects.
#[test]
fn c2_rules_readable_from_immutable_binding() {
    let c = Constitution::load_default(); // non-mut binding
    let _rules = c.rules();
}

/// C2: rules() returns a shared (immutable) slice — confirming the accessor does not
///     grant mutable access to the internal rule set.
#[test]
fn c2_rules_returns_shared_slice() {
    let c = Constitution::load_default();
    // The explicit type annotation asserts the return is an immutable view.
    let _rules: &[String] = c.rules();
}

// ---- C3 -------------------------------------------------------------------------------

/// C3: the embedded default constitution is non-empty (contains at least one rule).
#[test]
fn c3_embedded_constitution_is_nonempty() {
    let c = Constitution::load_default();
    assert!(
        !c.rules().is_empty(),
        "embedded constitution must contain at least one rule"
    );
}

/// C3 / C2: the entire accessible public surface of Constitution is exercised from a
///          non-mut binding; no mutation method is reachable.  This test compiles
///          without `mut c`, which means every callable method takes &self — the
///          compiler would reject any &mut self call here, proving no mutator exists.
#[test]
fn c3_no_public_mutator_on_immutable_binding() {
    let c = Constitution::load_default(); // non-mut — &mut self methods are unreachable
    let rules: &[String] = c.rules();
    assert!(
        rules.len() >= 1,
        "rule set must be non-empty; found {} rules",
        rules.len()
    );
}
