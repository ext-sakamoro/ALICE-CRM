//! activity.

use crate::id_gen::Id;

// ---------------------------------------------------------------------------
// Activity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    Call,
    Email,
    Meeting,
    Task,
    Other,
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub id: Id,
    pub contact_id: Id,
    pub kind: ActivityKind,
    pub description: String,
    pub timestamp: u64,
}
