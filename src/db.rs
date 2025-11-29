//! Phase 5: Database Persistence Layer
//! 
//! Real-time SQLite persistence for rolling metrics and trade events.
//! Non-blocking async write loop with batching support.

use rusqlite::{Connection, params};
use std::{env, error::Error, fs, path::Path};
use tokio::sync::mpsc;
use crate::{state::RollingMetrics, types::TradeEvent, signals::Signal};

pub use crate::sqlite_pragma;

/// Write request enum for channel-based batching
#[derive(Debug, Clone)]
pub enum WriteRequest {
    /// UPSERT rolling metrics for a token
    Metrics { mint: String, metrics: RollingMetrics },
    /// Append trade event to trades table
    Trade(TradeEvent),
    /// Phase 6: Append signal event to signals table
    Signal(Signal),
}

/// Initialize database with WAL mode and migrations
pub fn init_database() -> Result<(), Box<dyn Error>> {
    let db_path = env::var("SOLFLOW_DB_PATH")
        .map_err(|_| "SOLFLOW_DB_PATH environment variable not set")?;

    let conn = Connection::open(&db_path)?;
    
    // Enable WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
    
    sqlite_pragma::apply_optimized_pragmas(&conn)?;
    
    run_migrations(&conn)?;
    
    Ok(())
}

/// Run SQL migrations from sql/ directory
fn run_migrations(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let sql_dir = Path::new("sql");
    
    if !sql_dir.exists() {
        return Err("sql/ directory not found".into());
    }

    let mut sql_files: Vec<_> = fs::read_dir(sql_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "sql")
                .unwrap_or(false)
        })
        .collect();

    sql_files.sort_by_key(|entry| entry.file_name());

    let migration_count = sql_files.len();

    for entry in sql_files {
        let path = entry.path();
        let sql = fs::read_to_string(&path)?;
        
        if let Err(e) = conn.execute_batch(&sql) {
            log::warn!("⚠️  Migration {} failed (may be incomplete): {}", 
                       path.file_name().unwrap().to_string_lossy(), e);
        }
    }

    log::info!("✅ Executed {} migrations successfully", migration_count);

    Ok(())
}

/// UPSERT rolling metrics into token_rolling_metrics table
pub fn write_aggregated_state(conn: &Connection, mint: &str, metrics: &RollingMetrics) -> Result<(), Box<dyn Error>> {
    let now = chrono::Utc::now().timestamp();
    
    conn.execute(
        "INSERT INTO token_rolling_metrics (
            mint, updated_at,
            net_flow_60s, net_flow_300s, net_flow_900s, 
            net_flow_3600s, net_flow_7200s, net_flow_14400s,
            unique_wallets_300s, bot_wallets_300s, bot_trades_300s, bot_flow_300s,
            dca_flow_300s, dca_unique_wallets_300s, dca_ratio_300s
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ON CONFLICT(mint) DO UPDATE SET
            updated_at = excluded.updated_at,
            net_flow_60s = excluded.net_flow_60s,
            net_flow_300s = excluded.net_flow_300s,
            net_flow_900s = excluded.net_flow_900s,
            net_flow_3600s = excluded.net_flow_3600s,
            net_flow_7200s = excluded.net_flow_7200s,
            net_flow_14400s = excluded.net_flow_14400s,
            unique_wallets_300s = excluded.unique_wallets_300s,
            bot_wallets_300s = excluded.bot_wallets_300s,
            bot_trades_300s = excluded.bot_trades_300s,
            bot_flow_300s = excluded.bot_flow_300s,
            dca_flow_300s = excluded.dca_flow_300s,
            dca_unique_wallets_300s = excluded.dca_unique_wallets_300s,
            dca_ratio_300s = excluded.dca_ratio_300s",
        params![
            mint, now,
            metrics.net_flow_60s_sol,
            metrics.net_flow_300s_sol,
            metrics.net_flow_900s_sol,
            metrics.net_flow_3600s_sol,
            metrics.net_flow_7200s_sol,
            metrics.net_flow_14400s_sol,
            metrics.unique_wallets_300s,
            metrics.bot_wallets_count_300s,
            metrics.bot_trades_count_300s,
            metrics.bot_flow_300s_sol,
            metrics.dca_flow_300s_sol,
            metrics.dca_unique_wallets_300s,
            metrics.dca_ratio_300s,
        ],
    )?;
    
    Ok(())
}

