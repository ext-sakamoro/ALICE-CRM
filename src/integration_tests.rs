//! Integration tests.

#![allow(
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use crate::activity::*;
use crate::contact::*;
use crate::deal::*;
use crate::engine::*;
use crate::field::*;
use crate::funnel::*;
use crate::id_gen::*;
use crate::lead_score::*;
use crate::note::*;
use crate::rfm::*;
use std::collections::HashMap;

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
