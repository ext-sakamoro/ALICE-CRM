//! contact.

use crate::field::FieldValue;
use crate::id_gen::Id;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Contact
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Contact {
    pub id: Id,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub company: String,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, FieldValue>,
    pub lead_score: i32,
    pub created_at: u64,
}

impl Contact {
    pub(crate) fn new(id: Id, name: &str, email: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
            email: email.to_owned(),
            phone: String::new(),
            company: String::new(),
            tags: Vec::new(),
            custom_fields: HashMap::new(),
            lead_score: 0,
            created_at: 0,
        }
    }
}
