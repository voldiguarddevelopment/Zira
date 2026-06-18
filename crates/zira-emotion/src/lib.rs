//! zira-emotion — streaming emotion-tag parser + prosody tables.

use zira_proto::Emotion;

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
