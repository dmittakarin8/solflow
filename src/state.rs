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
    
    // Bot detection metrics (Phase 3-A)
    pub bot_wallets_count_300s: i32,
    pub bot_trades_count_300s: i32,

    // DCA buy counts (rolling windows)
    // Phase 6: DCA Rolling Windows
    pub dca_buys_60s: i32,
    pub dca_buys_300s: i32,
    pub dca_buys_900s: i32,
    pub dca_buys_3600s: i32,
    pub dca_buys_14400s: i32,
}

impl TokenRollingState {
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
    /// - Pushes trade to all window buffers
    /// - Updates unique_wallets_300s with trade wallet
    /// - Adds trade to program-specific bucket
    /// - Updates last_seen_ts for pruning
    /// - Appends DCA timestamps for JupiterDCA BUY trades
    pub fn add_trade(&mut self, trade: TradeEvent) {
        self.last_seen_ts = trade.timestamp;

        self.unique_wallets_300s
            .insert(trade.user_account.clone());

        self.trades_by_program
            .entry(trade.source_program.clone())
            .or_insert_with(Vec::new)
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
    /// - Removes trades outside each window's time range
    /// - Recomputes unique_wallets_300s from remaining trades
    /// - Recomputes bot_wallets_300s from remaining trades
    /// - Evicts old trades from program-specific buckets
    /// - Prunes DCA timestamps outside each window
    pub fn evict_old_trades(&mut self, now: i64) {
        let cutoff_60s = now - 60;
        let cutoff_300s = now - 300;
        let cutoff_900s = now - 900;
        let cutoff_3600s = now - 3600;
        let cutoff_7200s = now - 7200;
        let cutoff_14400s = now - 14400;

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
            bot_wallets_count_300s: 0,
            bot_trades_count_300s: 0,
            dca_buys_60s,
            dca_buys_300s,
            dca_buys_900s,
            dca_buys_3600s,
            dca_buys_14400s,
        }
    }
}
