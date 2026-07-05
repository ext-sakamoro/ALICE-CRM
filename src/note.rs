//! note.

use crate::id_gen::Id;

// ---------------------------------------------------------------------------
// Note
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Note {
    pub id: Id,
    pub contact_id: Id,
    pub content: String,
    pub timestamp: u64,
}
