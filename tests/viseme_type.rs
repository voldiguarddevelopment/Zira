//! Frozen tests for T-03.03 — "Define the viseme vocabulary".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_variants_exist, c1_derives_debug, c1_derives_clone,
//!          c1_derives_copy, c1_derives_partial_eq, c1_default_is_sil
//!   C2 -> c2_sil_label, c2_a_label, c2_i_label, c2_u_label, c2_e_label,
//!          c2_o_label, c2_all_labels_nonempty_and_distinct

use zira_avatar::Viseme;

// ---- C1 — enum variants + derives + Default ----------------------------------------

#[test]
fn c1_variants_exist() {
    // Every variant must be nameable and matchable.
    let variants = [Viseme::Sil, Viseme::A, Viseme::I, Viseme::U, Viseme::E, Viseme::O];
    assert_eq!(variants.len(), 6, "there must be exactly six viseme variants");
}

#[test]
fn c1_derives_debug() {
    let s = format!("{:?}", Viseme::Sil);
    assert!(s.contains("Sil"), "Debug output for Viseme::Sil must contain \"Sil\"");
    let s = format!("{:?}", Viseme::A);
    assert!(s.contains('A'), "Debug output for Viseme::A must contain 'A'");
}

#[test]
fn c1_derives_clone() {
    let original = Viseme::E;
    let cloned = original.clone();
    assert_eq!(original, cloned, "Clone must produce a value equal to the original");
}

#[test]
fn c1_derives_copy() {
    let a = Viseme::O;
    let b = a; // Copy: a is not moved
    assert_eq!(a, b, "Copy must allow reuse of the original binding after assignment");
}

#[test]
fn c1_derives_partial_eq() {
    assert_eq!(Viseme::Sil, Viseme::Sil, "Sil must equal itself via PartialEq");
    assert_ne!(Viseme::Sil, Viseme::A, "distinct variants must not be equal");
    assert_ne!(Viseme::A, Viseme::I, "A and I must not be equal");
}

#[test]
fn c1_default_is_sil() {
    let d: Viseme = Default::default();
    assert_eq!(d, Viseme::Sil, "Default must return Viseme::Sil (the rest/silence shape)");
}

// ---- C2 — as_label returns the lowercase shape name -------------------------------

#[test]
fn c2_sil_label() {
    assert_eq!(Viseme::Sil.as_label(), "sil", "Sil must label as \"sil\"");
}

#[test]
fn c2_a_label() {
    assert_eq!(Viseme::A.as_label(), "a", "A must label as \"a\"");
}

#[test]
fn c2_i_label() {
    assert_eq!(Viseme::I.as_label(), "i", "I must label as \"i\"");
}

#[test]
fn c2_u_label() {
    assert_eq!(Viseme::U.as_label(), "u", "U must label as \"u\"");
}

#[test]
fn c2_e_label() {
    assert_eq!(Viseme::E.as_label(), "e", "E must label as \"e\"");
}

#[test]
fn c2_o_label() {
    assert_eq!(Viseme::O.as_label(), "o", "O must label as \"o\"");
}

#[test]
fn c2_all_labels_nonempty_and_distinct() {
    let pairs = [
        (Viseme::Sil, Viseme::Sil.as_label()),
        (Viseme::A, Viseme::A.as_label()),
        (Viseme::I, Viseme::I.as_label()),
        (Viseme::U, Viseme::U.as_label()),
        (Viseme::E, Viseme::E.as_label()),
        (Viseme::O, Viseme::O.as_label()),
    ];
    for (variant, label) in pairs {
        assert!(!label.is_empty(), "{variant:?}.as_label() must not be empty");
    }
    // All labels must be distinct — check every pair.
    for i in 0..pairs.len() {
        for j in (i + 1)..pairs.len() {
            assert_ne!(
                pairs[i].1,
                pairs[j].1,
                "{:?} and {:?} must have distinct labels (got {:?})",
                pairs[i].0,
                pairs[j].0,
                pairs[i].1,
            );
        }
    }
}