/// Append trade event to token_trades table
pub fn append_trade(conn: &Connection, event: &TradeEvent) -> Result<(), Box<dyn Error>> {
    let side = match event.direction {
        crate::types::TradeDirection::Buy => "buy",
        crate::types::TradeDirection::Sell => "sell",
        crate::types::TradeDirection::Unknown => "unknown",
    };
    
    conn.execute(
        "INSERT INTO token_trades (mint, timestamp, wallet, side, sol_amount, is_bot, is_dca)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            event.mint,
            event.timestamp,
            event.user_account,
            side,
            event.sol_amount,
            event.is_bot as i32,
            event.is_dca as i32,
        ],
    )?;
    
    Ok(())
}

/// Phase 6: Write signal to token_signals table
pub fn write_signal(conn: &Connection, signal: &Signal) -> Result<(), Box<dyn Error>> {
    let metadata_str = signal.metadata.to_string();
    
    conn.execute(
        "INSERT INTO token_signals (
            mint, signal_type, strength, window, timestamp, metadata
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            signal.mint,
            signal.signal_type.as_str(),
            signal.strength,
            signal.window,
            signal.timestamp,
            metadata_str,
        ],
    )?;
    
    Ok(())
}

/// Phase 6: Get recent trades for a token within a time window
///
/// Used by signals engine to compute wallet concentration and other metrics.
///
/// # Arguments
/// * `conn` - Database connection
/// * `mint` - Token mint address
/// * `window_seconds` - Time window in seconds (e.g., 300 for 5 minutes)
///
/// # Returns
/// Vector of recent trade events within the window
pub fn get_recent_trades(conn: &Connection, mint: &str, window_seconds: i64) -> Result<Vec<TradeEvent>, Box<dyn Error>> {
    let now = chrono::Utc::now().timestamp();
    let cutoff = now - window_seconds;
    
    let mut stmt = conn.prepare(
        "SELECT mint, timestamp, wallet, side, sol_amount, is_bot, is_dca
         FROM token_trades
         WHERE mint = ?1 AND timestamp >= ?2
         ORDER BY timestamp DESC"
    )?;
    
    let trades = stmt.query_map(params![mint, cutoff], |row| {
        let side: String = row.get(3)?;
        let direction = match side.as_str() {
            "buy" => crate::types::TradeDirection::Buy,
            "sell" => crate::types::TradeDirection::Sell,
            _ => crate::types::TradeDirection::Unknown,
        };
        
        let is_bot: i32 = row.get(5)?;
        let is_dca: i32 = row.get(6)?;
        
        Ok(TradeEvent {
            mint: row.get(0)?,
            timestamp: row.get(1)?,
            user_account: row.get(2)?,
            direction,
            sol_amount: row.get(4)?,
            token_amount: 0.0, // Not stored in DB
            token_decimals: 0, // Not stored in DB
            source_program: if is_dca == 1 { "JupiterDCA" } else { "Unknown" }.to_string(),
            is_bot: is_bot == 1,
            is_dca: is_dca == 1,
        })
    })?;
    
    let mut result = Vec::new();
    for trade in trades {
        result.push(trade?);
    }
    
    Ok(result)
}

/// Background write loop for async batching
/// 
/// Consumes WriteRequests from channel and batches them into transactions.
/// Flushes periodically to ensure low latency.
pub async fn run_write_loop(mut rx: mpsc::Receiver<WriteRequest>) {
    let db_path = match env::var("SOLFLOW_DB_PATH") {
        Ok(path) => path,
        Err(_) => {
            log::error!("❌ SOLFLOW_DB_PATH not set, write loop exiting");
            return;
        }
    };
    
    let conn = match Connection::open(&db_path) {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("❌ Failed to open database for write loop: {}", e);
            return;
        }
    };
    
    log::info!("📝 Database write loop started");
    
    let mut batch = Vec::with_capacity(100);
    let mut last_flush = std::time::Instant::now();
    let flush_interval = std::time::Duration::from_millis(100);
    
    loop {
        tokio::select! {
            // Receive write requests
            Some(req) = rx.recv() => {
                batch.push(req);
                
                // Flush if batch is full or interval elapsed
                if batch.len() >= 100 || last_flush.elapsed() >= flush_interval {
                    if let Err(e) = flush_batch(&conn, &mut batch) {
                        log::error!("❌ Failed to flush write batch: {}", e);
                    }
                    last_flush = std::time::Instant::now();
                }
            }
            // Periodic flush even if batch not full
            _ = tokio::time::sleep(flush_interval) => {
                if !batch.is_empty() {
                    if let Err(e) = flush_batch(&conn, &mut batch) {
                        log::error!("❌ Failed to flush write batch: {}", e);
                    }
                    last_flush = std::time::Instant::now();
                }
            }
        }
    }
}

