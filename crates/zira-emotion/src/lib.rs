//! zira-emotion — streaming emotion-tag parser + prosody tables.

use zira_proto::Emotion;
pub use zira_proto::Segment;

/// Speech-synthesis prosody multipliers applied to a TTS engine's baseline.
///
/// All three fields are dimensionless ratios relative to the engine default.
/// `rate = 1.0` means normal speed, `pitch = 1.0` means normal pitch, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prosody {
    pub rate: f32,
    pub pitch: f32,
    pub volume: f32,
}

/// Return the prosody multipliers for `e`.
///
/// The function is total: every `Emotion` variant maps to a `Prosody`.
/// Invariant: all three fields lie within `0.5..=2.0`.
pub fn prosody(e: Emotion) -> Prosody {
    match e {
        Emotion::Neutral   => Prosody { rate: 1.00, pitch: 1.00, volume: 1.00 },
        Emotion::Happy     => Prosody { rate: 1.15, pitch: 1.10, volume: 1.10 },
        Emotion::Sad       => Prosody { rate: 0.85, pitch: 0.90, volume: 0.85 },
        Emotion::Angry     => Prosody { rate: 1.20, pitch: 1.15, volume: 1.30 },
        Emotion::Excited   => Prosody { rate: 1.30, pitch: 1.20, volume: 1.20 },
        Emotion::Calm      => Prosody { rate: 0.90, pitch: 0.95, volume: 1.00 },
        Emotion::Curious   => Prosody { rate: 1.05, pitch: 1.05, volume: 1.00 },
        Emotion::Concerned => Prosody { rate: 0.95, pitch: 0.95, volume: 0.95 },
        Emotion::Playful   => Prosody { rate: 1.10, pitch: 1.10, volume: 1.05 },
        Emotion::Tired     => Prosody { rate: 0.75, pitch: 0.85, volume: 0.80 },
    }
}

/// Split `s` into ordered `Segment`s at each `[emotion:...]` marker.
///
/// The emotion in effect for a span is the tag that opened it (Neutral for
/// text before the first marker). Empty-text spans are dropped.
/// Concatenating every segment's text equals `strip_tags(s)`.
pub fn segment(s: &str) -> Vec<Segment> {
    const PREFIX: &str = "[emotion:";
    let mut result = Vec::new();
    let mut remaining = s;
    let mut current_emotion = Emotion::Neutral;

    while let Some(marker_start) = remaining.find(PREFIX) {
        let text_before = &remaining[..marker_start];
        if !text_before.is_empty() {
            result.push(Segment { emotion: current_emotion, text: text_before.to_string() });
        }
        let after_prefix = &remaining[marker_start + PREFIX.len()..];
        if let Some(close) = after_prefix.find(']') {
            current_emotion = Emotion::from_tag(&after_prefix[..close]);
            remaining = &after_prefix[close + 1..];
        } else {
            // Malformed marker with no closing ']' — treat remainder as text.
            remaining = &remaining[marker_start..];
            break;
        }
    }

    if !remaining.is_empty() {
        result.push(Segment { emotion: current_emotion, text: remaining.to_string() });
    }

    result
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
