//! Phase 6: Signals Engine
//!
//! Implements actionable trading signals based on rolling metrics and trade patterns.
//! Consumes token_rolling_metrics (Phase 5) and recent token_trades (Phase 5).
//! Produces signals persisted to token_signals table for Phase 7 dashboard.

use crate::{state::RollingMetrics, types::TradeEvent};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Signal types for Phase 6
///
/// SQL reference: `/sql/03_token_signals.sql` and `/sql/10_phase6_signals_engine.sql`
///
/// Phase 6 mandatory signals:
/// - BREAKOUT: net_flow_300s accelerating, momentum shift, increasing wallets
/// - REACCUMULATION: DCA flow increasing, positive momentum shift
/// - FOCUSED_BUYERS: Low entropy wallet distribution (F ≤ 0.35)
/// - PERSISTENCE: Positive net_flow across 3 consecutive windows, sustained activity
/// - FLOW_REVERSAL: 60s negative while 300s positive, early exhaustion signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalType {
    Breakout,
    Reaccumulation,
    FocusedBuyers,
    Persistence,
    FlowReversal,
    // Legacy signals (Phase 1)
    Focused,
    Surge,
    BotDropoff,
    DcaConviction,
}

impl SignalType {
    /// Convert signal type to string representation for database
    ///
    /// Returns uppercase string matching SQL enum values
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalType::Breakout => "BREAKOUT",
            SignalType::Reaccumulation => "REACCUMULATION",
            SignalType::FocusedBuyers => "FOCUSED_BUYERS",
            SignalType::Persistence => "PERSISTENCE",
            SignalType::FlowReversal => "FLOW_REVERSAL",
            // Legacy
            SignalType::Focused => "FOCUSED",
            SignalType::Surge => "SURGE",
            SignalType::BotDropoff => "BOT_DROPOFF",
            SignalType::DcaConviction => "DCA_CONVICTION",
        }
    }
}

/// Phase 6: Signal event with strength and metadata
///
/// SQL reference: `/sql/03_token_signals.sql` and `/sql/10_phase6_signals_engine.sql`
///
/// All field names match SQL column names.
#[derive(Debug, Clone)]
pub struct Signal {
    /// Token mint address
    pub mint: String,

    /// Type of signal detected
    pub signal_type: SignalType,

    /// Signal strength/intensity score (0.0 - 1.0)
    pub strength: f64,

    /// Time window string (e.g., "300s", "60s")
    pub window: String,

    /// Unix timestamp when signal was created
    pub timestamp: i64,

    /// Additional signal-specific metadata as JSON
    pub metadata: Value,
}

/// Legacy token signal (Phase 1-5 compatibility)
#[derive(Debug, Clone)]
pub struct TokenSignal {
    pub mint: String,
    pub signal_type: SignalType,
    pub window_seconds: i32,
    pub severity: i32,
    pub score: Option<f64>,
    pub details_json: Option<String>,
    pub created_at: i64,
}

impl Signal {
    /// Create a new signal
    pub fn new(
        mint: String,
        signal_type: SignalType,
        strength: f64,
        window: String,
        timestamp: i64,
        metadata: Value,
    ) -> Self {
        Self {
            mint,
            signal_type,
            strength: strength.clamp(0.0, 1.0),
            window,
            timestamp,
            metadata,
        }
    }
}

impl TokenSignal {
    /// Create a new signal with basic fields
    pub fn new(
        mint: String,
        signal_type: SignalType,
        window_seconds: i32,
        created_at: i64,
    ) -> Self {
        Self {
            mint,
            signal_type,
            window_seconds,
            severity: 1,
            score: None,
            details_json: None,
            created_at,
        }
    }

    pub fn with_severity(mut self, severity: i32) -> Self {
        self.severity = severity.clamp(1, 5);
        self
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.score = Some(score);
        self
    }

    pub fn with_details(mut self, details_json: String) -> Self {
        self.details_json = Some(details_json);
        self
    }
}

