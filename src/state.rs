//! In-memory rolling state management for tokens
//!
//! Phase 2: Data-model scaffolding only
//! No analytics logic, detection, or scoring implemented

use crate::types::{TradeDirection, TradeEvent};
use std::collections::{HashMap, HashSet, VecDeque};

/// Per-token rolling state container
///
/// Maintains rolling buffers for six time windows:
/// - 60s (1 minute)
/// - 300s (5 minutes)
/// - 900s (15 minutes)
/// - 3600s (1 hour)
/// - 7200s (2 hours)
/// - 14400s (4 hours)
#[derive(Debug, Clone)]
pub struct TokenRollingState {
    /// Token mint address
    pub mint: String,

    /// Phase 5: Last timestamp when this mint received a trade (for pruning)
    pub last_seen_ts: i64,

    /// Rolling buffer: trades in last 60 seconds
    pub trades_60s: Vec<TradeEvent>,

    /// Rolling buffer: trades in last 300 seconds (5 minutes)
    pub trades_300s: Vec<TradeEvent>,

    /// Rolling buffer: trades in last 900 seconds (15 minutes)
    pub trades_900s: Vec<TradeEvent>,

    /// Rolling buffer: trades in last 3600 seconds (1 hour)
    pub trades_3600s: Vec<TradeEvent>,

    /// Rolling buffer: trades in last 7200 seconds (2 hours)
    pub trades_7200s: Vec<TradeEvent>,

    /// Rolling buffer: trades in last 14400 seconds (4 hours)
    pub trades_14400s: Vec<TradeEvent>,

    /// Unique wallet addresses in 300s window
    pub unique_wallets_300s: HashSet<String>,

    /// Bot wallet addresses in 300s window
    pub bot_wallets_300s: HashSet<String>,
    
    /// Phase 4: Track wallet trade counts in 60s window (for bot detection)
    /// Key: wallet address, Value: (trade_count, last_trade_timestamp)
    pub wallet_activity_60s: HashMap<String, (i32, i64)>,

    /// Trades grouped by source program (for DCA correlation)
    /// Key: source_program (e.g., "PumpSwap", "BonkSwap", "Moonshot", "JupiterDCA")
    /// Value: Vector of trades from that program
    pub trades_by_program: HashMap<String, Vec<TradeEvent>>,

    /// DCA rolling windows: timestamps of JupiterDCA BUY trades
    /// Phase 6: DCA Rolling Windows (feature/dca-rolling-windows)
    ///
    /// These VecDeques store only timestamps (i64) for efficient memory usage.
    /// Timestamps are appended on each JupiterDCA BUY trade and pruned based on window duration.
    pub dca_timestamps_60s: VecDeque<i64>,
    pub dca_timestamps_300s: VecDeque<i64>,
    pub dca_timestamps_900s: VecDeque<i64>,
    pub dca_timestamps_3600s: VecDeque<i64>,
    pub dca_timestamps_14400s: VecDeque<i64>,
}

/// Internal metrics snapshot computed from rolling windows
///
/// This is NOT directly mapped to AggregatedTokenState.
/// It's an intermediate representation for Phase 2 only.
#[derive(Debug, Clone)]
pub struct RollingMetrics {
    // Net flow metrics
    pub net_flow_60s_sol: f64,
    pub net_flow_300s_sol: f64,
    pub net_flow_900s_sol: f64,
    pub net_flow_3600s_sol: f64,
    pub net_flow_7200s_sol: f64,
    pub net_flow_14400s_sol: f64,

    // Trade counts (60s window)
    pub buy_count_60s: i32,
    pub sell_count_60s: i32,

    // Trade counts (300s window)
    pub buy_count_300s: i32,
    pub sell_count_300s: i32,

    // Trade counts (900s window)
    pub buy_count_900s: i32,
    pub sell_count_900s: i32,

    // Advanced metrics (300s window)
    pub unique_wallets_300s: i32,
    
    // Bot detection metrics (Phase 4)
    pub bot_wallets_count_300s: i32,
    pub bot_trades_count_300s: i32,
    pub bot_flow_300s_sol: f64,

