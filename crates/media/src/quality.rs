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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_layer_selection() {
        assert_eq!(QualityLayer::from_bitrate(100_000), QualityLayer::Low);
        assert_eq!(QualityLayer::from_bitrate(500_000), QualityLayer::Medium);
        assert_eq!(QualityLayer::from_bitrate(1_000_000), QualityLayer::High);
    }

    #[test]
    fn test_quality_layer_rid() {
        assert_eq!(QualityLayer::Low.to_rid(), "l");
        assert_eq!(QualityLayer::Medium.to_rid(), "m");
        assert_eq!(QualityLayer::High.to_rid(), "h");
    }
}