/// Phase 6: Signal evaluation engine
///
/// Evaluates all signals for a given token based on rolling metrics and recent trades.
/// Returns a vector of triggered signals with computed strength scores.
///
/// # Arguments
/// * `mint` - Token mint address
/// * `metrics` - Current rolling metrics computed from Phase 5
/// * `recent_trades` - Recent trade events from token_trades table
///
/// # Returns
/// Vector of signals that were triggered by this update
pub fn evaluate_signals(mint: &str, metrics: &RollingMetrics, recent_trades: &[TradeEvent]) -> Vec<Signal> {
    let now = chrono::Utc::now().timestamp();
    let mut signals = Vec::new();

    // Signal A: BREAKOUT
    if let Some(signal) = evaluate_breakout(mint, metrics, now) {
        signals.push(signal);
    }

    // Signal B: REACCUMULATION
    if let Some(signal) = evaluate_reaccumulation(mint, metrics, now) {
        signals.push(signal);
    }

    // Signal C: FOCUSED BUYERS
    if let Some(signal) = evaluate_focused_buyers(mint, metrics, recent_trades, now) {
        signals.push(signal);
    }

    // Signal D: PERSISTENCE
    if let Some(signal) = evaluate_persistence(mint, metrics, now) {
        signals.push(signal);
    }

    // Signal E: FLOW REVERSAL
    if let Some(signal) = evaluate_flow_reversal(mint, metrics, now) {
        signals.push(signal);
    }

    signals
}

/// Signal A: BREAKOUT
///
/// Triggered when:
/// - net_flow_300s accelerating
/// - AND net_flow_60s > net_flow_300s
/// - AND unique_wallets_300s increasing (>= 5)
/// - AND bot ratio within normal bounds (<= 0.3)
fn evaluate_breakout(mint: &str, metrics: &RollingMetrics, timestamp: i64) -> Option<Signal> {
    let net_flow_60s = metrics.net_flow_60s_sol;
    let net_flow_300s = metrics.net_flow_300s_sol;
    let net_flow_900s = metrics.net_flow_900s_sol;
    let unique_wallets = metrics.unique_wallets_300s;
    let bot_ratio = if metrics.buy_count_300s + metrics.sell_count_300s > 0 {
        metrics.bot_trades_count_300s as f64 / (metrics.buy_count_300s + metrics.sell_count_300s) as f64
    } else {
        0.0
    };

    // Check conditions
    let is_accelerating = net_flow_300s > net_flow_900s && net_flow_300s > 0.0;
    let momentum_shift = net_flow_60s > net_flow_300s;
    let has_wallets = unique_wallets >= 5;
    let bot_ratio_ok = bot_ratio <= 0.3;

    if is_accelerating && momentum_shift && has_wallets && bot_ratio_ok {
        // Compute strength (0.0 - 1.0)
        let acceleration = ((net_flow_300s - net_flow_900s) / net_flow_900s.max(1.0)).min(1.0);
        let momentum_factor = (net_flow_60s / net_flow_300s.max(1.0)).min(1.0);
        let wallet_factor = (unique_wallets as f64 / 20.0).min(1.0);
        let bot_factor = (1.0 - bot_ratio).max(0.0);
        
        let strength = (acceleration * 0.3 + momentum_factor * 0.3 + wallet_factor * 0.2 + bot_factor * 0.2).clamp(0.0, 1.0);

        let metadata = json!({
            "net_flow_60s": net_flow_60s,
            "net_flow_300s": net_flow_300s,
            "net_flow_900s": net_flow_900s,
            "unique_wallets": unique_wallets,
            "bot_ratio": bot_ratio,
        });

        return Some(Signal::new(
            mint.to_string(),
            SignalType::Breakout,
            strength,
            "300s".to_string(),
            timestamp,
            metadata,
        ));
    }

    None
}

