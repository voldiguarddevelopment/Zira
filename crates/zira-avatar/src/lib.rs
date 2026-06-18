//! zira-avatar — VRM avatar renderer.

/// Blendshape weight preset for a single VRM expression.
///
/// Stub: fields and derives exist; method bodies are intentionally wrong so
/// the frozen RED tests fail until the real implementation lands.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionPreset {
    pub joy: f32,
    pub sorrow: f32,
    pub anger: f32,
    pub surprise: f32,
    pub fun: f32,
}

impl ExpressionPreset {
    /// Stub — returns all-ones so `c1_neutral_all_zeros` fails in RED.
    pub fn neutral() -> Self {
        Self { joy: 1.0, sorrow: 1.0, anger: 1.0, surprise: 1.0, fun: 1.0 }
    }

    /// Stub — returns a plain clone without clamping so `c2_clamped_*` fail in RED.
    pub fn clamped(&self) -> ExpressionPreset {
        self.clone()
    }
}