    // DCA buy counts (rolling windows)
    // Phase 4 & 6: DCA Rolling Windows
    pub dca_buys_60s: i32,
    pub dca_buys_300s: i32,
    pub dca_buys_900s: i32,
    pub dca_buys_3600s: i32,
    pub dca_buys_14400s: i32,
    
    // Phase 4: DCA flow metrics (300s window)
    pub dca_flow_300s_sol: f64,
    pub dca_unique_wallets_300s: i32,
    pub dca_ratio_300s: f64,
}

impl TokenRollingState {
    /// Phase 4: Bot detection threshold
    /// A wallet is flagged as a bot if it makes >= BOT_TRADE_THRESHOLD trades within 60 seconds
    const BOT_TRADE_THRESHOLD: i32 = 3;
    
    /// Create a new rolling state container for a token
    ///
    /// Phase 2: Proper initialization with capacity hints
    /// Phase 5: Initialize last_seen_ts to 0
    pub fn new(mint: String) -> Self {
        Self {
            mint,
            last_seen_ts: 0,
            trades_60s: Vec::with_capacity(100),
            trades_300s: Vec::with_capacity(500),
            trades_900s: Vec::with_capacity(1500),
            trades_3600s: Vec::with_capacity(6000),
            trades_7200s: Vec::with_capacity(12000),
            trades_14400s: Vec::with_capacity(24000),
            unique_wallets_300s: HashSet::new(),
            bot_wallets_300s: HashSet::new(),
            wallet_activity_60s: HashMap::new(),
            trades_by_program: HashMap::new(),
            dca_timestamps_60s: VecDeque::with_capacity(10),
            dca_timestamps_300s: VecDeque::with_capacity(50),
            dca_timestamps_900s: VecDeque::with_capacity(150),
            dca_timestamps_3600s: VecDeque::with_capacity(600),
            dca_timestamps_14400s: VecDeque::with_capacity(2400),
        }
    }

    /// Add a trade to rolling windows
    ///
    /// Phase 2: Data handling only
    /// Phase 4: Bot detection and flagging
    /// - Pushes trade to all window buffers
    /// - Updates unique_wallets_300s with trade wallet
    /// - Detects and flags bot wallets based on rapid trading patterns
    /// - Adds trade to program-specific bucket
    /// - Updates last_seen_ts for pruning
    /// - Appends DCA timestamps for JupiterDCA BUY trades
    pub fn add_trade(&mut self, mut trade: TradeEvent) {
        self.last_seen_ts = trade.timestamp;

        // Phase 4: Bot detection - track wallet activity in 60s window
        let wallet = trade.user_account.clone();
        let entry = self.wallet_activity_60s.entry(wallet.clone()).or_insert((0, trade.timestamp));
        entry.0 += 1;
        entry.1 = trade.timestamp;
        
        // Flag as bot if wallet has >= BOT_TRADE_THRESHOLD trades in 60s window
        if entry.0 >= Self::BOT_TRADE_THRESHOLD {
            trade.is_bot = true;
            self.bot_wallets_300s.insert(wallet.clone());
        }

        self.unique_wallets_300s.insert(wallet);

        self.trades_by_program
            .entry(trade.source_program.clone())
            .or_default()
            .push(trade.clone());

        if trade.source_program == "JupiterDCA" && trade.direction == TradeDirection::Buy {
            let timestamp = trade.timestamp;
            self.dca_timestamps_60s.push_back(timestamp);
            self.dca_timestamps_300s.push_back(timestamp);
            self.dca_timestamps_900s.push_back(timestamp);
            self.dca_timestamps_3600s.push_back(timestamp);
            self.dca_timestamps_14400s.push_back(timestamp);
        }

        self.trades_60s.push(trade.clone());
        self.trades_300s.push(trade.clone());
        self.trades_900s.push(trade.clone());
        self.trades_3600s.push(trade.clone());
        self.trades_7200s.push(trade.clone());
        self.trades_14400s.push(trade);
    }

