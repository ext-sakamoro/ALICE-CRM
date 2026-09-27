[English](README.md) | **日本語**

# ALICE-CRM

ALICEエコシステムの顧客関係管理 (CRM) モジュール。外部依存なしの純Rust実装。

## 概要

| 項目 | 値 |
|------|-----|
| **クレート名** | `alice-crm` |
| **バージョン** | 1.0.0 |
| **ライセンス** | AGPL-3.0 |
| **エディション** | 2021 |

## 機能

- **コンタクト管理** — メール、電話、会社名、カスタムフィールド付きの連絡先作成・更新・検索
- **案件パイプライン** — 確度と金額を持つ多段階案件トラッキング
- **リードスコアリング** — ルールベースの見込み客優先度スコアリング
- **RFMセグメンテーション** — 最終購買日・購買頻度・購買金額による顧客分類
- **活動追跡** — 電話、メール、会議、タスクのコンタクト別ログ
- **ノート＆タグ** — コンタクトへのメモ付与とタグによる柔軟な分類
- **カスタムフィールド** — Text / Number / Bool 型でコンタクトレコードを拡張
- **ファネル分析** — ステージ別コンバージョン率とパイプライン指標

## アーキテクチャ

```
alice-crm (lib.rs — 単一ファイルクレート)
├── Contact / Id / FieldValue     # コアデータモデル
├── Deal / DealStage              # パイプライン管理
├── Activity / ActivityKind       # 活動追跡
├── Note                          # コンタクトメモ
├── RfmScore / RfmSegment         # 顧客セグメンテーション
└── Crm                           # トップレベルエンジン
```

## クイックスタート

```rust
use alice_crm::Crm;

let mut crm = Crm::new();
let contact_id = crm.add_contact("Alice", "alice@example.com");
crm.add_tag(contact_id, "vip");
let deal_id = crm.add_deal(contact_id, "Enterprise License", 50_000);
```

## ビルド

```bash
cargo build
cargo test
cargo clippy -- -W clippy::all
```

## ライセンス

`AGPL-3.0 OR LicenseRef-Commercial` — デュアルライセンス どちらかを選べる

| 選択肢 | 条文 | こういう時 |
|--------|------|-----------|
| **AGPL-3.0** | [LICENSE-AGPL](LICENSE-AGPL) — 無償、報告義務なし | 自分の project も AGPL 互換の OSS、または社内利用のみ |
| **商用ライセンス** | [LICENSE-COMMERCIAL.md](LICENSE-COMMERCIAL.md) — 有償、コピーレフト義務を解除 | クローズドソース製品 / 商用 SaaS / エッジ・ファームウェア配布 / plugin 再配布 / ソース開示を禁じるプラットフォーム NDA |

AGPL は強いコピーレフト: `alice-crm` を link して配布 / 提供する製品・ファームウェア・
サービスは AGPL で公開する義務がある これはオープンなエコシステムのための意図的な
選択で、それが実行できない場合のために商用ライセンスを用意している

商用ライセンスの問い合わせ: <contact@extoria.co.jp>
