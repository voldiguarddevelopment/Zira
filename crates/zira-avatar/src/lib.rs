//! zira-avatar — VRM avatar renderer.

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
