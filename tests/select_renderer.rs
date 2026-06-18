//! Frozen tests for T-03.09 — "Select the renderer".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_renderer_kind_derives_debug, c1_renderer_kind_derives_clone,
//!          c1_renderer_kind_derives_copy, c1_renderer_kind_derives_partial_eq,
//!          c1_select_renderer_none_gives_fallback2d
//!   C2 -> c2_select_renderer_some_nonempty_gives_vrm
//!   C3 -> c3_select_renderer_some_empty_gives_fallback2d

use zira_avatar::{select_renderer, RendererKind};
use zira_config::AvatarConfig;

// ---- C1 — RendererKind enum shape + derives + None → Fallback2d -------------------

#[test]
fn c1_renderer_kind_derives_debug() {
    let kind = RendererKind::Fallback2d;
    let out = format!("{kind:?}");
    assert!(out.contains("Fallback2d"), "Debug output must name the variant");
}

#[test]
fn c1_renderer_kind_derives_clone() {
    let a = RendererKind::Vrm;
    let b = a.clone();
    assert_eq!(a, b, "Clone must produce a value equal to the original");
}

#[test]
fn c1_renderer_kind_derives_copy() {
    let a = RendererKind::Vrm;
    let b = a; // if Copy is not derived this is a move and the next line is a use-after-move
    let _ = a; // must still be valid
    assert_eq!(a, b, "Copy must allow both bindings to remain live");
}

#[test]
fn c1_renderer_kind_derives_partial_eq() {
    assert_eq!(RendererKind::Vrm, RendererKind::Vrm, "Vrm == Vrm");
    assert_eq!(RendererKind::Fallback2d, RendererKind::Fallback2d, "Fallback2d == Fallback2d");
    assert_ne!(RendererKind::Vrm, RendererKind::Fallback2d, "Vrm != Fallback2d");
}

#[test]
fn c1_select_renderer_none_gives_fallback2d() {
    let cfg = AvatarConfig { vrm_path: None };
    assert_eq!(
        select_renderer(&cfg),
        RendererKind::Fallback2d,
        "None vrm_path must select the Fallback2d renderer"
    );
}

// ---- C2 — Some(non-empty path) → Vrm -------------------------------------------

#[test]
fn c2_select_renderer_some_nonempty_gives_vrm() {
    let cfg = AvatarConfig { vrm_path: Some("model.vrm".to_string()) };
    assert_eq!(
        select_renderer(&cfg),
        RendererKind::Vrm,
        "Some(non-empty path) must select the Vrm renderer"
    );
}

// ---- C3 — Some("") → Fallback2d (empty path treated as absent) ------------------

#[test]
fn c3_select_renderer_some_empty_gives_fallback2d() {
    let cfg = AvatarConfig { vrm_path: Some(String::new()) };
    assert_eq!(
        select_renderer(&cfg),
        RendererKind::Fallback2d,
        "Some(\"\") must be treated as absent and yield the Fallback2d renderer"
    );
}
