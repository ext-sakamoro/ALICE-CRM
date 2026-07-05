//! engine.

use crate::activity::{Activity, ActivityKind};
use crate::contact::*;
use crate::deal::*;
use crate::field::FieldValue;
use crate::funnel::*;
use crate::id_gen::*;
use crate::lead_score::*;
use crate::note::Note;
use crate::rfm::*;
use std::collections::HashMap;

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
