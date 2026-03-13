**English** | [日本語](README_JP.md)

# ALICE-CRM

Customer Relationship Management module for the ALICE ecosystem. Pure Rust implementation with zero external dependencies.

## Overview

| Item | Value |
|------|-------|
| **Crate** | `alice-crm` |
| **Version** | 1.0.0 |
| **License** | AGPL-3.0 |
| **Edition** | 2021 |

## Features

- **Contact Management** — Create, update, and query contacts with email, phone, company, and custom fields
- **Deal Pipeline** — Multi-stage deal tracking with probability and value
- **Lead Scoring** — Rule-based scoring system for prioritizing prospects
- **RFM Segmentation** — Recency, Frequency, Monetary analysis for customer segmentation
- **Activity Tracking** — Log calls, emails, meetings, and tasks per contact
- **Notes & Tags** — Attach notes and tags to contacts for flexible organization
- **Custom Fields** — Extend contact records with Text, Number, or Bool fields
- **Funnel Analytics** — Stage-by-stage conversion rate and pipeline metrics

## Architecture

```
alice-crm (lib.rs — single-file crate)
├── Contact / Id / FieldValue     # Core data models
├── Deal / DealStage              # Pipeline management
├── Activity / ActivityKind       # Activity tracking
├── Note                          # Contact notes
├── RfmScore / RfmSegment         # Customer segmentation
└── Crm                           # Top-level engine (contacts, deals, funnel)
```

## Quick Start

```rust
use alice_crm::Crm;

let mut crm = Crm::new();
let contact_id = crm.add_contact("Alice", "alice@example.com");
crm.add_tag(contact_id, "vip");
let deal_id = crm.add_deal(contact_id, "Enterprise License", 50_000);
```

## Build

```bash
cargo build
cargo test
cargo clippy -- -W clippy::all
```

## License

AGPL-3.0 -- see [LICENSE](LICENSE) for details.