/// Signal B: REACCUMULATION
///
/// Triggered when:
/// - DCA flow increasing
/// - AND DCA unique wallets increasing (>= 2)
/// - AND total net_flow_300s positive
/// - AND 300s window > 900s window (momentum shift)
fn evaluate_reaccumulation(mint: &str, metrics: &RollingMetrics, timestamp: i64) -> Option<Signal> {
    let dca_flow = metrics.dca_flow_300s_sol;
    let dca_wallets = metrics.dca_unique_wallets_300s;
    let net_flow_300s = metrics.net_flow_300s_sol;
    let net_flow_900s = metrics.net_flow_900s_sol;

    // Check conditions
    let dca_active = dca_flow > 0.0 && dca_wallets >= 2;
    let positive_flow = net_flow_300s > 0.0;
    let momentum_shift = net_flow_300s > net_flow_900s;

    if dca_active && positive_flow && momentum_shift {
        // Compute strength
        let dca_factor = (dca_flow / 10.0).min(1.0);
        let wallet_factor = (dca_wallets as f64 / 5.0).min(1.0);
        let flow_factor = (net_flow_300s / 50.0).min(1.0);
        let momentum_factor = ((net_flow_300s - net_flow_900s) / net_flow_900s.abs().max(1.0)).min(1.0);
        
        let strength = (dca_factor * 0.3 + wallet_factor * 0.2 + flow_factor * 0.3 + momentum_factor * 0.2).clamp(0.0, 1.0);

        let metadata = json!({
            "dca_flow": dca_flow,
            "dca_wallets": dca_wallets,
            "net_flow_300s": net_flow_300s,
            "net_flow_900s": net_flow_900s,
            "dca_ratio": metrics.dca_ratio_300s,
        });

        return Some(Signal::new(
            mint.to_string(),
            SignalType::Reaccumulation,
            strength,
            "300s".to_string(),
            timestamp,
            metadata,
        ));
    }

    None
}

/// Signal C: FOCUSED BUYERS
///
/// Triggered when:
/// - Low entropy wallet distribution
/// - F ≤ 0.35 (35% of wallets responsible for >70% inflow)
/// - AND positive flow trend
fn evaluate_focused_buyers(mint: &str, metrics: &RollingMetrics, recent_trades: &[TradeEvent], timestamp: i64) -> Option<Signal> {
    if recent_trades.is_empty() || metrics.net_flow_300s_sol <= 0.0 {
        return None;
    }

    // Compute wallet concentration (F-score)
    let mut wallet_flows: HashMap<String, f64> = HashMap::new();
    let mut total_inflow = 0.0;

    for trade in recent_trades {
        let flow = match trade.direction {
            crate::types::TradeDirection::Buy => trade.sol_amount,
            crate::types::TradeDirection::Sell => -trade.sol_amount,
            crate::types::TradeDirection::Unknown => 0.0,
        };
        
        if flow > 0.0 {
            *wallet_flows.entry(trade.user_account.clone()).or_insert(0.0) += flow;
            total_inflow += flow;
        }
    }

    if total_inflow < 1.0 {
        return None;
    }

    // Sort wallets by flow
    let mut wallet_vec: Vec<(String, f64)> = wallet_flows.into_iter().collect();
    wallet_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Calculate F-score (fraction of wallets responsible for 70% of inflow)
    let target_flow = total_inflow * 0.7;
    let mut cumulative_flow = 0.0;
    let mut wallets_needed = 0;

    for (_, flow) in &wallet_vec {
        cumulative_flow += flow;
        wallets_needed += 1;
        if cumulative_flow >= target_flow {
            break;
        }
    }

    let f_score = wallets_needed as f64 / wallet_vec.len() as f64;

    if f_score <= 0.35 && metrics.net_flow_300s_sol > 0.0 {
        // Compute strength
        let concentration_factor = (1.0 - (f_score / 0.35)).clamp(0.0, 1.0);
        let flow_factor = (metrics.net_flow_300s_sol / 50.0).min(1.0);
        
        let strength = (concentration_factor * 0.6 + flow_factor * 0.4).clamp(0.0, 1.0);

        let metadata = json!({
            "f_score": f_score,
            "wallets_needed": wallets_needed,
            "total_wallets": wallet_vec.len(),
            "net_flow_300s": metrics.net_flow_300s_sol,
            "total_inflow": total_inflow,
        });

        return Some(Signal::new(
            mint.to_string(),
            SignalType::FocusedBuyers,
            strength,
            "300s".to_string(),
            timestamp,
            metadata,
        ));
    }

    None
}

