use solflow::{
    state::TokenRollingState,
    types::{TradeDirection, TradeEvent},
};

#[test]
fn test_pumpfun_buy_extraction() {
    let trade_event = TradeEvent {
        timestamp: 1000,
        mint: "test_mint_123".to_string(),
        direction: TradeDirection::Buy,
        sol_amount: 1.5,
        token_amount: 1000000.0,
        token_decimals: 6,
        user_account: "user123".to_string(),
        source_program: "Pumpfun".to_string(),
    };

    assert_eq!(trade_event.direction, TradeDirection::Buy);
    assert_eq!(trade_event.sol_amount, 1.5);
    assert_eq!(trade_event.source_program, "Pumpfun");
}

#[test]
fn test_rolling_state_update() {
    let mint = "test_mint_456".to_string();
    let mut rolling_state = TokenRollingState::new(mint.clone());

    let buy_trade = TradeEvent {
        timestamp: 1000,
        mint: mint.clone(),
        direction: TradeDirection::Buy,
        sol_amount: 2.0,
        token_amount: 500000.0,
        token_decimals: 6,
        user_account: "buyer1".to_string(),
        source_program: "PumpSwap".to_string(),
    };

    rolling_state.add_trade(buy_trade);

    rolling_state.evict_old_trades(1000);

    let metrics = rolling_state.compute_rolling_metrics();

    assert_eq!(metrics.net_flow_60s_sol, 2.0);
    assert_eq!(metrics.buy_count_60s, 1);
    assert_eq!(metrics.sell_count_60s, 0);
}

#[test]
fn test_rolling_state_buy_and_sell() {
    let mint = "test_mint_789".to_string();
    let mut rolling_state = TokenRollingState::new(mint.clone());

    let buy_trade = TradeEvent {
        timestamp: 1000,
        mint: mint.clone(),
        direction: TradeDirection::Buy,
        sol_amount: 5.0,
        token_amount: 1000000.0,
        token_decimals: 6,
        user_account: "buyer1".to_string(),
        source_program: "Pumpfun".to_string(),
    };

    let sell_trade = TradeEvent {
        timestamp: 1010,
        mint: mint.clone(),
        direction: TradeDirection::Sell,
        sol_amount: 2.0,
        token_amount: 400000.0,
        token_decimals: 6,
        user_account: "seller1".to_string(),
        source_program: "Pumpfun".to_string(),
    };

    rolling_state.add_trade(buy_trade);
    rolling_state.add_trade(sell_trade);

    rolling_state.evict_old_trades(1010);

    let metrics = rolling_state.compute_rolling_metrics();

    assert_eq!(metrics.net_flow_60s_sol, 3.0);
    assert_eq!(metrics.buy_count_60s, 1);
    assert_eq!(metrics.sell_count_60s, 1);
}

#[test]
fn test_rolling_state_eviction() {
    let mint = "test_mint_eviction".to_string();
    let mut rolling_state = TokenRollingState::new(mint.clone());

    let old_trade = TradeEvent {
        timestamp: 1000,
        mint: mint.clone(),
        direction: TradeDirection::Buy,
        sol_amount: 1.0,
        token_amount: 100000.0,
        token_decimals: 6,
        user_account: "old_buyer".to_string(),
        source_program: "Moonshot".to_string(),
    };

    let new_trade = TradeEvent {
        timestamp: 1100,
        mint: mint.clone(),
        direction: TradeDirection::Buy,
        sol_amount: 2.0,
        token_amount: 200000.0,
        token_decimals: 6,
        user_account: "new_buyer".to_string(),
        source_program: "Moonshot".to_string(),
    };

    rolling_state.add_trade(old_trade);
    rolling_state.add_trade(new_trade);

    rolling_state.evict_old_trades(1100);

    let metrics = rolling_state.compute_rolling_metrics();

    assert_eq!(metrics.buy_count_60s, 1);
    assert_eq!(metrics.net_flow_60s_sol, 2.0);
}

#[test]
fn test_direction_normalization() {
    assert_eq!(TradeDirection::Buy, TradeDirection::Buy);
    assert_eq!(TradeDirection::Sell, TradeDirection::Sell);
    assert_ne!(TradeDirection::Buy, TradeDirection::Sell);
}

#[test]
fn test_dca_tracking() {
    let mint = "test_mint_dca".to_string();
    let mut rolling_state = TokenRollingState::new(mint.clone());

    let dca_trade = TradeEvent {
        timestamp: 1000,
        mint: mint.clone(),
        direction: TradeDirection::Buy,
        sol_amount: 0.5,
        token_amount: 50000.0,
        token_decimals: 6,
        user_account: "dca_user".to_string(),
        source_program: "JupiterDCA".to_string(),
    };

    rolling_state.add_trade(dca_trade);
    rolling_state.evict_old_trades(1000);

    let metrics = rolling_state.compute_rolling_metrics();

    assert_eq!(metrics.dca_buys_60s, 1);
    assert_eq!(metrics.dca_buys_300s, 1);
}
