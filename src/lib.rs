//! ALICE-CRM: CRM engine.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::return_self_not_must_use
)]

pub mod activity;
pub mod contact;
pub mod deal;
pub mod engine;
pub mod field;
pub mod funnel;
pub mod id_gen;
pub mod lead_score;
pub mod note;
pub mod prelude;
pub mod rfm;
pub mod signed_contact;

#[cfg(test)]
mod integration_tests;

pub use crate::activity::*;
pub use crate::contact::*;
pub use crate::deal::*;
pub use crate::engine::*;
pub use crate::field::*;
pub use crate::funnel::*;
pub use crate::id_gen::*;
pub use crate::lead_score::*;
pub use crate::note::*;
pub use crate::rfm::*;
