//! zira-emotion — streaming emotion-tag parser + prosody tables.

use zira_proto::Emotion;

/// Parse a leading `[emotion:NAME]` marker from `s`.
///
/// Returns the resolved `Emotion` and the remaining text with leading
/// whitespace trimmed. If no marker is present at the start, returns
/// `(Emotion::Neutral, s)` with the original slice unchanged.
pub fn parse_tag(_s: &str) -> (Emotion, &str) {
    todo!("T-01.01 green phase: implement parse_tag")
}