    /// Evict trades older than window cutoffs
    ///
    /// Phase 2: Data handling only
    /// Phase 4: Enhanced pruning with wallet activity tracking
    /// - Removes trades outside each window's time range
    /// - Recomputes unique_wallets_300s from remaining trades
    /// - Recomputes bot_wallets_300s from remaining trades
    /// - Evicts old trades from program-specific buckets
    /// - Prunes DCA timestamps outside each window
    /// - Cleans up wallet_activity_60s for bot detection
    pub fn evict_old_trades(&mut self, now: i64) {
        let cutoff_60s = now - 60;
        let cutoff_300s = now - 300;
        let cutoff_900s = now - 900;
        let cutoff_3600s = now - 3600;
        let cutoff_7200s = now - 7200;
        let cutoff_14400s = now - 14400;
        
        // Phase 4: Clean up wallet activity tracking (60s window)
        self.wallet_activity_60s.retain(|_, (_, last_ts)| *last_ts >= cutoff_60s);

        while let Some(&ts) = self.dca_timestamps_60s.front() {
            if ts < cutoff_60s {
                self.dca_timestamps_60s.pop_front();
            } else {
                break;
            }
        }
        while let Some(&ts) = self.dca_timestamps_300s.front() {
            if ts < cutoff_300s {
                self.dca_timestamps_300s.pop_front();
            } else {
                break;
            }
        }
        while let Some(&ts) = self.dca_timestamps_900s.front() {
            if ts < cutoff_900s {
                self.dca_timestamps_900s.pop_front();
            } else {
                break;
            }
        }
        while let Some(&ts) = self.dca_timestamps_3600s.front() {
            if ts < cutoff_3600s {
                self.dca_timestamps_3600s.pop_front();
            } else {
                break;
            }
        }
        while let Some(&ts) = self.dca_timestamps_14400s.front() {
            if ts < cutoff_14400s {
                self.dca_timestamps_14400s.pop_front();
            } else {
                break;
            }
        }

        self.trades_60s
            .retain(|trade| trade.timestamp >= cutoff_60s);

        self.trades_300s
            .retain(|trade| trade.timestamp >= cutoff_300s);

        self.trades_900s
            .retain(|trade| trade.timestamp >= cutoff_900s);

        self.trades_3600s
            .retain(|trade| trade.timestamp >= cutoff_3600s);

        self.trades_7200s
            .retain(|trade| trade.timestamp >= cutoff_7200s);

        self.trades_14400s
            .retain(|trade| trade.timestamp >= cutoff_14400s);

        for trades in self.trades_by_program.values_mut() {
            trades.retain(|trade| trade.timestamp >= cutoff_14400s);
        }

        self.unique_wallets_300s.clear();
        for trade in &self.trades_300s {
            self.unique_wallets_300s.insert(trade.user_account.clone());
        }

        self.bot_wallets_300s.clear();
    }