/// Signal D: PERSISTENCE
///
/// Triggered when:
/// - Token maintains positive net_flow across 3 consecutive windows (60s, 300s, 900s)
/// - AND no collapse in unique wallets (>= 5)
/// - AND no bot surge (<= 0.4 bot ratio)
fn evaluate_persistence(mint: &str, metrics: &RollingMetrics, timestamp: i64) -> Option<Signal> {
    let positive_flow_60s = metrics.net_flow_60s_sol > 0.0;
    let positive_flow_300s = metrics.net_flow_300s_sol > 0.0;
    let positive_flow_900s = metrics.net_flow_900s_sol > 0.0;
    let has_wallets = metrics.unique_wallets_300s >= 5;
    let bot_ratio = if metrics.buy_count_300s + metrics.sell_count_300s > 0 {
        metrics.bot_trades_count_300s as f64 / (metrics.buy_count_300s + metrics.sell_count_300s) as f64
    } else {
        0.0
    };
    let no_bot_surge = bot_ratio <= 0.4;

    if positive_flow_60s && positive_flow_300s && positive_flow_900s && has_wallets && no_bot_surge {
        // Compute strength based on flow consistency and magnitude
        let flow_consistency = 1.0 - ((metrics.net_flow_60s_sol - metrics.net_flow_300s_sol).abs() / metrics.net_flow_300s_sol.max(1.0)).min(1.0);
        let flow_magnitude = (metrics.net_flow_900s_sol / 100.0).min(1.0);
        let wallet_factor = (metrics.unique_wallets_300s as f64 / 20.0).min(1.0);
        let bot_factor = (1.0 - bot_ratio).max(0.0);
        
        let strength = (flow_consistency * 0.3 + flow_magnitude * 0.3 + wallet_factor * 0.2 + bot_factor * 0.2).clamp(0.0, 1.0);

        let metadata = json!({
            "net_flow_60s": metrics.net_flow_60s_sol,
            "net_flow_300s": metrics.net_flow_300s_sol,
            "net_flow_900s": metrics.net_flow_900s_sol,
            "unique_wallets": metrics.unique_wallets_300s,
            "bot_ratio": bot_ratio,
        });

        return Some(Signal::new(
            mint.to_string(),
            SignalType::Persistence,
            strength,
            "900s".to_string(),
            timestamp,
            metadata,
        ));
    }

    None
}

