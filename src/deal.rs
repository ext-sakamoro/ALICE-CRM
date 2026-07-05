//! deal.

use crate::field::FieldValue;
use crate::id_gen::Id;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Deal pipeline
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DealStage {
    Prospect,
    Qualified,
    Proposal,
    Negotiation,
    ClosedWon,
    ClosedLost,
}

impl DealStage {
    /// Default win probability for each stage (0.0 .. 1.0).
    #[must_use]
    pub const fn default_probability(self) -> f64 {
        match self {
            Self::Prospect => 0.10,
            Self::Qualified => 0.25,
            Self::Proposal => 0.50,
            Self::Negotiation => 0.75,
            Self::ClosedWon => 1.00,
            Self::ClosedLost => 0.00,
        }
    }

    pub(crate) const ALL: [Self; 6] = [
        Self::Prospect,
        Self::Qualified,
        Self::Proposal,
        Self::Negotiation,
        Self::ClosedWon,
        Self::ClosedLost,
    ];

    /// Stage ordering index for funnel (0 = top).
    #[must_use]
    pub const fn ordinal(self) -> usize {
        match self {
            Self::Prospect => 0,
            Self::Qualified => 1,
            Self::Proposal => 2,
            Self::Negotiation => 3,
            Self::ClosedWon => 4,
            Self::ClosedLost => 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Deal {
    pub id: Id,
    pub contact_id: Id,
    pub title: String,
    pub value: f64,
    pub stage: DealStage,
    pub probability: f64,
    pub created_at: u64,
    pub closed_at: Option<u64>,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, FieldValue>,
}

impl Deal {
    pub(crate) fn new(id: Id, contact_id: Id, title: &str, value: f64) -> Self {
        let stage = DealStage::Prospect;
        Self {
            id,
            contact_id,
            title: title.to_owned(),
            value,
            stage,
            probability: stage.default_probability(),
            created_at: 0,
            closed_at: None,
            tags: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
}
