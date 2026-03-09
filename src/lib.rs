#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions, clippy::cast_precision_loss)]

//! ALICE-CRM: Customer Relationship Management
//!
//! Pure Rust CRM with contact management, deal pipeline, lead scoring,
//! RFM segmentation, activity tracking, notes, tags, custom fields,
//! funnel metrics, and conversion rate analytics.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ID generation (simple monotonic counter per CRM instance)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Custom field values
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Text(String),
    Number(f64),
    Bool(bool),
}

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
    fn new(id: Id, name: &str, email: &str) -> Self {
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

    const ALL: [Self; 6] = [
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
    fn new(id: Id, contact_id: Id, title: &str, value: f64) -> Self {
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

// ---------------------------------------------------------------------------
// Funnel metrics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FunnelMetrics {
    pub stage_counts: HashMap<DealStage, usize>,
    pub stage_values: HashMap<DealStage, f64>,
    pub conversion_rates: HashMap<DealStage, f64>,
    pub total_pipeline_value: f64,
    pub weighted_pipeline_value: f64,
    pub win_rate: f64,
    pub average_deal_value: f64,
}

// ---------------------------------------------------------------------------
// CRM engine
// ---------------------------------------------------------------------------

pub struct Crm {
    next_id: u64,
    contacts: Vec<Contact>,
    deals: Vec<Deal>,
    activities: Vec<Activity>,
    notes: Vec<Note>,
    scoring_rules: Vec<ScoringRule>,
}

impl Default for Crm {
    fn default() -> Self {
        Self::new()
    }
}

impl Crm {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_id: 1,
            contacts: Vec::new(),
            deals: Vec::new(),
            activities: Vec::new(),
            notes: Vec::new(),
            scoring_rules: Vec::new(),
        }
    }

    const fn gen_id(&mut self) -> Id {
        let id = Id(self.next_id);
        self.next_id += 1;
        id
    }

    // -- Contacts -----------------------------------------------------------

    pub fn add_contact(&mut self, name: &str, email: &str) -> Id {
        let id = self.gen_id();
        self.contacts.push(Contact::new(id, name, email));
        id
    }

    #[must_use]
    pub fn get_contact(&self, id: Id) -> Option<&Contact> {
        self.contacts.iter().find(|c| c.id == id)
    }

    pub fn get_contact_mut(&mut self, id: Id) -> Option<&mut Contact> {
        self.contacts.iter_mut().find(|c| c.id == id)
    }

    #[must_use]
    pub fn list_contacts(&self) -> &[Contact] {
        &self.contacts
    }

    pub fn delete_contact(&mut self, id: Id) -> bool {
        let len = self.contacts.len();
        self.contacts.retain(|c| c.id != id);
        self.contacts.len() != len
    }

    #[must_use]
    pub fn search_contacts(&self, query: &str) -> Vec<&Contact> {
        let q = query.to_lowercase();
        self.contacts
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&q)
                    || c.email.to_lowercase().contains(&q)
                    || c.company.to_lowercase().contains(&q)
            })
            .collect()
    }

    #[must_use]
    pub fn contacts_by_tag(&self, tag: &str) -> Vec<&Contact> {
        self.contacts
            .iter()
            .filter(|c| c.tags.iter().any(|t| t == tag))
            .collect()
    }

    #[must_use]
    pub const fn contact_count(&self) -> usize {
        self.contacts.len()
    }

    // -- Tags ---------------------------------------------------------------

    pub fn add_tag_to_contact(&mut self, contact_id: Id, tag: &str) -> bool {
        if let Some(c) = self.get_contact_mut(contact_id) {
            let tag_str = tag.to_owned();
            if !c.tags.contains(&tag_str) {
                c.tags.push(tag_str);
            }
            true
        } else {
            false
        }
    }

    pub fn remove_tag_from_contact(&mut self, contact_id: Id, tag: &str) -> bool {
        if let Some(c) = self.get_contact_mut(contact_id) {
            let len = c.tags.len();
            c.tags.retain(|t| t != tag);
            c.tags.len() != len
        } else {
            false
        }
    }

    // -- Custom fields ------------------------------------------------------

    pub fn set_contact_field(&mut self, contact_id: Id, key: &str, value: FieldValue) -> bool {
        if let Some(c) = self.get_contact_mut(contact_id) {
            c.custom_fields.insert(key.to_owned(), value);
            true
        } else {
            false
        }
    }

    pub fn remove_contact_field(&mut self, contact_id: Id, key: &str) -> bool {
        if let Some(c) = self.get_contact_mut(contact_id) {
            c.custom_fields.remove(key).is_some()
        } else {
            false
        }
    }

    // -- Deals --------------------------------------------------------------

    pub fn add_deal(&mut self, contact_id: Id, title: &str, value: f64) -> Id {
        let id = self.gen_id();
        self.deals.push(Deal::new(id, contact_id, title, value));
        id
    }

    #[must_use]
    pub fn get_deal(&self, id: Id) -> Option<&Deal> {
        self.deals.iter().find(|d| d.id == id)
    }

    pub fn get_deal_mut(&mut self, id: Id) -> Option<&mut Deal> {
        self.deals.iter_mut().find(|d| d.id == id)
    }

    #[must_use]
    pub fn list_deals(&self) -> &[Deal] {
        &self.deals
    }

    pub fn delete_deal(&mut self, id: Id) -> bool {
        let len = self.deals.len();
        self.deals.retain(|d| d.id != id);
        self.deals.len() != len
    }

    pub fn advance_deal(&mut self, deal_id: Id, stage: DealStage) -> bool {
        if let Some(d) = self.get_deal_mut(deal_id) {
            d.stage = stage;
            d.probability = stage.default_probability();
            if stage == DealStage::ClosedWon || stage == DealStage::ClosedLost {
                d.closed_at = Some(0);
            }
            true
        } else {
            false
        }
    }

    pub fn set_deal_probability(&mut self, deal_id: Id, probability: f64) -> bool {
        if let Some(d) = self.get_deal_mut(deal_id) {
            d.probability = probability.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    pub fn set_deal_field(&mut self, deal_id: Id, key: &str, value: FieldValue) -> bool {
        if let Some(d) = self.get_deal_mut(deal_id) {
            d.custom_fields.insert(key.to_owned(), value);
            true
        } else {
            false
        }
    }

    pub fn add_tag_to_deal(&mut self, deal_id: Id, tag: &str) -> bool {
        if let Some(d) = self.get_deal_mut(deal_id) {
            let tag_str = tag.to_owned();
            if !d.tags.contains(&tag_str) {
                d.tags.push(tag_str);
            }
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn deals_by_contact(&self, contact_id: Id) -> Vec<&Deal> {
        self.deals
            .iter()
            .filter(|d| d.contact_id == contact_id)
            .collect()
    }

    #[must_use]
    pub fn deals_by_stage(&self, stage: DealStage) -> Vec<&Deal> {
        self.deals.iter().filter(|d| d.stage == stage).collect()
    }

    #[must_use]
    pub const fn deal_count(&self) -> usize {
        self.deals.len()
    }

    // -- Activities ---------------------------------------------------------

    pub fn add_activity(&mut self, contact_id: Id, kind: ActivityKind, description: &str) -> Id {
        let id = self.gen_id();
        self.activities.push(Activity {
            id,
            contact_id,
            kind,
            description: description.to_owned(),
            timestamp: 0,
        });
        id
    }

    #[must_use]
    pub fn activities_for_contact(&self, contact_id: Id) -> Vec<&Activity> {
        self.activities
            .iter()
            .filter(|a| a.contact_id == contact_id)
            .collect()
    }

    #[must_use]
    pub fn activities_by_kind(&self, kind: ActivityKind) -> Vec<&Activity> {
        self.activities.iter().filter(|a| a.kind == kind).collect()
    }

    #[must_use]
    pub const fn activity_count(&self) -> usize {
        self.activities.len()
    }

    // -- Notes --------------------------------------------------------------

    pub fn add_note(&mut self, contact_id: Id, content: &str) -> Id {
        let id = self.gen_id();
        self.notes.push(Note {
            id,
            contact_id,
            content: content.to_owned(),
            timestamp: 0,
        });
        id
    }

    #[must_use]
    pub fn notes_for_contact(&self, contact_id: Id) -> Vec<&Note> {
        self.notes
            .iter()
            .filter(|n| n.contact_id == contact_id)
            .collect()
    }

    pub fn delete_note(&mut self, note_id: Id) -> bool {
        let len = self.notes.len();
        self.notes.retain(|n| n.id != note_id);
        self.notes.len() != len
    }

    #[must_use]
    pub const fn note_count(&self) -> usize {
        self.notes.len()
    }

    // -- Lead scoring -------------------------------------------------------

    pub fn add_scoring_rule(&mut self, rule: ScoringRule) {
        self.scoring_rules.push(rule);
    }

    pub fn clear_scoring_rules(&mut self) {
        self.scoring_rules.clear();
    }

    #[must_use]
    pub const fn scoring_rule_count(&self) -> usize {
        self.scoring_rules.len()
    }

    /// Compute lead score for a contact based on scoring rules.
    #[must_use]
    pub fn compute_lead_score(&self, contact_id: Id) -> i32 {
        let Some(contact) = self.get_contact(contact_id) else {
            return 0;
        };

        let activity_count = self.activities_for_contact(contact_id).len();
        let max_deal_value = self
            .deals_by_contact(contact_id)
            .iter()
            .map(|d| d.value)
            .fold(0.0_f64, f64::max);

        let mut score = 0i32;
        for rule in &self.scoring_rules {
            let matches = match &rule.condition {
                ScoringCondition::HasTag(tag) => contact.tags.contains(tag),
                ScoringCondition::HasCustomField(key) => contact.custom_fields.contains_key(key),
                ScoringCondition::ActivityCountGte(n) => activity_count >= *n,
                ScoringCondition::DealValueGte(v) => max_deal_value >= *v,
                ScoringCondition::EmailDomainContains(domain) => contact
                    .email
                    .to_lowercase()
                    .contains(&domain.to_lowercase()),
            };
            if matches {
                score += rule.points;
            }
        }
        score
    }

    /// Apply scoring rules and update each contact's `lead_score`.
    pub fn apply_scoring(&mut self) {
        let scores: Vec<(Id, i32)> = self
            .contacts
            .iter()
            .map(|c| (c.id, self.compute_lead_score(c.id)))
            .collect();
        for (id, score) in scores {
            if let Some(c) = self.get_contact_mut(id) {
                c.lead_score = score;
            }
        }
    }

    // -- RFM segmentation ---------------------------------------------------

    /// Compute RFM score for a contact.
    ///
    /// - `now`: current timestamp
    /// - Uses deals as monetary transactions
    /// - Recency = time since last closed-won deal
    /// - Frequency = number of closed-won deals
    /// - Monetary = total value of closed-won deals
    #[must_use]
    pub fn compute_rfm(&self, contact_id: Id, now: u64) -> RfmScore {
        let deals: Vec<&Deal> = self
            .deals
            .iter()
            .filter(|d| d.contact_id == contact_id && d.stage == DealStage::ClosedWon)
            .collect();

        let frequency = deals.len();
        let monetary: f64 = deals.iter().map(|d| d.value).sum();
        let last_closed = deals.iter().filter_map(|d| d.closed_at).max().unwrap_or(0);
        let days_since = if now > last_closed {
            (now - last_closed) / 86400
        } else {
            0
        };

        let r = match days_since {
            0..=30 => 5,
            31..=90 => 4,
            91..=180 => 3,
            181..=365 => 2,
            _ => 1,
        };

        let f = match frequency {
            0 => 1,
            1 => 2,
            2..=4 => 3,
            5..=9 => 4,
            _ => 5,
        };

        #[allow(clippy::cast_possible_truncation)]
        let m = if monetary >= 100_000.0 {
            5
        } else if monetary >= 50_000.0 {
            4
        } else if monetary >= 10_000.0 {
            3
        } else if monetary >= 1_000.0 {
            2
        } else {
            1
        };

        RfmScore {
            recency: r,
            frequency: f,
            monetary: m,
        }
    }

    #[must_use]
    pub fn segment_contact(&self, contact_id: Id, now: u64) -> RfmSegment {
        self.compute_rfm(contact_id, now).segment()
    }

    // -- Funnel metrics -----------------------------------------------------

    #[must_use]
    pub fn funnel_metrics(&self) -> FunnelMetrics {
        let mut stage_counts: HashMap<DealStage, usize> = HashMap::new();
        let mut stage_values: HashMap<DealStage, f64> = HashMap::new();

        for deal in &self.deals {
            *stage_counts.entry(deal.stage).or_insert(0) += 1;
            *stage_values.entry(deal.stage).or_insert(0.0) += deal.value;
        }

        let total_pipeline_value: f64 = self
            .deals
            .iter()
            .filter(|d| d.stage != DealStage::ClosedWon && d.stage != DealStage::ClosedLost)
            .map(|d| d.value)
            .sum();

        let weighted_pipeline_value: f64 = self
            .deals
            .iter()
            .filter(|d| d.stage != DealStage::ClosedWon && d.stage != DealStage::ClosedLost)
            .map(|d| d.value * d.probability)
            .sum();

        let won = stage_counts
            .get(&DealStage::ClosedWon)
            .copied()
            .unwrap_or(0);
        let lost = stage_counts
            .get(&DealStage::ClosedLost)
            .copied()
            .unwrap_or(0);
        let closed_total = won + lost;
        let win_rate = if closed_total > 0 {
            won as f64 / closed_total as f64
        } else {
            0.0
        };

        let deal_total = self.deals.len();
        let average_deal_value = if deal_total > 0 {
            self.deals.iter().map(|d| d.value).sum::<f64>() / deal_total as f64
        } else {
            0.0
        };

        // Conversion rates between consecutive open stages
        let mut conversion_rates: HashMap<DealStage, f64> = HashMap::new();
        let open_stages = [
            DealStage::Prospect,
            DealStage::Qualified,
            DealStage::Proposal,
            DealStage::Negotiation,
        ];
        for i in 0..open_stages.len() - 1 {
            let current = stage_counts.get(&open_stages[i]).copied().unwrap_or(0);
            let next = stage_counts.get(&open_stages[i + 1]).copied().unwrap_or(0);
            let rate = if current > 0 {
                next as f64 / current as f64
            } else {
                0.0
            };
            conversion_rates.insert(open_stages[i], rate);
        }

        FunnelMetrics {
            stage_counts,
            stage_values,
            conversion_rates,
            total_pipeline_value,
            weighted_pipeline_value,
            win_rate,
            average_deal_value,
        }
    }

    /// Overall conversion rate from `from` stage to `to` stage.
    #[must_use]
    pub fn conversion_rate(&self, from: DealStage, to: DealStage) -> f64 {
        let from_count = self
            .deals
            .iter()
            .filter(|d| d.stage.ordinal() >= from.ordinal())
            .count();
        let to_count = self
            .deals
            .iter()
            .filter(|d| d.stage.ordinal() >= to.ordinal())
            .count();
        if from_count > 0 {
            to_count as f64 / from_count as f64
        } else {
            0.0
        }
    }

    // -- Bulk / utility -----------------------------------------------------

    #[must_use]
    pub fn top_contacts_by_score(&self, limit: usize) -> Vec<&Contact> {
        let mut sorted: Vec<&Contact> = self.contacts.iter().collect();
        sorted.sort_by(|a, b| b.lead_score.cmp(&a.lead_score));
        sorted.truncate(limit);
        sorted
    }

    #[must_use]
    pub fn deals_total_value(&self) -> f64 {
        self.deals.iter().map(|d| d.value).sum()
    }

    #[must_use]
    pub fn won_deals_value(&self) -> f64 {
        self.deals
            .iter()
            .filter(|d| d.stage == DealStage::ClosedWon)
            .map(|d| d.value)
            .sum()
    }

    #[must_use]
    pub fn lost_deals_count(&self) -> usize {
        self.deals
            .iter()
            .filter(|d| d.stage == DealStage::ClosedLost)
            .count()
    }

    #[must_use]
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .contacts
            .iter()
            .flat_map(|c| c.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    #[must_use]
    pub fn contacts_with_segment(&self, segment: RfmSegment, now: u64) -> Vec<&Contact> {
        self.contacts
            .iter()
            .filter(|c| self.segment_contact(c.id, now) == segment)
            .collect()
    }

    /// Return all deal stages that have at least one deal.
    #[must_use]
    pub fn active_stages(&self) -> Vec<DealStage> {
        DealStage::ALL
            .iter()
            .copied()
            .filter(|s| self.deals.iter().any(|d| d.stage == *s))
            .collect()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_crm() -> Crm {
        Crm::new()
    }

    // --- Contact tests ---

    #[test]
    fn test_add_contact() {
        let mut crm = make_crm();
        let id = crm.add_contact("Alice", "alice@example.com");
        assert_eq!(crm.contact_count(), 1);
        let c = crm.get_contact(id).unwrap();
        assert_eq!(c.name, "Alice");
    }

    #[test]
    fn test_add_multiple_contacts() {
        let mut crm = make_crm();
        crm.add_contact("A", "a@a.com");
        crm.add_contact("B", "b@b.com");
        crm.add_contact("C", "c@c.com");
        assert_eq!(crm.contact_count(), 3);
    }

    #[test]
    fn test_get_contact_not_found() {
        let crm = make_crm();
        assert!(crm.get_contact(Id(999)).is_none());
    }

    #[test]
    fn test_delete_contact() {
        let mut crm = make_crm();
        let id = crm.add_contact("Del", "d@d.com");
        assert!(crm.delete_contact(id));
        assert_eq!(crm.contact_count(), 0);
    }

    #[test]
    fn test_delete_contact_not_found() {
        let mut crm = make_crm();
        assert!(!crm.delete_contact(Id(999)));
    }

    #[test]
    fn test_search_contacts_by_name() {
        let mut crm = make_crm();
        crm.add_contact("Alice Smith", "a@a.com");
        crm.add_contact("Bob Jones", "b@b.com");
        let res = crm.search_contacts("alice");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "Alice Smith");
    }

    #[test]
    fn test_search_contacts_by_email() {
        let mut crm = make_crm();
        crm.add_contact("Alice", "alice@example.com");
        let res = crm.search_contacts("example.com");
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_search_contacts_by_company() {
        let mut crm = make_crm();
        let id = crm.add_contact("Alice", "a@a.com");
        crm.get_contact_mut(id).unwrap().company = "Acme Corp".to_owned();
        let res = crm.search_contacts("acme");
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_search_contacts_no_match() {
        let mut crm = make_crm();
        crm.add_contact("Alice", "a@a.com");
        let res = crm.search_contacts("zzz");
        assert!(res.is_empty());
    }

    #[test]
    fn test_list_contacts() {
        let mut crm = make_crm();
        crm.add_contact("A", "a@a.com");
        crm.add_contact("B", "b@b.com");
        assert_eq!(crm.list_contacts().len(), 2);
    }

    #[test]
    fn test_contact_phone() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        crm.get_contact_mut(id).unwrap().phone = "123".to_owned();
        assert_eq!(crm.get_contact(id).unwrap().phone, "123");
    }

    // --- Tag tests ---

    #[test]
    fn test_add_tag() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        assert!(crm.add_tag_to_contact(id, "vip"));
        assert_eq!(crm.get_contact(id).unwrap().tags, vec!["vip"]);
    }

    #[test]
    fn test_add_duplicate_tag() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        crm.add_tag_to_contact(id, "vip");
        crm.add_tag_to_contact(id, "vip");
        assert_eq!(crm.get_contact(id).unwrap().tags.len(), 1);
    }

    #[test]
    fn test_remove_tag() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        crm.add_tag_to_contact(id, "vip");
        assert!(crm.remove_tag_from_contact(id, "vip"));
        assert!(crm.get_contact(id).unwrap().tags.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_tag() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        assert!(!crm.remove_tag_from_contact(id, "nope"));
    }

    #[test]
    fn test_contacts_by_tag() {
        let mut crm = make_crm();
        let a = crm.add_contact("A", "a@a.com");
        let _b = crm.add_contact("B", "b@b.com");
        crm.add_tag_to_contact(a, "vip");
        let res = crm.contacts_by_tag("vip");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "A");
    }

    #[test]
    fn test_all_tags() {
        let mut crm = make_crm();
        let a = crm.add_contact("A", "a@a.com");
        let b = crm.add_contact("B", "b@b.com");
        crm.add_tag_to_contact(a, "beta");
        crm.add_tag_to_contact(a, "alpha");
        crm.add_tag_to_contact(b, "beta");
        let tags = crm.all_tags();
        assert_eq!(tags, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_add_tag_invalid_contact() {
        let mut crm = make_crm();
        assert!(!crm.add_tag_to_contact(Id(999), "vip"));
    }

    // --- Custom fields ---

    #[test]
    fn test_set_contact_field_text() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        crm.set_contact_field(id, "industry", FieldValue::Text("tech".into()));
        let c = crm.get_contact(id).unwrap();
        assert_eq!(c.custom_fields["industry"], FieldValue::Text("tech".into()));
    }

    #[test]
    fn test_set_contact_field_number() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        crm.set_contact_field(id, "revenue", FieldValue::Number(50_000.0));
        let c = crm.get_contact(id).unwrap();
        assert_eq!(c.custom_fields["revenue"], FieldValue::Number(50_000.0));
    }

    #[test]
    fn test_set_contact_field_bool() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        crm.set_contact_field(id, "active", FieldValue::Bool(true));
        let c = crm.get_contact(id).unwrap();
        assert_eq!(c.custom_fields["active"], FieldValue::Bool(true));
    }

    #[test]
    fn test_remove_contact_field() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        crm.set_contact_field(id, "k", FieldValue::Bool(true));
        assert!(crm.remove_contact_field(id, "k"));
        assert!(crm.get_contact(id).unwrap().custom_fields.is_empty());
    }

    #[test]
    fn test_remove_contact_field_nonexistent() {
        let mut crm = make_crm();
        let id = crm.add_contact("A", "a@a.com");
        assert!(!crm.remove_contact_field(id, "nope"));
    }

    #[test]
    fn test_set_field_invalid_contact() {
        let mut crm = make_crm();
        assert!(!crm.set_contact_field(Id(999), "k", FieldValue::Bool(true)));
    }

    // --- Deal tests ---

    #[test]
    fn test_add_deal() {
        let mut crm = make_crm();
        let cid = crm.add_contact("A", "a@a.com");
        let did = crm.add_deal(cid, "Big Deal", 100_000.0);
        assert_eq!(crm.deal_count(), 1);
        let d = crm.get_deal(did).unwrap();
        assert_eq!(d.title, "Big Deal");
        assert_eq!(d.stage, DealStage::Prospect);
    }

    #[test]
    fn test_advance_deal() {
        let mut crm = make_crm();
        let cid = crm.add_contact("A", "a@a.com");
        let did = crm.add_deal(cid, "D", 1000.0);
        crm.advance_deal(did, DealStage::Qualified);
        assert_eq!(crm.get_deal(did).unwrap().stage, DealStage::Qualified);
    }

    #[test]
    fn test_advance_deal_to_closed_won() {
        let mut crm = make_crm();
        let cid = crm.add_contact("A", "a@a.com");
        let did = crm.add_deal(cid, "D", 5000.0);
        crm.advance_deal(did, DealStage::ClosedWon);
        let d = crm.get_deal(did).unwrap();
        assert_eq!(d.stage, DealStage::ClosedWon);
        assert!(d.closed_at.is_some());
    }

    #[test]
    fn test_advance_deal_to_closed_lost() {
        let mut crm = make_crm();
        let cid = crm.add_contact("A", "a@a.com");
        let did = crm.add_deal(cid, "D", 5000.0);
        crm.advance_deal(did, DealStage::ClosedLost);
        let d = crm.get_deal(did).unwrap();
        assert_eq!(d.stage, DealStage::ClosedLost);
        assert!(d.closed_at.is_some());
    }

    #[test]
    fn test_advance_deal_invalid() {
        let mut crm = make_crm();
        assert!(!crm.advance_deal(Id(999), DealStage::Qualified));
    }

    #[test]
    fn test_set_deal_probability() {
        let mut crm = make_crm();
        let cid = crm.add_contact("A", "a@a.com");
        let did = crm.add_deal(cid, "D", 1000.0);
        crm.set_deal_probability(did, 0.65);
        assert!((crm.get_deal(did).unwrap().probability - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_deal_probability_clamped() {
        let mut crm = make_crm();
        let cid = crm.add_contact("A", "a@a.com");
        let did = crm.add_deal(cid, "D", 1000.0);
        crm.set_deal_probability(did, 1.5);
        assert!((crm.get_deal(did).unwrap().probability - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_deal_probability_invalid() {
        let mut crm = make_crm();
        assert!(!crm.set_deal_probability(Id(999), 0.5));
    }

    #[test]
    fn test_delete_deal() {
        let mut crm = make_crm();
        let cid = crm.add_contact("A", "a@a.com");
        let did = crm.add_deal(cid, "D", 1000.0);
        assert!(crm.delete_deal(did));
        assert_eq!(crm.deal_count(), 0);
    }

    #[test]
    fn test_delete_deal_not_found() {
        let mut crm = make_crm();
        assert!(!crm.delete_deal(Id(999)));
    }

    #[test]
    fn test_deals_by_contact() {
        let mut crm = make_crm();
        let a = crm.add_contact("A", "a@a.com");
        let b = crm.add_contact("B", "b@b.com");
        crm.add_deal(a, "D1", 1000.0);
        crm.add_deal(a, "D2", 2000.0);
        crm.add_deal(b, "D3", 3000.0);
        assert_eq!(crm.deals_by_contact(a).len(), 2);
        assert_eq!(crm.deals_by_contact(b).len(), 1);
    }

    #[test]
    fn test_deals_by_stage() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        let d1 = crm.add_deal(c, "D1", 1000.0);
        crm.add_deal(c, "D2", 2000.0);
        crm.advance_deal(d1, DealStage::Qualified);
        assert_eq!(crm.deals_by_stage(DealStage::Prospect).len(), 1);
        assert_eq!(crm.deals_by_stage(DealStage::Qualified).len(), 1);
    }

    #[test]
    fn test_deal_custom_field() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        let d = crm.add_deal(c, "D", 100.0);
        crm.set_deal_field(d, "source", FieldValue::Text("web".into()));
        assert_eq!(
            crm.get_deal(d).unwrap().custom_fields["source"],
            FieldValue::Text("web".into())
        );
    }

    #[test]
    fn test_deal_custom_field_invalid() {
        let mut crm = make_crm();
        assert!(!crm.set_deal_field(Id(999), "k", FieldValue::Bool(false)));
    }

    #[test]
    fn test_add_tag_to_deal() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        let d = crm.add_deal(c, "D", 100.0);
        assert!(crm.add_tag_to_deal(d, "hot"));
        assert_eq!(crm.get_deal(d).unwrap().tags, vec!["hot"]);
    }

    #[test]
    fn test_add_tag_to_deal_invalid() {
        let mut crm = make_crm();
        assert!(!crm.add_tag_to_deal(Id(999), "hot"));
    }

    #[test]
    fn test_deal_default_probability() {
        assert!((DealStage::Prospect.default_probability() - 0.10).abs() < f64::EPSILON);
        assert!((DealStage::ClosedWon.default_probability() - 1.0).abs() < f64::EPSILON);
        assert!((DealStage::ClosedLost.default_probability()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deals_total_value() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D1", 1000.0);
        crm.add_deal(c, "D2", 2000.0);
        assert!((crm.deals_total_value() - 3000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_won_deals_value() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        let d1 = crm.add_deal(c, "D1", 5000.0);
        crm.add_deal(c, "D2", 2000.0);
        crm.advance_deal(d1, DealStage::ClosedWon);
        assert!((crm.won_deals_value() - 5000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lost_deals_count() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        let d = crm.add_deal(c, "D", 100.0);
        crm.advance_deal(d, DealStage::ClosedLost);
        assert_eq!(crm.lost_deals_count(), 1);
    }

    // --- Activity tests ---

    #[test]
    fn test_add_activity() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_activity(c, ActivityKind::Call, "Called client");
        assert_eq!(crm.activity_count(), 1);
    }

    #[test]
    fn test_activities_for_contact() {
        let mut crm = make_crm();
        let a = crm.add_contact("A", "a@a.com");
        let b = crm.add_contact("B", "b@b.com");
        crm.add_activity(a, ActivityKind::Email, "Sent proposal");
        crm.add_activity(a, ActivityKind::Call, "Follow up");
        crm.add_activity(b, ActivityKind::Meeting, "Demo");
        assert_eq!(crm.activities_for_contact(a).len(), 2);
        assert_eq!(crm.activities_for_contact(b).len(), 1);
    }

    #[test]
    fn test_activities_by_kind() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_activity(c, ActivityKind::Call, "Call 1");
        crm.add_activity(c, ActivityKind::Email, "Email 1");
        crm.add_activity(c, ActivityKind::Call, "Call 2");
        assert_eq!(crm.activities_by_kind(ActivityKind::Call).len(), 2);
        assert_eq!(crm.activities_by_kind(ActivityKind::Email).len(), 1);
    }

    #[test]
    fn test_activity_kinds() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_activity(c, ActivityKind::Task, "Task");
        crm.add_activity(c, ActivityKind::Other, "Other");
        crm.add_activity(c, ActivityKind::Meeting, "Meet");
        assert_eq!(crm.activity_count(), 3);
    }

    // --- Note tests ---

    #[test]
    fn test_add_note() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_note(c, "Important client");
        assert_eq!(crm.note_count(), 1);
    }

    #[test]
    fn test_notes_for_contact() {
        let mut crm = make_crm();
        let a = crm.add_contact("A", "a@a.com");
        let b = crm.add_contact("B", "b@b.com");
        crm.add_note(a, "Note 1");
        crm.add_note(a, "Note 2");
        crm.add_note(b, "Note 3");
        assert_eq!(crm.notes_for_contact(a).len(), 2);
    }

    #[test]
    fn test_delete_note() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        let nid = crm.add_note(c, "Del me");
        assert!(crm.delete_note(nid));
        assert_eq!(crm.note_count(), 0);
    }

    #[test]
    fn test_delete_note_not_found() {
        let mut crm = make_crm();
        assert!(!crm.delete_note(Id(999)));
    }

    // --- Lead scoring tests ---

    #[test]
    fn test_add_scoring_rule() {
        let mut crm = make_crm();
        crm.add_scoring_rule(ScoringRule {
            name: "VIP".into(),
            points: 10,
            condition: ScoringCondition::HasTag("vip".into()),
        });
        assert_eq!(crm.scoring_rule_count(), 1);
    }

    #[test]
    fn test_clear_scoring_rules() {
        let mut crm = make_crm();
        crm.add_scoring_rule(ScoringRule {
            name: "R".into(),
            points: 5,
            condition: ScoringCondition::HasTag("x".into()),
        });
        crm.clear_scoring_rules();
        assert_eq!(crm.scoring_rule_count(), 0);
    }

    #[test]
    fn test_compute_score_has_tag() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_tag_to_contact(c, "vip");
        crm.add_scoring_rule(ScoringRule {
            name: "VIP".into(),
            points: 10,
            condition: ScoringCondition::HasTag("vip".into()),
        });
        assert_eq!(crm.compute_lead_score(c), 10);
    }

    #[test]
    fn test_compute_score_no_match() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_scoring_rule(ScoringRule {
            name: "VIP".into(),
            points: 10,
            condition: ScoringCondition::HasTag("vip".into()),
        });
        assert_eq!(crm.compute_lead_score(c), 0);
    }

    #[test]
    fn test_compute_score_has_custom_field() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.set_contact_field(c, "budget", FieldValue::Number(50_000.0));
        crm.add_scoring_rule(ScoringRule {
            name: "Budget".into(),
            points: 15,
            condition: ScoringCondition::HasCustomField("budget".into()),
        });
        assert_eq!(crm.compute_lead_score(c), 15);
    }

    #[test]
    fn test_compute_score_activity_count() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_activity(c, ActivityKind::Call, "C1");
        crm.add_activity(c, ActivityKind::Email, "E1");
        crm.add_activity(c, ActivityKind::Meeting, "M1");
        crm.add_scoring_rule(ScoringRule {
            name: "Active".into(),
            points: 20,
            condition: ScoringCondition::ActivityCountGte(3),
        });
        assert_eq!(crm.compute_lead_score(c), 20);
    }

    #[test]
    fn test_compute_score_activity_count_not_met() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_activity(c, ActivityKind::Call, "C1");
        crm.add_scoring_rule(ScoringRule {
            name: "Active".into(),
            points: 20,
            condition: ScoringCondition::ActivityCountGte(5),
        });
        assert_eq!(crm.compute_lead_score(c), 0);
    }

    #[test]
    fn test_compute_score_deal_value() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "Big", 100_000.0);
        crm.add_scoring_rule(ScoringRule {
            name: "BigDeal".into(),
            points: 30,
            condition: ScoringCondition::DealValueGte(50_000.0),
        });
        assert_eq!(crm.compute_lead_score(c), 30);
    }

    #[test]
    fn test_compute_score_email_domain() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@enterprise.com");
        crm.add_scoring_rule(ScoringRule {
            name: "Enterprise".into(),
            points: 25,
            condition: ScoringCondition::EmailDomainContains("enterprise".into()),
        });
        assert_eq!(crm.compute_lead_score(c), 25);
    }

    #[test]
    fn test_compute_score_multiple_rules() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@enterprise.com");
        crm.add_tag_to_contact(c, "vip");
        crm.add_scoring_rule(ScoringRule {
            name: "VIP".into(),
            points: 10,
            condition: ScoringCondition::HasTag("vip".into()),
        });
        crm.add_scoring_rule(ScoringRule {
            name: "Enterprise".into(),
            points: 25,
            condition: ScoringCondition::EmailDomainContains("enterprise".into()),
        });
        assert_eq!(crm.compute_lead_score(c), 35);
    }

    #[test]
    fn test_compute_score_invalid_contact() {
        let crm = make_crm();
        assert_eq!(crm.compute_lead_score(Id(999)), 0);
    }

    #[test]
    fn test_apply_scoring() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_tag_to_contact(c, "hot");
        crm.add_scoring_rule(ScoringRule {
            name: "Hot".into(),
            points: 50,
            condition: ScoringCondition::HasTag("hot".into()),
        });
        crm.apply_scoring();
        assert_eq!(crm.get_contact(c).unwrap().lead_score, 50);
    }

    #[test]
    fn test_top_contacts_by_score() {
        let mut crm = make_crm();
        let a = crm.add_contact("A", "a@a.com");
        let b = crm.add_contact("B", "b@b.com");
        let c = crm.add_contact("C", "c@c.com");
        crm.get_contact_mut(a).unwrap().lead_score = 10;
        crm.get_contact_mut(b).unwrap().lead_score = 50;
        crm.get_contact_mut(c).unwrap().lead_score = 30;
        let top = crm.top_contacts_by_score(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].name, "B");
        assert_eq!(top[1].name, "C");
    }

    // --- RFM tests ---

    #[test]
    fn test_rfm_no_deals() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        // No deals -> last_closed=0, now=86400*400 -> days_since=400 -> recency=1
        let rfm = crm.compute_rfm(c, 86400 * 400);
        assert_eq!(rfm.recency, 1);
        assert_eq!(rfm.frequency, 1);
        assert_eq!(rfm.monetary, 1);
    }

    #[test]
    fn test_rfm_champion() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        for i in 0..10 {
            let d = crm.add_deal(c, &format!("D{i}"), 20_000.0);
            crm.advance_deal(d, DealStage::ClosedWon);
            crm.get_deal_mut(d).unwrap().closed_at = Some(1_000_000);
        }
        let rfm = crm.compute_rfm(c, 1_000_000 + 86400 * 10);
        assert_eq!(rfm.segment(), RfmSegment::Champion);
    }

    #[test]
    fn test_rfm_loyal() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        for i in 0..5 {
            let d = crm.add_deal(c, &format!("D{i}"), 3_000.0);
            crm.advance_deal(d, DealStage::ClosedWon);
            crm.get_deal_mut(d).unwrap().closed_at = Some(1_000_000);
        }
        let rfm = crm.compute_rfm(c, 1_000_000 + 86400 * 60);
        assert_eq!(rfm.segment(), RfmSegment::Loyal);
    }

    #[test]
    fn test_rfm_at_risk() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        let d = crm.add_deal(c, "D", 500.0);
        crm.advance_deal(d, DealStage::ClosedWon);
        crm.get_deal_mut(d).unwrap().closed_at = Some(0);
        let rfm = crm.compute_rfm(c, 86400 * 300);
        assert_eq!(rfm.segment(), RfmSegment::AtRisk);
    }

    #[test]
    fn test_rfm_lost() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        let rfm = crm.compute_rfm(c, 86400 * 500);
        assert_eq!(rfm.segment(), RfmSegment::Lost);
    }

    #[test]
    fn test_rfm_score_total() {
        let score = RfmScore {
            recency: 5,
            frequency: 4,
            monetary: 3,
        };
        assert_eq!(score.total(), 12);
    }

    #[test]
    fn test_segment_contact() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        assert_eq!(crm.segment_contact(c, 86400 * 500), RfmSegment::Lost);
    }

    #[test]
    fn test_contacts_with_segment() {
        let mut crm = make_crm();
        let a = crm.add_contact("A", "a@a.com");
        crm.add_contact("B", "b@b.com");
        for i in 0..10 {
            let d = crm.add_deal(a, &format!("D{i}"), 20_000.0);
            crm.advance_deal(d, DealStage::ClosedWon);
            crm.get_deal_mut(d).unwrap().closed_at = Some(100);
        }
        let champs = crm.contacts_with_segment(RfmSegment::Champion, 100 + 86400);
        assert_eq!(champs.len(), 1);
        assert_eq!(champs[0].name, "A");
    }

    #[test]
    fn test_rfm_potential() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        for i in 0..3 {
            let d = crm.add_deal(c, &format!("D{i}"), 5_000.0);
            crm.advance_deal(d, DealStage::ClosedWon);
            crm.get_deal_mut(d).unwrap().closed_at = Some(1_000_000);
        }
        let rfm = crm.compute_rfm(c, 1_000_000 + 86400 * 100);
        assert_eq!(rfm.segment(), RfmSegment::Potential);
    }

    // --- Funnel metrics tests ---

    #[test]
    fn test_funnel_metrics_empty() {
        let crm = make_crm();
        let m = crm.funnel_metrics();
        assert!(m.total_pipeline_value.abs() < f64::EPSILON);
        assert!(m.win_rate.abs() < f64::EPSILON);
        assert!(m.average_deal_value.abs() < f64::EPSILON);
    }

    #[test]
    fn test_funnel_metrics_with_deals() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D1", 1000.0);
        let d2 = crm.add_deal(c, "D2", 2000.0);
        crm.advance_deal(d2, DealStage::ClosedWon);
        let m = crm.funnel_metrics();
        assert!((m.total_pipeline_value - 1000.0).abs() < f64::EPSILON);
        assert!((m.win_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_funnel_win_rate() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        let d1 = crm.add_deal(c, "Won", 1000.0);
        let d2 = crm.add_deal(c, "Lost", 2000.0);
        crm.advance_deal(d1, DealStage::ClosedWon);
        crm.advance_deal(d2, DealStage::ClosedLost);
        let m = crm.funnel_metrics();
        assert!((m.win_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_funnel_average_deal_value() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D1", 1000.0);
        crm.add_deal(c, "D2", 3000.0);
        let m = crm.funnel_metrics();
        assert!((m.average_deal_value - 2000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_funnel_weighted_pipeline() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D1", 10_000.0);
        let m = crm.funnel_metrics();
        // Prospect default probability = 0.10
        assert!((m.weighted_pipeline_value - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_funnel_stage_counts() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D1", 100.0);
        crm.add_deal(c, "D2", 200.0);
        let d3 = crm.add_deal(c, "D3", 300.0);
        crm.advance_deal(d3, DealStage::Qualified);
        let m = crm.funnel_metrics();
        assert_eq!(
            m.stage_counts
                .get(&DealStage::Prospect)
                .copied()
                .unwrap_or(0),
            2
        );
        assert_eq!(
            m.stage_counts
                .get(&DealStage::Qualified)
                .copied()
                .unwrap_or(0),
            1
        );
    }

    #[test]
    fn test_funnel_conversion_rates() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D1", 100.0);
        crm.add_deal(c, "D2", 200.0);
        let d3 = crm.add_deal(c, "D3", 300.0);
        crm.advance_deal(d3, DealStage::Qualified);
        let m = crm.funnel_metrics();
        // 2 prospect, 1 qualified -> prospect->qualified = 0.5
        let rate = m
            .conversion_rates
            .get(&DealStage::Prospect)
            .copied()
            .unwrap_or(0.0);
        assert!((rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_conversion_rate() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D1", 100.0);
        let d2 = crm.add_deal(c, "D2", 200.0);
        crm.advance_deal(d2, DealStage::Proposal);
        let rate = crm.conversion_rate(DealStage::Prospect, DealStage::Proposal);
        assert!((rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_conversion_rate_empty() {
        let crm = make_crm();
        assert!(
            crm.conversion_rate(DealStage::Prospect, DealStage::ClosedWon)
                .abs()
                < f64::EPSILON
        );
    }

    // --- Active stages ---

    #[test]
    fn test_active_stages() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D", 100.0);
        let stages = crm.active_stages();
        assert_eq!(stages, vec![DealStage::Prospect]);
    }

    #[test]
    fn test_active_stages_multiple() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D1", 100.0);
        let d2 = crm.add_deal(c, "D2", 200.0);
        crm.advance_deal(d2, DealStage::ClosedWon);
        let stages = crm.active_stages();
        assert!(stages.contains(&DealStage::Prospect));
        assert!(stages.contains(&DealStage::ClosedWon));
    }

    // --- Default ---

    #[test]
    fn test_crm_default() {
        let crm = Crm::default();
        assert_eq!(crm.contact_count(), 0);
        assert_eq!(crm.deal_count(), 0);
    }

    // --- Id ---

    #[test]
    fn test_id_display() {
        let id = Id(42);
        assert_eq!(format!("{id}"), "42");
    }

    #[test]
    fn test_id_value() {
        let id = Id(7);
        assert_eq!(id.value(), 7);
    }

    // --- DealStage ordinal ---

    #[test]
    fn test_stage_ordinal() {
        assert_eq!(DealStage::Prospect.ordinal(), 0);
        assert_eq!(DealStage::ClosedLost.ordinal(), 5);
    }

    #[test]
    fn test_stage_probabilities() {
        assert!((DealStage::Qualified.default_probability() - 0.25).abs() < f64::EPSILON);
        assert!((DealStage::Proposal.default_probability() - 0.50).abs() < f64::EPSILON);
        assert!((DealStage::Negotiation.default_probability() - 0.75).abs() < f64::EPSILON);
    }

    // --- Edge cases ---

    #[test]
    fn test_get_deal_not_found() {
        let crm = make_crm();
        assert!(crm.get_deal(Id(999)).is_none());
    }

    #[test]
    fn test_contacts_by_tag_empty() {
        let crm = make_crm();
        assert!(crm.contacts_by_tag("any").is_empty());
    }

    #[test]
    fn test_notes_for_contact_empty() {
        let crm = make_crm();
        assert!(crm.notes_for_contact(Id(1)).is_empty());
    }

    #[test]
    fn test_activities_for_contact_empty() {
        let crm = make_crm();
        assert!(crm.activities_for_contact(Id(1)).is_empty());
    }

    #[test]
    fn test_remove_contact_field_invalid_contact() {
        let mut crm = make_crm();
        assert!(!crm.remove_contact_field(Id(999), "k"));
    }

    #[test]
    fn test_remove_tag_invalid_contact() {
        let mut crm = make_crm();
        assert!(!crm.remove_tag_from_contact(Id(999), "x"));
    }

    #[test]
    fn test_field_value_clone() {
        let v = FieldValue::Text("hello".into());
        let v2 = v.clone();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_deal_stage_all_count() {
        assert_eq!(DealStage::ALL.len(), 6);
    }

    #[test]
    fn test_multiple_contacts_scoring() {
        let mut crm = make_crm();
        let a = crm.add_contact("A", "a@a.com");
        let b = crm.add_contact("B", "b@b.com");
        crm.add_tag_to_contact(a, "hot");
        crm.add_scoring_rule(ScoringRule {
            name: "Hot".into(),
            points: 10,
            condition: ScoringCondition::HasTag("hot".into()),
        });
        crm.apply_scoring();
        assert_eq!(crm.get_contact(a).unwrap().lead_score, 10);
        assert_eq!(crm.get_contact(b).unwrap().lead_score, 0);
    }

    #[test]
    fn test_funnel_stage_values() {
        let mut crm = make_crm();
        let c = crm.add_contact("A", "a@a.com");
        crm.add_deal(c, "D1", 500.0);
        crm.add_deal(c, "D2", 700.0);
        let m = crm.funnel_metrics();
        let val = m
            .stage_values
            .get(&DealStage::Prospect)
            .copied()
            .unwrap_or(0.0);
        assert!((val - 1200.0).abs() < f64::EPSILON);
    }
}