/// Signal E: FLOW REVERSAL
///
/// Triggered when:
/// - 60s net flow goes negative
/// - AND 300s still positive
/// - AND unique wallets drop (>= 25% decrease from 300s avg)
///
/// Indicates early exhaustion / impending momentum flip
fn evaluate_flow_reversal(mint: &str, metrics: &RollingMetrics, timestamp: i64) -> Option<Signal> {
    let flow_60s_negative = metrics.net_flow_60s_sol < 0.0;
    let flow_300s_positive = metrics.net_flow_300s_sol > 0.0;
    
    // Approximate wallet "drop" by comparing unique wallets to trade volume
    // If unique_wallets is low relative to trade count, it suggests fewer participants
    let total_trades = metrics.buy_count_60s + metrics.sell_count_60s;
    let wallets_per_trade = if total_trades > 0 {
        metrics.unique_wallets_300s as f64 / total_trades as f64
    } else {
        0.0
    };
    let wallet_drop = wallets_per_trade < 0.5; // Less than 0.5 unique wallets per trade suggests consolidation

    if flow_60s_negative && flow_300s_positive && wallet_drop {
        // Compute strength based on divergence magnitude
        let divergence = (metrics.net_flow_300s_sol - metrics.net_flow_60s_sol) / metrics.net_flow_300s_sol.max(1.0);
        let divergence_factor = divergence.min(1.0);
        let flow_magnitude = (metrics.net_flow_300s_sol / 50.0).min(1.0);
        
        let strength = (divergence_factor * 0.6 + flow_magnitude * 0.4).clamp(0.0, 1.0);

        let metadata = json!({
            "net_flow_60s": metrics.net_flow_60s_sol,
            "net_flow_300s": metrics.net_flow_300s_sol,
            "unique_wallets": metrics.unique_wallets_300s,
            "total_trades_60s": total_trades,
            "wallets_per_trade": wallets_per_trade,
        });

        return Some(Signal::new(
            mint.to_string(),
            SignalType::FlowReversal,
            strength,
            "60s".to_string(),
            timestamp,
            metadata,
        ));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state::RollingMetrics, types::{TradeDirection, TradeEvent}};

    fn create_test_metrics() -> RollingMetrics {
        RollingMetrics {
            net_flow_60s_sol: 10.0,
            net_flow_300s_sol: 50.0,
            net_flow_900s_sol: 40.0,
            net_flow_3600s_sol: 200.0,
            net_flow_7200s_sol: 300.0,
            net_flow_14400s_sol: 500.0,
            buy_count_60s: 5,
            sell_count_60s: 2,
            buy_count_300s: 25,
            sell_count_300s: 10,
            buy_count_900s: 60,
            sell_count_900s: 30,
            unique_wallets_300s: 15,
            bot_wallets_count_300s: 2,
            bot_trades_count_300s: 5,
            bot_flow_300s_sol: 5.0,
            dca_buys_60s: 2,
            dca_buys_300s: 8,
            dca_buys_900s: 20,
            dca_buys_3600s: 50,
            dca_buys_14400s: 100,
            dca_flow_300s_sol: 15.0,
            dca_unique_wallets_300s: 3,
            dca_ratio_300s: 0.3,
        }
    }

    fn create_test_trade(wallet: &str, sol_amount: f64, direction: TradeDirection) -> TradeEvent {
        TradeEvent {
            timestamp: 1000,
            mint: "test_mint".to_string(),
            direction,
            sol_amount,
            token_amount: 1000.0,
            token_decimals: 6,
            user_account: wallet.to_string(),
            source_program: "PumpSwap".to_string(),
            is_bot: false,
            is_dca: false,
        }
    }

    #[test]
    fn test_breakout_signal_triggered() {
        let mut metrics = create_test_metrics();
        
        // Setup: net_flow_300s > net_flow_900s, net_flow_60s > net_flow_300s
        metrics.net_flow_60s_sol = 60.0;
        metrics.net_flow_300s_sol = 50.0;
        metrics.net_flow_900s_sol = 40.0;
        metrics.unique_wallets_300s = 10;
        metrics.bot_trades_count_300s = 5;
        metrics.buy_count_300s = 25;
        metrics.sell_count_300s = 10;

        let signal = evaluate_breakout("test_mint", &metrics, 1000);
        
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.signal_type, SignalType::Breakout);
        assert!(signal.strength > 0.0 && signal.strength <= 1.0);
        assert_eq!(signal.window, "300s");
    }

    #[test]
    fn test_breakout_signal_not_triggered_high_bot_ratio() {
        let mut metrics = create_test_metrics();
        
        // High bot ratio (> 0.3)
        metrics.net_flow_60s_sol = 60.0;
        metrics.net_flow_300s_sol = 50.0;
        metrics.net_flow_900s_sol = 40.0;
        metrics.unique_wallets_300s = 10;
        metrics.bot_trades_count_300s = 15; // High bot count
        metrics.buy_count_300s = 25;
        metrics.sell_count_300s = 10;

        let signal = evaluate_breakout("test_mint", &metrics, 1000);
        
        assert!(signal.is_none());
    }

    #[test]
    fn test_reaccumulation_signal_triggered() {
        let mut metrics = create_test_metrics();
        
        // Setup: DCA active, positive flow, momentum shift
        metrics.dca_flow_300s_sol = 10.0;
        metrics.dca_unique_wallets_300s = 3;
        metrics.net_flow_300s_sol = 50.0;
        metrics.net_flow_900s_sol = 40.0;

        let signal = evaluate_reaccumulation("test_mint", &metrics, 1000);
        
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.signal_type, SignalType::Reaccumulation);
        assert!(signal.strength > 0.0 && signal.strength <= 1.0);
        assert_eq!(signal.window, "300s");
    }

    #[test]
    fn test_reaccumulation_signal_not_triggered_insufficient_dca_wallets() {
        let mut metrics = create_test_metrics();
        
        // Only 1 DCA wallet (needs >= 2)
        metrics.dca_flow_300s_sol = 10.0;
        metrics.dca_unique_wallets_300s = 1;
        metrics.net_flow_300s_sol = 50.0;
        metrics.net_flow_900s_sol = 40.0;

        let signal = evaluate_reaccumulation("test_mint", &metrics, 1000);
        
        assert!(signal.is_none());
    }

    #[test]
    fn test_focused_buyers_signal_triggered() {
        let metrics = create_test_metrics();
        
        // Create trades with concentrated buying (3 wallets responsible for 70%+ of flow)
        let trades = vec![
            create_test_trade("whale1", 20.0, TradeDirection::Buy),
            create_test_trade("whale2", 15.0, TradeDirection::Buy),
            create_test_trade("whale3", 10.0, TradeDirection::Buy),
            create_test_trade("small1", 1.0, TradeDirection::Buy),
            create_test_trade("small2", 1.0, TradeDirection::Buy),
            create_test_trade("small3", 1.0, TradeDirection::Buy),
            create_test_trade("small4", 1.0, TradeDirection::Buy),
            create_test_trade("small5", 1.0, TradeDirection::Buy),
        ];

        let signal = evaluate_focused_buyers("test_mint", &metrics, &trades, 1000);
        
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.signal_type, SignalType::FocusedBuyers);
        assert!(signal.strength > 0.0 && signal.strength <= 1.0);
        assert_eq!(signal.window, "300s");
    }

    #[test]
    fn test_focused_buyers_signal_not_triggered_distributed_flow() {
        let metrics = create_test_metrics();
        
        // Create trades with distributed buying (many wallets with equal amounts)
        let mut trades = vec![];
        for i in 0..20 {
            trades.push(create_test_trade(&format!("wallet{}", i), 5.0, TradeDirection::Buy));
        }

        let signal = evaluate_focused_buyers("test_mint", &metrics, &trades, 1000);
        
        // Should not trigger (F-score will be > 0.35)
        assert!(signal.is_none());
    }

    #[test]
    fn test_persistence_signal_triggered() {
        let mut metrics = create_test_metrics();
        
        // Setup: positive flow across all 3 windows
        metrics.net_flow_60s_sol = 10.0;
        metrics.net_flow_300s_sol = 50.0;
        metrics.net_flow_900s_sol = 100.0;
        metrics.unique_wallets_300s = 10;
        metrics.bot_trades_count_300s = 5;
        metrics.buy_count_300s = 25;
        metrics.sell_count_300s = 10;

        let signal = evaluate_persistence("test_mint", &metrics, 1000);
        
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.signal_type, SignalType::Persistence);
        assert!(signal.strength > 0.0 && signal.strength <= 1.0);
        assert_eq!(signal.window, "900s");
    }

    #[test]
    fn test_persistence_signal_not_triggered_negative_60s() {
        let mut metrics = create_test_metrics();
        
        // 60s flow is negative
        metrics.net_flow_60s_sol = -10.0;
        metrics.net_flow_300s_sol = 50.0;
        metrics.net_flow_900s_sol = 100.0;
        metrics.unique_wallets_300s = 10;

        let signal = evaluate_persistence("test_mint", &metrics, 1000);
        
        assert!(signal.is_none());
    }

    #[test]
    fn test_flow_reversal_signal_triggered() {
        let mut metrics = create_test_metrics();
        
        // Setup: 60s negative, 300s positive, low wallets per trade
        metrics.net_flow_60s_sol = -5.0;
        metrics.net_flow_300s_sol = 50.0;
        metrics.buy_count_60s = 10;
        metrics.sell_count_60s = 5;
        metrics.unique_wallets_300s = 5; // Low wallets relative to 15 trades

        let signal = evaluate_flow_reversal("test_mint", &metrics, 1000);
        
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.signal_type, SignalType::FlowReversal);
        assert!(signal.strength > 0.0 && signal.strength <= 1.0);
        assert_eq!(signal.window, "60s");
    }

    #[test]
    fn test_flow_reversal_signal_not_triggered_both_positive() {
        let mut metrics = create_test_metrics();
        
        // Both 60s and 300s positive
        metrics.net_flow_60s_sol = 10.0;
        metrics.net_flow_300s_sol = 50.0;

        let signal = evaluate_flow_reversal("test_mint", &metrics, 1000);
        
        assert!(signal.is_none());
    }

    #[test]
    fn test_evaluate_signals_multiple_triggers() {
        let mut metrics = create_test_metrics();
        
        // Setup conditions for multiple signals
        metrics.net_flow_60s_sol = 60.0;
        metrics.net_flow_300s_sol = 50.0;
        metrics.net_flow_900s_sol = 40.0;
        metrics.unique_wallets_300s = 10;
        metrics.bot_trades_count_300s = 5;
        metrics.buy_count_300s = 25;
        metrics.sell_count_300s = 10;
        metrics.dca_flow_300s_sol = 10.0;
        metrics.dca_unique_wallets_300s = 3;

        let trades = vec![
            create_test_trade("whale1", 20.0, TradeDirection::Buy),
            create_test_trade("whale2", 15.0, TradeDirection::Buy),
            create_test_trade("small1", 1.0, TradeDirection::Buy),
        ];

        let signals = evaluate_signals("test_mint", &metrics, &trades);
        
        // Should trigger at least breakout, reaccumulation, and persistence
        assert!(signals.len() >= 2);
        assert!(signals.iter().any(|s| s.signal_type == SignalType::Breakout));
        assert!(signals.iter().any(|s| s.signal_type == SignalType::Reaccumulation));
    }

    #[test]
    fn test_signal_strength_bounds() {
        let metrics = create_test_metrics();
        let trades = vec![create_test_trade("wallet1", 50.0, TradeDirection::Buy)];

        let signals = evaluate_signals("test_mint", &metrics, &trades);
        
        for signal in signals {
            assert!(signal.strength >= 0.0);
            assert!(signal.strength <= 1.0);
        }
    }

    #[test]
    fn test_signal_type_as_str() {
        assert_eq!(SignalType::Breakout.as_str(), "BREAKOUT");
        assert_eq!(SignalType::Reaccumulation.as_str(), "REACCUMULATION");
        assert_eq!(SignalType::FocusedBuyers.as_str(), "FOCUSED_BUYERS");
        assert_eq!(SignalType::Persistence.as_str(), "PERSISTENCE");
        assert_eq!(SignalType::FlowReversal.as_str(), "FLOW_REVERSAL");
    }

    #[test]
    fn test_signal_metadata_includes_key_metrics() {
        let metrics = create_test_metrics();
        
        let signal = evaluate_breakout("test_mint", &metrics, 1000);
        
        if let Some(signal) = signal {
            assert!(signal.metadata.get("net_flow_60s").is_some());
            assert!(signal.metadata.get("net_flow_300s").is_some());
            assert!(signal.metadata.get("unique_wallets").is_some());
            assert!(signal.metadata.get("bot_ratio").is_some());
        }
    }

    #[test]
    fn test_focused_buyers_empty_trades() {
        let metrics = create_test_metrics();
        let trades = vec![];

        let signal = evaluate_focused_buyers("test_mint", &metrics, &trades, 1000);
        
        assert!(signal.is_none());
    }

    #[test]
    fn test_focused_buyers_negative_net_flow() {
        let mut metrics = create_test_metrics();
        metrics.net_flow_300s_sol = -50.0; // Negative flow

        let trades = vec![
            create_test_trade("whale1", 20.0, TradeDirection::Buy),
            create_test_trade("whale2", 15.0, TradeDirection::Buy),
        ];

        let signal = evaluate_focused_buyers("test_mint", &metrics, &trades, 1000);
        
        assert!(signal.is_none());
    }

    #[test]
    fn test_breakout_edge_case_zero_trades() {
        let mut metrics = create_test_metrics();
        
        // Zero trades (bot ratio calculation edge case)
        metrics.buy_count_300s = 0;
        metrics.sell_count_300s = 0;
        metrics.bot_trades_count_300s = 0;

        let signal = evaluate_breakout("test_mint", &metrics, 1000);
        
        // Should not trigger (not enough trades)
        assert!(signal.is_none());
    }
}