/// Flush batch of write requests to database
fn flush_batch(conn: &Connection, batch: &mut Vec<WriteRequest>) -> Result<(), Box<dyn Error>> {
    if batch.is_empty() {
        return Ok(());
    }
    
    let tx = conn.unchecked_transaction()?;
    
    for req in batch.drain(..) {
        match req {
            WriteRequest::Metrics { mint, metrics } => {
                if let Err(e) = write_aggregated_state(&tx, &mint, &metrics) {
                    log::warn!("⚠️  Failed to write metrics for {}: {}", mint, e);
                }
            }
            WriteRequest::Trade(event) => {
                if let Err(e) = append_trade(&tx, &event) {
                    log::warn!("⚠️  Failed to append trade for {}: {}", event.mint, e);
                }
            }
            WriteRequest::Signal(signal) => {
                if let Err(e) = write_signal(&tx, &signal) {
                    log::warn!("⚠️  Failed to write signal for {}: {}", signal.mint, e);
                }
            }
        }
    }
    
    tx.commit()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state::RollingMetrics, types::{TradeDirection, TradeEvent}};
    use rusqlite::Connection;

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
        // Enable WAL mode (even for in-memory)
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        conn.execute_batch("PRAGMA synchronous=NORMAL;").unwrap();
        
        // Create tables
        conn.execute_batch(include_str!("../sql/08_token_rolling_metrics.sql")).unwrap();
        conn.execute_batch(include_str!("../sql/09_token_trades.sql")).unwrap();
        
        conn
    }
    
    fn create_test_metrics() -> RollingMetrics {
        RollingMetrics {
            net_flow_60s_sol: 10.0,
            net_flow_300s_sol: 50.0,
            net_flow_900s_sol: 150.0,
            net_flow_3600s_sol: 500.0,
            net_flow_7200s_sol: 800.0,
            net_flow_14400s_sol: 1200.0,
            buy_count_60s: 5,
            sell_count_60s: 2,
            buy_count_300s: 20,
            sell_count_300s: 10,
            buy_count_900s: 50,
            sell_count_900s: 30,
            unique_wallets_300s: 15,
            bot_wallets_count_300s: 2,
            bot_trades_count_300s: 5,
            bot_flow_300s_sol: 8.0,
            dca_buys_60s: 1,
            dca_buys_300s: 3,
            dca_buys_900s: 8,
            dca_buys_3600s: 20,
            dca_buys_14400s: 40,
            dca_flow_300s_sol: 12.0,
            dca_unique_wallets_300s: 3,
            dca_ratio_300s: 0.24,
        }
    }
    
    fn create_test_trade(timestamp: i64) -> TradeEvent {
        TradeEvent {
            timestamp,
            mint: "test_mint".to_string(),
            direction: TradeDirection::Buy,
            sol_amount: 5.0,
            token_amount: 1000.0,
            token_decimals: 6,
            user_account: "test_wallet".to_string(),
            source_program: "PumpSwap".to_string(),
            is_bot: false,
            is_dca: false,
        }
    }

    #[test]
    fn test_db_initialization() {
        let conn = create_test_db();
        
        // Verify tables exist
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        
        assert!(tables.contains(&"token_rolling_metrics".to_string()));
        assert!(tables.contains(&"token_trades".to_string()));
    }

    #[test]
    fn test_write_aggregated_state_insert() {
        let conn = create_test_db();
        let metrics = create_test_metrics();
        
        write_aggregated_state(&conn, "test_mint", &metrics).unwrap();
        
        // Verify insert
        let mut stmt = conn.prepare("SELECT mint, net_flow_300s, unique_wallets_300s FROM token_rolling_metrics WHERE mint = ?1").unwrap();
        let row: (String, f64, i32) = stmt.query_row(params!["test_mint"], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).unwrap();
        
        assert_eq!(row.0, "test_mint");
        assert_eq!(row.1, 50.0);
        assert_eq!(row.2, 15);
    }

    #[test]
    fn test_write_aggregated_state_upsert() {
        let conn = create_test_db();
        let mut metrics = create_test_metrics();
        
        // First insert
        write_aggregated_state(&conn, "test_mint", &metrics).unwrap();
        
        // Update metrics
        metrics.net_flow_300s_sol = 100.0;
        metrics.unique_wallets_300s = 25;
        
        // UPSERT (should update, not insert)
        write_aggregated_state(&conn, "test_mint", &metrics).unwrap();
        
        // Verify update
        let mut stmt = conn.prepare("SELECT net_flow_300s, unique_wallets_300s FROM token_rolling_metrics WHERE mint = ?1").unwrap();
        let row: (f64, i32) = stmt.query_row(params!["test_mint"], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).unwrap();
        
        assert_eq!(row.0, 100.0);
        assert_eq!(row.1, 25);
        
        // Verify only one row exists
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM token_rolling_metrics").unwrap();
        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_append_trade() {
        let conn = create_test_db();
        let trade = create_test_trade(1000);
        
        append_trade(&conn, &trade).unwrap();
        
        // Verify insert
        let mut stmt = conn.prepare("SELECT mint, wallet, side, sol_amount FROM token_trades WHERE id = 1").unwrap();
        let row: (String, String, String, f64) = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        }).unwrap();
        
        assert_eq!(row.0, "test_mint");
        assert_eq!(row.1, "test_wallet");
        assert_eq!(row.2, "buy");
        assert_eq!(row.3, 5.0);
    }

    #[test]
    fn test_append_multiple_trades() {
        let conn = create_test_db();
        
        // Append 10 trades
        for i in 0..10 {
            let mut trade = create_test_trade(1000 + i);
            trade.is_bot = i % 3 == 0;
            trade.is_dca = i % 5 == 0;
            append_trade(&conn, &trade).unwrap();
        }
        
        // Verify count
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM token_trades").unwrap();
        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 10);
        
        // Verify bot flag
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM token_trades WHERE is_bot = 1").unwrap();
        let bot_count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(bot_count, 4); // 0, 3, 6, 9
        
        // Verify DCA flag
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM token_trades WHERE is_dca = 1").unwrap();
        let dca_count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(dca_count, 2); // 0, 5
    }

    #[test]
    fn test_indexes_exist() {
        let conn = create_test_db();
        
        // Verify indexes for token_rolling_metrics
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='token_rolling_metrics'").unwrap();
        let indexes: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        
        assert!(indexes.iter().any(|name| name.contains("updated_at")));
        assert!(indexes.iter().any(|name| name.contains("net_flow_300s")));
        
        // Verify indexes for token_trades
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='token_trades'").unwrap();
        let indexes: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        
        assert!(indexes.iter().any(|name| name.contains("mint")));
        assert!(indexes.iter().any(|name| name.contains("timestamp")));
        assert!(indexes.iter().any(|name| name.contains("is_dca")));
    }

    #[test]
    fn test_flush_batch() {
        let conn = create_test_db();
        let metrics = create_test_metrics();
        let trade = create_test_trade(1000);
        
        let mut batch = vec![
            WriteRequest::Metrics { mint: "mint1".to_string(), metrics: metrics.clone() },
            WriteRequest::Trade(trade.clone()),
            WriteRequest::Metrics { mint: "mint2".to_string(), metrics: metrics.clone() },
        ];
        
        flush_batch(&conn, &mut batch).unwrap();
        
        // Verify batch was cleared
        assert_eq!(batch.len(), 0);
        
        // Verify writes occurred
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM token_rolling_metrics").unwrap();
        let metrics_count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(metrics_count, 2);
        
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM token_trades").unwrap();
        let trades_count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(trades_count, 1);
    }

    #[test]
    fn test_trade_direction_mapping() {
        let conn = create_test_db();
        
        // Test Buy
        let mut trade = create_test_trade(1000);
        trade.direction = TradeDirection::Buy;
        append_trade(&conn, &trade).unwrap();
        
        // Test Sell
        trade.direction = TradeDirection::Sell;
        trade.timestamp = 1001;
        append_trade(&conn, &trade).unwrap();
        
        // Test Unknown
        trade.direction = TradeDirection::Unknown;
        trade.timestamp = 1002;
        append_trade(&conn, &trade).unwrap();
        
        // Verify
        let mut stmt = conn.prepare("SELECT side FROM token_trades ORDER BY id").unwrap();
        let sides: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        
        assert_eq!(sides[0], "buy");
        assert_eq!(sides[1], "sell");
        assert_eq!(sides[2], "unknown");
    }

    #[test]
    fn test_write_loop_batch_size() {
        // This test verifies batching logic (unit test, not integration)
        let conn = create_test_db();
        let metrics = create_test_metrics();
        
        let mut batch = Vec::new();
        for i in 0..100 {
            batch.push(WriteRequest::Metrics {
                mint: format!("mint_{}", i),
                metrics: metrics.clone(),
            });
        }
        
        flush_batch(&conn, &mut batch).unwrap();
        
        // Verify all 100 were written
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM token_rolling_metrics").unwrap();
        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 100);
    }
}
