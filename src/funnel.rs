//! funnel.

use crate::deal::DealStage;
use std::collections::HashMap;

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
