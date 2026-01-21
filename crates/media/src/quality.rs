#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLayer {
    Low,
    Medium,
    High,
}

impl QualityLayer {
    pub fn from_bitrate(bitrate: u64) -> Self {
        if bitrate < 150_000 {
            QualityLayer::Low
        } else if bitrate < 750_000 {
            QualityLayer::Medium
        } else {
            QualityLayer::High
        }
    }

    pub fn to_rid(&self) -> &'static str {
        match self {
            QualityLayer::Low => "l",
            QualityLayer::Medium => "m",
            QualityLayer::High => "h",
        }
    }
}
