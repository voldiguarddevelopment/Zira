//! zira-emotion — streaming emotion-tag parser + prosody tables.

use zira_proto::Emotion;
pub use zira_proto::Segment;

/// Split `s` into ordered `Segment`s at each `[emotion:...]` marker.
///
/// The emotion in effect for a span is the tag that opened it (Neutral for
/// text before the first marker). Empty-text spans are dropped.
/// Concatenating every segment's text equals `strip_tags(s)`.
pub fn segment(_s: &str) -> Vec<Segment> {
    todo!("T-01.03: not yet implemented")
}

/// Remove every `[emotion:NAME]` marker from `s`, returning the remaining text unchanged.
pub fn strip_tags(s: &str) -> String {
    const PREFIX: &str = "[emotion:";
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;
    while let Some(start) = remaining.find(PREFIX) {
        result.push_str(&remaining[..start]);
        let after_prefix = &remaining[start + PREFIX.len()..];
        if let Some(close) = after_prefix.find(']') {
            remaining = &after_prefix[close + 1..];
        } else {
            result.push_str(&remaining[start..]);
            return result;
        }
    }
    result.push_str(remaining);
    result
}

/// Parse a leading `[emotion:NAME]` marker from `s`.
///
/// Returns the resolved `Emotion` and the remaining text with leading
/// whitespace trimmed. If no marker is present at the start, returns
/// `(Emotion::Neutral, s)` with the original slice unchanged.
pub fn parse_tag(s: &str) -> (Emotion, &str) {
    const PREFIX: &str = "[emotion:";
    if let Some(after_prefix) = s.strip_prefix(PREFIX) {
        if let Some(close) = after_prefix.find(']') {
            let name = &after_prefix[..close];
            let emotion = Emotion::from_tag(name);
            let rest = after_prefix[close + 1..].trim_start();
            return (emotion, rest);
        }
    }
    (Emotion::Neutral, s)
}
