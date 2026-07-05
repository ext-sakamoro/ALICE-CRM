//! rfm.

// ---------------------------------------------------------------------------
// RFM segmentation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RfmSegment {
    Champion,
    Loyal,
    Potential,
    AtRisk,
    Lost,
}

#[derive(Debug, Clone, Copy)]
pub struct RfmScore {
    pub recency: u8,
    pub frequency: u8,
    pub monetary: u8,
}

impl RfmScore {
    #[must_use]
    pub const fn total(self) -> u16 {
        self.recency as u16 + self.frequency as u16 + self.monetary as u16
    }

    #[must_use]
    pub const fn segment(self) -> RfmSegment {
        let t = self.total();
        if t >= 13 {
            RfmSegment::Champion
        } else if t >= 10 {
            RfmSegment::Loyal
        } else if t >= 7 {
            RfmSegment::Potential
        } else if t >= 4 {
            RfmSegment::AtRisk
        } else {
            RfmSegment::Lost
        }
    }
}
