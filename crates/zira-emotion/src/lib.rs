//! zira-emotion — streaming emotion-tag parser + prosody tables.

use zira_proto::Emotion;

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