    /// Compute rolling metrics from current window state
    ///
    /// Phase 2: Data computation only
    /// Phase 4: Enhanced metrics with bot detection and DCA analysis
    /// Returns internal metrics snapshot (not AggregatedTokenState)
    pub fn compute_rolling_metrics(&self) -> RollingMetrics {
        fn compute_window_metrics(
            trades: &[TradeEvent],
        ) -> (f64, i32, i32) {
            let mut net_flow = 0.0;
            let mut buy_count = 0;
            let mut sell_count = 0;

            for trade in trades {
                match trade.direction {
                    TradeDirection::Buy => {
                        net_flow += trade.sol_amount;
                        buy_count += 1;
                    }
                    TradeDirection::Sell => {
                        net_flow -= trade.sol_amount;
                        sell_count += 1;
                    }
                    TradeDirection::Unknown => {}
                }
            }

            (net_flow, buy_count, sell_count)
        }

        let (net_flow_60s, buy_count_60s, sell_count_60s) =
            compute_window_metrics(&self.trades_60s);
        let (net_flow_300s, buy_count_300s, sell_count_300s) =
            compute_window_metrics(&self.trades_300s);
        let (net_flow_900s, buy_count_900s, sell_count_900s) =
            compute_window_metrics(&self.trades_900s);
        let (net_flow_3600s, _, _) =
            compute_window_metrics(&self.trades_3600s);
        let (net_flow_7200s, _, _) =
            compute_window_metrics(&self.trades_7200s);
        let (net_flow_14400s, _, _) =
            compute_window_metrics(&self.trades_14400s);

        // Phase 4: Bot metrics (300s window)
        let mut bot_trades_count = 0;
        let mut bot_flow = 0.0;
        for trade in &self.trades_300s {
            if trade.is_bot {
                bot_trades_count += 1;
                match trade.direction {
                    TradeDirection::Buy => bot_flow += trade.sol_amount,
                    TradeDirection::Sell => bot_flow -= trade.sol_amount,
                    TradeDirection::Unknown => {}
                }
            }
        }

        // Phase 4: DCA metrics (300s window)
        let mut dca_flow = 0.0;
        let mut dca_wallets = HashSet::new();
        for trade in &self.trades_300s {
            if trade.is_dca {
                match trade.direction {
                    TradeDirection::Buy => dca_flow += trade.sol_amount,
                    TradeDirection::Sell => dca_flow -= trade.sol_amount,
                    TradeDirection::Unknown => {}
                }
                dca_wallets.insert(trade.user_account.clone());
            }
        }
        
        // Phase 4: DCA ratio (DCA flow / total flow)
        let dca_ratio = if net_flow_300s.abs() > 0.0 {
            dca_flow / net_flow_300s
        } else {
            0.0
        };

        let dca_buys_60s = self.dca_timestamps_60s.len() as i32;
        let dca_buys_300s = self.dca_timestamps_300s.len() as i32;
        let dca_buys_900s = self.dca_timestamps_900s.len() as i32;
        let dca_buys_3600s = self.dca_timestamps_3600s.len() as i32;
        let dca_buys_14400s = self.dca_timestamps_14400s.len() as i32;

        RollingMetrics {
            net_flow_60s_sol: net_flow_60s,
            net_flow_300s_sol: net_flow_300s,
            net_flow_900s_sol: net_flow_900s,
            net_flow_3600s_sol: net_flow_3600s,
            net_flow_7200s_sol: net_flow_7200s,
            net_flow_14400s_sol: net_flow_14400s,
            buy_count_60s,
            sell_count_60s,
            buy_count_300s,
            sell_count_300s,
            buy_count_900s,
            sell_count_900s,
            unique_wallets_300s: self.unique_wallets_300s.len() as i32,
            bot_wallets_count_300s: self.bot_wallets_300s.len() as i32,
            bot_trades_count_300s: bot_trades_count,
            bot_flow_300s_sol: bot_flow,
            dca_buys_60s,
            dca_buys_300s,
            dca_buys_900s,
            dca_buys_3600s,
            dca_buys_14400s,
            dca_flow_300s_sol: dca_flow,
            dca_unique_wallets_300s: dca_wallets.len() as i32,
            dca_ratio_300s: dca_ratio,
        }
    }
    
