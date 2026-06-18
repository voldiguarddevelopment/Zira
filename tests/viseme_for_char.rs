//! Frozen tests for T-03.04 — "Select the viseme".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_lowercase_a_maps_to_a, c1_lowercase_e_maps_to_e,
//!          c1_lowercase_i_maps_to_i, c1_lowercase_o_maps_to_o,
//!          c1_lowercase_u_maps_to_u, c1_uppercase_a_maps_to_a,
//!          c1_uppercase_e_maps_to_e, c1_uppercase_i_maps_to_i,
//!          c1_uppercase_o_maps_to_o, c1_uppercase_u_maps_to_u
//!   C2 -> c2_consonant_maps_to_sil, c2_digit_maps_to_sil,
//!          c2_whitespace_maps_to_sil

use zira_avatar::{viseme_for_char, Viseme};

// ---- C1 — vowel characters map to their named mouth shape, case-insensitive --------

#[test]
fn c1_lowercase_a_maps_to_a() {
    assert_eq!(viseme_for_char('a'), Viseme::A, "'a' must map to Viseme::A");
}

#[test]
fn c1_lowercase_e_maps_to_e() {
    assert_eq!(viseme_for_char('e'), Viseme::E, "'e' must map to Viseme::E");
}

#[test]
fn c1_lowercase_i_maps_to_i() {
    assert_eq!(viseme_for_char('i'), Viseme::I, "'i' must map to Viseme::I");
}

#[test]
fn c1_lowercase_o_maps_to_o() {
    assert_eq!(viseme_for_char('o'), Viseme::O, "'o' must map to Viseme::O");
}

#[test]
fn c1_lowercase_u_maps_to_u() {
    assert_eq!(viseme_for_char('u'), Viseme::U, "'u' must map to Viseme::U");
}

#[test]
fn c1_uppercase_a_maps_to_a() {
    assert_eq!(viseme_for_char('A'), Viseme::A, "'A' must map to Viseme::A (case-insensitive)");
}

#[test]
fn c1_uppercase_e_maps_to_e() {
    assert_eq!(viseme_for_char('E'), Viseme::E, "'E' must map to Viseme::E (case-insensitive)");
}

#[test]
fn c1_uppercase_i_maps_to_i() {
    assert_eq!(viseme_for_char('I'), Viseme::I, "'I' must map to Viseme::I (case-insensitive)");
}

#[test]
fn c1_uppercase_o_maps_to_o() {
    assert_eq!(viseme_for_char('O'), Viseme::O, "'O' must map to Viseme::O (case-insensitive)");
}

#[test]
fn c1_uppercase_u_maps_to_u() {
    assert_eq!(viseme_for_char('U'), Viseme::U, "'U' must map to Viseme::U (case-insensitive)");
}

// ---- C2 — non-vowel characters (consonants, digits, whitespace) rest at Sil --------

#[test]
fn c2_consonant_maps_to_sil() {
    assert_eq!(viseme_for_char('b'), Viseme::Sil, "consonant 'b' must map to Viseme::Sil");
    assert_eq!(viseme_for_char('z'), Viseme::Sil, "consonant 'z' must map to Viseme::Sil");
}

#[test]
fn c2_digit_maps_to_sil() {
    assert_eq!(viseme_for_char('0'), Viseme::Sil, "digit '0' must map to Viseme::Sil");
    assert_eq!(viseme_for_char('9'), Viseme::Sil, "digit '9' must map to Viseme::Sil");
}

#[test]
fn c2_whitespace_maps_to_sil() {
    assert_eq!(viseme_for_char(' '), Viseme::Sil, "space must map to Viseme::Sil");
    assert_eq!(viseme_for_char('\n'), Viseme::Sil, "newline must map to Viseme::Sil");
}
