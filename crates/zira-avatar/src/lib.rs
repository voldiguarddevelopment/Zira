//! zira-avatar — VRM avatar renderer.

use zira_proto::Emotion;

/// Mouth-shape variants for lip-sync.
///
/// Stub: derives are correct; Default and as_label are intentionally wrong so
/// the frozen RED tests fail until the real implementation lands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Viseme {
    Sil,
    A,
    I,
    U,
    E,
    O,
}

impl Default for Viseme {
    /// Stub — returns A so that `c1_default_is_sil` fails in RED.
    fn default() -> Self {
        Viseme::A
    }
}

impl Viseme {
    /// Stub — returns the same string for every variant so that every
    /// `c2_*_label` test fails and the distinct-labels test also fails in RED.
    pub fn as_label(self) -> &'static str {
        "x"
    }
}

/// Blendshape weight preset for a single VRM expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionPreset {
    pub joy: f32,
    pub sorrow: f32,
    pub anger: f32,
    pub surprise: f32,
    pub fun: f32,
}

impl ExpressionPreset {
    pub fn neutral() -> Self {
        Self { joy: 0.0, sorrow: 0.0, anger: 0.0, surprise: 0.0, fun: 0.0 }
    }

    pub fn clamped(&self) -> ExpressionPreset {
        Self {
            joy: self.joy.clamp(0.0, 1.0),
            sorrow: self.sorrow.clamp(0.0, 1.0),
            anger: self.anger.clamp(0.0, 1.0),
            surprise: self.surprise.clamp(0.0, 1.0),
            fun: self.fun.clamp(0.0, 1.0),
        }
    }
}

/// Map an `Emotion` to its corresponding blendshape `ExpressionPreset`.
///
/// Total over all ten `Emotion` variants. Every returned preset is already
/// within `[0.0, 1.0]` on every weight, so `preset == preset.clamped()`.
pub fn expression_for(emotion: Emotion) -> ExpressionPreset {
    match emotion {
        Emotion::Neutral => ExpressionPreset { joy: 0.0, sorrow: 0.0, anger: 0.0, surprise: 0.0, fun: 0.0 },
        Emotion::Happy => ExpressionPreset { joy: 0.8, sorrow: 0.0, anger: 0.0, surprise: 0.0, fun: 0.2 },
        Emotion::Sad => ExpressionPreset { joy: 0.0, sorrow: 0.8, anger: 0.0, surprise: 0.0, fun: 0.0 },
        Emotion::Angry => ExpressionPreset { joy: 0.0, sorrow: 0.0, anger: 0.9, surprise: 0.0, fun: 0.0 },
        Emotion::Excited => ExpressionPreset { joy: 0.6, sorrow: 0.0, anger: 0.0, surprise: 0.6, fun: 0.4 },
        Emotion::Calm => ExpressionPreset { joy: 0.2, sorrow: 0.0, anger: 0.0, surprise: 0.0, fun: 0.0 },
        Emotion::Curious => ExpressionPreset { joy: 0.1, sorrow: 0.0, anger: 0.0, surprise: 0.5, fun: 0.2 },
        Emotion::Concerned => ExpressionPreset { joy: 0.0, sorrow: 0.4, anger: 0.1, surprise: 0.2, fun: 0.0 },
        Emotion::Playful => ExpressionPreset { joy: 0.5, sorrow: 0.0, anger: 0.0, surprise: 0.2, fun: 0.8 },
        Emotion::Tired => ExpressionPreset { joy: 0.0, sorrow: 0.3, anger: 0.0, surprise: 0.0, fun: 0.0 },
    }
}