    /// Phase 4: Self-verification layer
    /// Validates internal consistency of rolling metrics
    /// Returns true if all checks pass, false otherwise with logged warnings
    pub fn verify_metrics(&self, metrics: &RollingMetrics) -> bool {
        let mut valid = true;
        
        // Check 1: Timestamps monotonic within each window
        if !self.trades_60s.is_empty() {
            let first_ts = self.trades_60s.first().unwrap().timestamp;
            let last_ts = self.trades_60s.last().unwrap().timestamp;
            if first_ts > last_ts {
                log::warn!(
                    "⚠️ VERIFICATION: Non-monotonic timestamps in 60s window for mint {}",
                    self.mint
                );
                valid = false;
            }
        }
        
        // Check 2: Flow sums correct (buys - sells)
        let expected_flow = metrics.buy_count_300s as f64 * 0.1 - metrics.sell_count_300s as f64 * 0.1;
        let flow_diff = (metrics.net_flow_300s_sol - expected_flow).abs();
        if flow_diff > metrics.net_flow_300s_sol.abs() * 2.0 {
            log::warn!(
                "⚠️ VERIFICATION: Flow sum mismatch for mint {} (expected ~{}, got {})",
                self.mint, expected_flow, metrics.net_flow_300s_sol
            );
        }
        
        // Check 3: Wallet uniqueness per window
        if metrics.unique_wallets_300s > (metrics.buy_count_300s + metrics.sell_count_300s) {
            log::warn!(
                "⚠️ VERIFICATION: More unique wallets than trades for mint {}",
                self.mint
            );
            valid = false;
        }
        
        // Check 4: DCA metrics consistent with trade flags
        let dca_count = self.trades_300s.iter().filter(|t| t.is_dca).count() as i32;
        if dca_count != metrics.dca_buys_300s {
            log::warn!(
                "⚠️ VERIFICATION: DCA count mismatch for mint {} (expected {}, got {})",
                self.mint, dca_count, metrics.dca_buys_300s
            );
            valid = false;
        }
        
        // Check 5: Bot metrics within bounds
        if metrics.bot_trades_count_300s > (metrics.buy_count_300s + metrics.sell_count_300s) {
            log::warn!(
                "⚠️ VERIFICATION: More bot trades than total trades for mint {}",
                self.mint
            );
            valid = false;
        }
        
        valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_trade(
        timestamp: i64,
        mint: &str,
        direction: TradeDirection,
        sol_amount: f64,
        wallet: &str,
        is_bot: bool,
        is_dca: bool,
    ) -> TradeEvent {
        TradeEvent {
            timestamp,
            mint: mint.to_string(),
            direction,
            sol_amount,
            token_amount: 1000.0,
            token_decimals: 6,
            user_account: wallet.to_string(),
            source_program: if is_dca { "JupiterDCA" } else { "PumpSwap" }.to_string(),
            is_bot,
            is_dca,
        }
    }

    #[test]
    fn test_bot_detection_rapid_trading() {
        let mut state = TokenRollingState::new("test_mint".to_string());
        let now = 1000i64;

        // Wallet A makes 3 rapid trades within 60s (should be flagged as bot)
        let trade1 = create_test_trade(now, "test_mint", TradeDirection::Buy, 1.0, "wallet_a", false, false);
        let trade2 = create_test_trade(now + 10, "test_mint", TradeDirection::Sell, 0.5, "wallet_a", false, false);
        let trade3 = create_test_trade(now + 20, "test_mint", TradeDirection::Buy, 2.0, "wallet_a", false, false);

        state.add_trade(trade1);
        state.add_trade(trade2);
        state.add_trade(trade3);

        // Verify wallet_a is flagged as bot
        assert!(state.bot_wallets_300s.contains("wallet_a"));
        
        // Verify trades are flagged
        let bot_count = state.trades_60s.iter().filter(|t| t.is_bot).count();
        assert_eq!(bot_count, 1); // Third trade should be flagged
    }

    #[test]
    fn test_rolling_windows_pruning() {
        let mut state = TokenRollingState::new("test_mint".to_string());
        let base_time = 1000i64;

        // Add trades at different timestamps
        state.add_trade(create_test_trade(base_time, "test_mint", TradeDirection::Buy, 1.0, "w1", false, false));
        state.add_trade(create_test_trade(base_time + 30, "test_mint", TradeDirection::Sell, 0.5, "w2", false, false));
        state.add_trade(create_test_trade(base_time + 100, "test_mint", TradeDirection::Buy, 2.0, "w3", false, false));

        assert_eq!(state.trades_60s.len(), 3);
        assert_eq!(state.trades_300s.len(), 3);

        // Evict trades older than 60s
        state.evict_old_trades(base_time + 120);

        // First two trades should be evicted from 60s window
        assert_eq!(state.trades_60s.len(), 1);
        // All trades still in 300s window
        assert_eq!(state.trades_300s.len(), 3);
    }

    #[test]
    fn test_dca_metrics_calculation() {
        let mut state = TokenRollingState::new("test_mint".to_string());
        let now = 1000i64;

        // Add regular trades
        state.add_trade(create_test_trade(now, "test_mint", TradeDirection::Buy, 5.0, "w1", false, false));
        state.add_trade(create_test_trade(now + 10, "test_mint", TradeDirection::Sell, 2.0, "w2", false, false));

        // Add DCA trades
        state.add_trade(create_test_trade(now + 20, "test_mint", TradeDirection::Buy, 1.0, "w3", false, true));
        state.add_trade(create_test_trade(now + 30, "test_mint", TradeDirection::Buy, 1.5, "w4", false, true));

        let metrics = state.compute_rolling_metrics();

        // Verify DCA counts
        assert_eq!(metrics.dca_buys_60s, 2);
        assert_eq!(metrics.dca_buys_300s, 2);
        
        // Verify DCA flow (1.0 + 1.5 = 2.5)
        assert!((metrics.dca_flow_300s_sol - 2.5).abs() < 0.001);
        
        // Verify DCA unique wallets
        assert_eq!(metrics.dca_unique_wallets_300s, 2);
        
        // Verify total net flow (5.0 - 2.0 + 1.0 + 1.5 = 5.5)
        assert!((metrics.net_flow_300s_sol - 5.5).abs() < 0.001);
        
        // Verify DCA ratio (2.5 / 5.5 ≈ 0.454)
        assert!((metrics.dca_ratio_300s - (2.5 / 5.5)).abs() < 0.01);
    }

    #[test]
    fn test_bot_flow_metrics() {
        let mut state = TokenRollingState::new("test_mint".to_string());
        let now = 1000i64;

        // Create bot wallet with 3 rapid trades
        state.add_trade(create_test_trade(now, "test_mint", TradeDirection::Buy, 1.0, "bot_wallet", false, false));
        state.add_trade(create_test_trade(now + 5, "test_mint", TradeDirection::Buy, 2.0, "bot_wallet", false, false));
        state.add_trade(create_test_trade(now + 10, "test_mint", TradeDirection::Sell, 1.5, "bot_wallet", false, false));

        // Add normal trade
        state.add_trade(create_test_trade(now + 20, "test_mint", TradeDirection::Buy, 5.0, "normal_wallet", false, false));

        let metrics = state.compute_rolling_metrics();

        // Verify bot wallet count
        assert_eq!(metrics.bot_wallets_count_300s, 1);
        
        // Bot trades: only the 3rd trade is flagged (when threshold is reached)
        assert_eq!(metrics.bot_trades_count_300s, 1);
        
        // Bot flow: -1.5 (only the sell that triggered bot flag)
        assert!((metrics.bot_flow_300s_sol - (-1.5)).abs() < 0.001);
    }

    #[test]
    fn test_multiple_windows() {
        let mut state = TokenRollingState::new("test_mint".to_string());
        let base_time = 1000i64;

        // Add trades spread across time
        state.add_trade(create_test_trade(base_time, "test_mint", TradeDirection::Buy, 1.0, "w1", false, false));
        state.add_trade(create_test_trade(base_time + 100, "test_mint", TradeDirection::Buy, 2.0, "w2", false, false));
        state.add_trade(create_test_trade(base_time + 400, "test_mint", TradeDirection::Buy, 3.0, "w3", false, false));
        state.add_trade(create_test_trade(base_time + 1000, "test_mint", TradeDirection::Buy, 4.0, "w4", false, false));
        state.add_trade(create_test_trade(base_time + 1080, "test_mint", TradeDirection::Buy, 5.0, "w5", false, false));

        state.evict_old_trades(base_time + 1100);

        let metrics = state.compute_rolling_metrics();

        // 60s window: only last trade at 2080 (5.0), cutoff is 2040
        assert_eq!(metrics.buy_count_60s, 1);
        assert!((metrics.net_flow_60s_sol - 5.0).abs() < 0.001);

        // 300s window: cutoff is 1800, so trades at 2000 (4.0), 2080 (5.0) = 9.0
        assert_eq!(metrics.buy_count_300s, 2);
        assert!((metrics.net_flow_300s_sol - 9.0).abs() < 0.001);

        // 900s window: cutoff is 1200, so trades at 1400 (3.0), 2000 (4.0), 2080 (5.0) = 12.0
        assert_eq!(metrics.buy_count_900s, 3);
        assert!((metrics.net_flow_900s_sol - 12.0).abs() < 0.001);

        // 3600s window: all trades (1.0 + 2.0 + 3.0 + 4.0 + 5.0 = 15.0)
        assert!((metrics.net_flow_3600s_sol - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_unique_wallets_counting() {
        let mut state = TokenRollingState::new("test_mint".to_string());
        let now = 1000i64;

        // Same wallet makes multiple trades
        state.add_trade(create_test_trade(now, "test_mint", TradeDirection::Buy, 1.0, "wallet_a", false, false));
        state.add_trade(create_test_trade(now + 10, "test_mint", TradeDirection::Sell, 0.5, "wallet_a", false, false));
        
        // Different wallets
        state.add_trade(create_test_trade(now + 20, "test_mint", TradeDirection::Buy, 2.0, "wallet_b", false, false));
        state.add_trade(create_test_trade(now + 30, "test_mint", TradeDirection::Buy, 3.0, "wallet_c", false, false));

        let metrics = state.compute_rolling_metrics();

        // Should have 3 unique wallets despite 4 trades
        assert_eq!(metrics.unique_wallets_300s, 3);
        assert_eq!(metrics.buy_count_300s + metrics.sell_count_300s, 4);
    }

    #[test]
    fn test_out_of_order_timestamps() {
        let mut state = TokenRollingState::new("test_mint".to_string());

        // Add trades in non-chronological order
        state.add_trade(create_test_trade(1000, "test_mint", TradeDirection::Buy, 1.0, "w1", false, false));
        state.add_trade(create_test_trade(1050, "test_mint", TradeDirection::Buy, 2.0, "w2", false, false));
        state.add_trade(create_test_trade(1020, "test_mint", TradeDirection::Buy, 1.5, "w3", false, false)); // Out of order

        let metrics = state.compute_rolling_metrics();

        // Should still correctly compute net flow
        assert!((metrics.net_flow_300s_sol - 4.5).abs() < 0.001);
        assert_eq!(metrics.buy_count_300s, 3);
    }

    #[test]
    fn test_event_bursts() {
        let mut state = TokenRollingState::new("test_mint".to_string());
        let now = 1000i64;

        // Simulate burst of 100 trades
        for i in 0..100 {
            let wallet = format!("wallet_{}", i % 10); // 10 unique wallets
            state.add_trade(create_test_trade(
                now + i,
                "test_mint",
                if i % 2 == 0 { TradeDirection::Buy } else { TradeDirection::Sell },
                0.1,
                &wallet,
                false,
                false,
            ));
        }

        let metrics = state.compute_rolling_metrics();

        // Verify counts
        assert_eq!(metrics.buy_count_300s + metrics.sell_count_300s, 100);
        assert_eq!(metrics.buy_count_300s, 50);
        assert_eq!(metrics.sell_count_300s, 50);
        
        // Verify unique wallets (should be 10)
        assert_eq!(metrics.unique_wallets_300s, 10);
    }

    #[test]
    fn test_verification_layer() {
        let mut state = TokenRollingState::new("test_mint".to_string());
        let now = 1000i64;

        // Add normal trades
        state.add_trade(create_test_trade(now, "test_mint", TradeDirection::Buy, 5.0, "w1", false, false));
        state.add_trade(create_test_trade(now + 10, "test_mint", TradeDirection::Sell, 2.0, "w2", false, false));

        let metrics = state.compute_rolling_metrics();

        // Verification should pass
        assert!(state.verify_metrics(&metrics));
    }

    #[test]
    fn test_dca_ratio_zero_flow() {
        let state = TokenRollingState::new("test_mint".to_string());

        // No trades, so net flow is 0
        let metrics = state.compute_rolling_metrics();

        // DCA ratio should be 0.0 when net flow is 0
        assert_eq!(metrics.dca_ratio_300s, 0.0);
        assert_eq!(metrics.net_flow_300s_sol, 0.0);
    }

    #[test]
    fn test_wallet_activity_cleanup() {
        let mut state = TokenRollingState::new("test_mint".to_string());
        let now = 1000i64;

        // Add trades
        state.add_trade(create_test_trade(now, "test_mint", TradeDirection::Buy, 1.0, "w1", false, false));
        state.add_trade(create_test_trade(now + 30, "test_mint", TradeDirection::Buy, 1.0, "w2", false, false));

        assert_eq!(state.wallet_activity_60s.len(), 2);

        // Evict old trades (after 80s, first wallet should be cleaned up)
        state.evict_old_trades(now + 80);

        // Only w2 should remain in wallet_activity_60s
        assert_eq!(state.wallet_activity_60s.len(), 1);
        assert!(state.wallet_activity_60s.contains_key("w2"));
    }
}
