//! `signed_contact` — tamper-evident customer contact + deal activity trail.
//!
//! Wraps every sales-facing event (call, meeting, email, deal stage change,
//! quote, contract, opt-out request) in an `Ed25519`-signed record chained
//! via `prev_hash → hash`. Legal disputes over "the customer never agreed"
//! or "the rep made a promise" can be resolved with cryptographic evidence.
//!
//! # Regulatory alignment
//!
//! - **`GDPR` Art. 7(1)** — proof of consent for direct-marketing contacts;
//!   the signed chain doubles as evidence of the consent moment.
//! - **`GDPR` Art. 17** — right to erasure; opt-out events are recorded
//!   in the chain so downstream processors can honour the timestamp.
//! - **`SOX` §404** — sales cycle transactions feed revenue recognition;
//!   contact history is a supporting audit artefact.
//! - **`CAN-SPAM Act` §5(a)(3)** — opt-out honour within 10 business days
//!   requires provable receipt time.
//! - **改正個人情報保護法 §21** — 苦情処理体制; 接触記録の完全性が
//!   監査対象.
//!
//! Cryptographic primitives are provided by `alice-blockchain` (`Ed25519`).

#![allow(
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::too_many_arguments,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]

use alice_blockchain::signature::{KeyPair, PublicKey, Signature};

// ---------------------------------------------------------------------------
// ContactEventKind
// ---------------------------------------------------------------------------

/// The sales-facing event captured in the trail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContactEventKind {
    /// A call was placed or received.
    Call,
    /// A meeting (in-person or video) took place.
    Meeting,
    /// An email was sent or delivered.
    Email,
    /// The deal moved to a new pipeline stage.
    StageChange,
    /// A quote was issued.
    Quote,
    /// A contract was signed.
    ContractSigned,
    /// The contact opted out of further marketing.
    OptOut,
    /// The contact filed a complaint or dispute.
    Complaint,
}

impl ContactEventKind {
    /// Short code used in canonical serialization.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Call => "CALL",
            Self::Meeting => "MEET",
            Self::Email => "MAIL",
            Self::StageChange => "STAGE",
            Self::Quote => "QUOTE",
            Self::ContractSigned => "CONTR",
            Self::OptOut => "OPTOUT",
            Self::Complaint => "CMPL",
        }
    }
}

// ---------------------------------------------------------------------------
// ContactRecord
// ---------------------------------------------------------------------------

/// One sales-facing event ready to be signed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactRecord {
    /// Monotonic sequence number.
    pub seq: u64,
    /// Kind of event.
    pub kind: ContactEventKind,
    /// Unix nanosecond timestamp.
    pub timestamp_ns: u64,
    /// Contact / customer identifier.
    pub contact_id: String,
    /// Optional deal identifier (empty when no deal is attached).
    pub deal_id: String,
    /// Sales representative user id.
    pub rep_id: String,
    /// Communication channel (`phone`, `zoom`, `email`, `web`, ...).
    pub channel: String,
    /// Optional outcome tag (`connected`, `no_answer`, `qualified`, ...).
    pub outcome: String,
    /// Free-form summary / notes.
    pub summary: String,
    /// Hash of the previous record (0 for genesis).
    pub prev_hash: u64,
}

impl ContactRecord {
    /// Canonical byte layout used for hashing and signing.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(224);
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(self.kind.code().as_bytes());
        buf.push(0);
        buf.extend_from_slice(&self.timestamp_ns.to_le_bytes());
        buf.extend_from_slice(self.contact_id.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.deal_id.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.rep_id.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.channel.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.outcome.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.summary.as_bytes());
        buf.push(0);
        buf.extend_from_slice(&self.prev_hash.to_le_bytes());
        buf
    }

    /// `FNV-1a` hash of the canonical byte layout.
    #[must_use]
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in &self.canonical_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }
}

// ---------------------------------------------------------------------------
// SignedContactRecord
// ---------------------------------------------------------------------------

/// [`ContactRecord`] plus the sales rep's `Ed25519` signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedContactRecord {
    /// The wrapped record.
    pub record: ContactRecord,
    /// `FNV-1a` hash of the record's canonical bytes.
    pub hash: u64,
    /// `Ed25519` signature over the canonical bytes.
    pub signature: Signature,
    /// Rep's `Ed25519` public key.
    pub rep: PublicKey,
}

impl SignedContactRecord {
    /// Verify signature and hash consistency.
    #[must_use]
    pub fn verify(&self) -> bool {
        if self.hash != self.record.hash() {
            return false;
        }
        self.rep
            .verify(&self.record.canonical_bytes(), &self.signature)
    }
}

// ---------------------------------------------------------------------------
// ContactTrail
// ---------------------------------------------------------------------------

/// Append-only chain of [`SignedContactRecord`] records.
#[derive(Debug, Clone, Default)]
pub struct ContactTrail {
    entries: Vec<SignedContactRecord>,
}

impl ContactTrail {
    /// Construct an empty trail.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Number of entries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the trail is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Read-only view.
    #[must_use]
    pub fn entries(&self) -> &[SignedContactRecord] {
        &self.entries
    }

