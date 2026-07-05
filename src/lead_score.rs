//! lead score.

// ---------------------------------------------------------------------------
// Lead scoring rules
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ScoringRule {
    pub name: String,
    pub points: i32,
    pub condition: ScoringCondition,
}

#[derive(Debug, Clone)]
pub enum ScoringCondition {
    HasTag(String),
    HasCustomField(String),
    ActivityCountGte(usize),
    DealValueGte(f64),
    EmailDomainContains(String),
}