    /// Hash of the last record (0 for empty).
    #[must_use]
    pub fn tail_hash(&self) -> u64 {
        self.entries.last().map_or(0, |e| e.hash)
    }

    /// Append a new event signed with the rep's key pair.
    pub fn append(
        &mut self,
        keypair: &KeyPair,
        kind: ContactEventKind,
        timestamp_ns: u64,
        contact_id: impl Into<String>,
        deal_id: impl Into<String>,
        rep_id: impl Into<String>,
        channel: impl Into<String>,
        outcome: impl Into<String>,
        summary: impl Into<String>,
    ) -> &SignedContactRecord {
        let seq = self.entries.len() as u64;
        let prev_hash = self.tail_hash();
        let record = ContactRecord {
            seq,
            kind,
            timestamp_ns,
            contact_id: contact_id.into(),
            deal_id: deal_id.into(),
            rep_id: rep_id.into(),
            channel: channel.into(),
            outcome: outcome.into(),
            summary: summary.into(),
            prev_hash,
        };
        let bytes = record.canonical_bytes();
        let hash = record.hash();
        let signature = keypair.sign(&bytes);
        let rep = keypair.public();
        self.entries.push(SignedContactRecord {
            record,
            hash,
            signature,
            rep,
        });
        self.entries.last().expect("entry was just pushed")
    }

    /// Verify signature and chain integrity end-to-end.
    #[must_use]
    pub fn find_first_tamper(&self) -> Option<usize> {
        let mut expected_prev: u64 = 0;
        for (i, e) in self.entries.iter().enumerate() {
            if e.record.seq as usize != i {
                return Some(i);
            }
            if e.record.prev_hash != expected_prev {
                return Some(i);
            }
            if !e.verify() {
                return Some(i);
            }
            expected_prev = e.hash;
        }
        None
    }

    /// Whether the trail is intact.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.find_first_tamper().is_none()
    }

    /// Timestamp of the earliest opt-out event for the contact, or `None`
    /// if no opt-out is recorded. Used for CAN-SPAM / GDPR compliance
    /// enforcement.
    #[must_use]
    pub fn opt_out_at(&self, contact_id: &str) -> Option<u64> {
        self.entries
            .iter()
            .filter(|e| {
                e.record.contact_id == contact_id && e.record.kind == ContactEventKind::OptOut
            })
            .map(|e| e.record.timestamp_ns)
            .min()
    }

    /// All distinct contact ids seen in the trail.
    #[must_use]
    pub fn contacts(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for e in &self.entries {
            if !out.contains(&e.record.contact_id) {
                out.push(e.record.contact_id.clone());
            }
        }
        out
    }

    /// Count of events of the given kind.
    #[must_use]
    pub fn count_kind(&self, kind: ContactEventKind) -> usize {
        self.entries
            .iter()
            .filter(|e| e.record.kind == kind)
            .count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn kp(seed: u8) -> KeyPair {
        KeyPair::from_seed([seed; 32])
    }

    #[test]
    fn kind_code_is_stable() {
        assert_eq!(ContactEventKind::Call.code(), "CALL");
        assert_eq!(ContactEventKind::Meeting.code(), "MEET");
        assert_eq!(ContactEventKind::Email.code(), "MAIL");
        assert_eq!(ContactEventKind::StageChange.code(), "STAGE");
        assert_eq!(ContactEventKind::Quote.code(), "QUOTE");
        assert_eq!(ContactEventKind::ContractSigned.code(), "CONTR");
        assert_eq!(ContactEventKind::OptOut.code(), "OPTOUT");
        assert_eq!(ContactEventKind::Complaint.code(), "CMPL");
    }

    #[test]
    fn canonical_bytes_are_deterministic() {
        let r = ContactRecord {
            seq: 0,
            kind: ContactEventKind::Call,
            timestamp_ns: 1,
            contact_id: String::from("C-001"),
            deal_id: String::from("D-100"),
            rep_id: String::from("rep-1"),
            channel: String::from("phone"),
            outcome: String::from("connected"),
            summary: String::from("intro call"),
            prev_hash: 0,
        };
        assert_eq!(r.canonical_bytes(), r.canonical_bytes());
    }

    #[test]
    fn hash_differs_when_summary_changes() {
        let mut r = ContactRecord {
            seq: 0,
            kind: ContactEventKind::Call,
            timestamp_ns: 1,
            contact_id: String::from("C-001"),
            deal_id: String::new(),
            rep_id: String::from("rep-1"),
            channel: String::from("phone"),
            outcome: String::from("connected"),
            summary: String::from("original"),
            prev_hash: 0,
        };
        let h1 = r.hash();
        r.summary = String::from("modified");
        assert_ne!(h1, r.hash());
    }

    #[test]
    fn empty_trail_tail_hash_is_zero() {
        let trail = ContactTrail::new();
        assert_eq!(trail.tail_hash(), 0);
        assert!(trail.is_empty());
    }

    #[test]
    fn signed_record_verifies_on_append() {
        let mut trail = ContactTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            ContactEventKind::Call,
            1,
            "C-001",
            "D-100",
            "rep-1",
            "phone",
            "connected",
            "intro",
        );
        assert!(trail.entries()[0].verify());
    }

    #[test]
    fn chained_prev_hash_matches_predecessor() {
        let mut trail = ContactTrail::new();
        let k = kp(1);
        trail.append(&k, ContactEventKind::Call, 1, "C-001", "", "r", "p", "", "");
        trail.append(
            &k,
            ContactEventKind::Email,
            2,
            "C-001",
            "",
            "r",
            "mail",
            "",
            "",
        );
        let first = trail.entries()[0].hash;
        assert_eq!(trail.entries()[1].record.prev_hash, first);
    }

    #[test]
    fn intact_trail_is_valid() {
        let mut trail = ContactTrail::new();
        let k = kp(1);
        for i in 0..5 {
            trail.append(&k, ContactEventKind::Call, i, "C-001", "", "r", "p", "", "");
        }
        assert!(trail.is_valid());
    }

    #[test]
    fn tampered_outcome_is_detected() {
        let mut trail = ContactTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            ContactEventKind::Meeting,
            1,
            "C-001",
            "",
            "rep-1",
            "zoom",
            "qualified",
            "",
        );
        // Attacker rewrites "qualified" → "declined".
        trail.entries[0].record.outcome = String::from("declined");
        assert!(!trail.entries[0].verify());
        assert_eq!(trail.find_first_tamper(), Some(0));
    }

    #[test]
    fn tampered_contact_id_is_detected() {
        let mut trail = ContactTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            ContactEventKind::OptOut,
            1,
            "C-genuine",
            "",
            "rep-1",
            "web",
            "",
            "",
        );
        trail.entries[0].record.contact_id = String::from("C-attacker");
        assert!(!trail.entries[0].verify());
    }

    #[test]
    fn foreign_rep_signature_is_rejected() {
        let mut trail = ContactTrail::new();
        let genuine = kp(1);
        let attacker = kp(2);
        trail.append(
            &genuine,
            ContactEventKind::ContractSigned,
            1,
            "C-001",
            "D-100",
            "rep-1",
            "docusign",
            "signed",
            "",
        );
        let bytes = trail.entries[0].record.canonical_bytes();
        trail.entries[0].signature = attacker.sign(&bytes);
        assert!(!trail.entries[0].verify());
    }

    #[test]
    fn opt_out_at_returns_earliest_timestamp() {
        let mut trail = ContactTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            ContactEventKind::Call,
            100,
            "C-001",
            "",
            "r",
            "p",
            "",
            "",
        );
        trail.append(
            &k,
            ContactEventKind::OptOut,
            200,
            "C-001",
            "",
            "r",
            "web",
            "",
            "",
        );
        // A second opt-out later must not overwrite the first.
        trail.append(
            &k,
            ContactEventKind::OptOut,
            300,
            "C-001",
            "",
            "r",
            "web",
            "",
            "",
        );
        assert_eq!(trail.opt_out_at("C-001"), Some(200));
    }

    #[test]
    fn opt_out_at_returns_none_when_never_recorded() {
        let mut trail = ContactTrail::new();
        let k = kp(1);
        trail.append(
            &k,
            ContactEventKind::Call,
            100,
            "C-001",
            "",
            "r",
            "p",
            "",
            "",
        );
        assert_eq!(trail.opt_out_at("C-001"), None);
    }

    #[test]
    fn contacts_lists_distinct() {
        let mut trail = ContactTrail::new();
        let k = kp(1);
        trail.append(&k, ContactEventKind::Call, 1, "C-A", "", "r", "p", "", "");
        trail.append(&k, ContactEventKind::Call, 2, "C-B", "", "r", "p", "", "");
        trail.append(&k, ContactEventKind::Email, 3, "C-A", "", "r", "m", "", "");
        let contacts = trail.contacts();
        assert_eq!(contacts.len(), 2);
        assert!(contacts.contains(&String::from("C-A")));
        assert!(contacts.contains(&String::from("C-B")));
    }

    #[test]
    fn count_kind_filters() {
        let mut trail = ContactTrail::new();
        let k = kp(1);
        for _ in 0..3 {
            trail.append(&k, ContactEventKind::Call, 0, "C", "", "r", "p", "", "");
        }
        for _ in 0..2 {
            trail.append(&k, ContactEventKind::Email, 0, "C", "", "r", "m", "", "");
        }
        assert_eq!(trail.count_kind(ContactEventKind::Call), 3);
        assert_eq!(trail.count_kind(ContactEventKind::Email), 2);
        assert_eq!(trail.count_kind(ContactEventKind::OptOut), 0);
    }

    #[test]
    fn different_kinds_produce_different_hashes() {
        let mk = |kind: ContactEventKind| ContactRecord {
            seq: 0,
            kind,
            timestamp_ns: 1,
            contact_id: String::new(),
            deal_id: String::new(),
            rep_id: String::new(),
            channel: String::new(),
            outcome: String::new(),
            summary: String::new(),
            prev_hash: 0,
        };
        assert_ne!(
            mk(ContactEventKind::Call).hash(),
            mk(ContactEventKind::OptOut).hash()
        );
    }
}
