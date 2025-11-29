mod processor;
mod state;
mod trade_extractor;
mod types;
mod signals;
pub mod sqlite_pragma;
pub mod db;

use {
    dotenv::dotenv,
    std::{collections::HashMap, env, sync::Arc},
    dashmap::DashMap,
    tokio::sync::RwLock,
    carbon_core::pipeline::Pipeline,
    carbon_yellowstone_grpc_datasource::{
        BlockFilters, YellowstoneGrpcClientConfig, YellowstoneGrpcGeyserClient,
    },
    carbon_pump_swap_decoder::{PumpSwapDecoder, PROGRAM_ID as PUMPSWAP_PID},
    carbon_moonshot_decoder::{MoonshotDecoder, PROGRAM_ID as MOONSHOT_PID},
    carbon_bonkswap_decoder::{BonkswapDecoder, PROGRAM_ID as BONKSWAP_PID},
    carbon_jupiter_dca_decoder::{JupiterDcaDecoder, PROGRAM_ID as JUPITER_DCA_PID},
    yellowstone_grpc_proto::geyser::{CommitmentLevel, SubscribeRequestFilterTransactions},
    crate::{processor::NetSolFlowProcessor, state::TokenRollingState, trade_extractor::TradeExtractor},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    log::info!("🗄️  Initializing database");
    db::init_database().expect("Failed to initialize database");

    let geyser_url = env::var("GEYSER_URL").expect("GEYSER_URL not set");
    let x_token = env::var("X_TOKEN").expect("X_TOKEN not set");

    log::info!("🚀 Initializing SolFlow Pipeline");
    log::info!("📡 Connecting to Geyser: {}", geyser_url);

    let mut transaction_filters = HashMap::new();
    transaction_filters.insert(
        "solflow_filter".to_string(),
        SubscribeRequestFilterTransactions {
            vote: Some(false),
            failed: Some(false),
            account_include: vec![
                PUMPSWAP_PID.to_string(),
                MOONSHOT_PID.to_string(),
                BONKSWAP_PID.to_string(),
                JUPITER_DCA_PID.to_string(),
            ],
            ..Default::default()
        },
    );

    log::info!("🎯 Filtering for 4 DEX Program IDs (PumpSwap + Others)");

    let client = YellowstoneGrpcGeyserClient::new(
        geyser_url,
        Some(x_token),
        Some(CommitmentLevel::Finalized),
        HashMap::new(),
        transaction_filters,
        BlockFilters::default(),
        Arc::new(RwLock::new(std::collections::HashSet::new())),
        YellowstoneGrpcClientConfig::default(),
    );

    let seen_signatures = Arc::new(DashMap::new());
    let rolling_states: Arc<DashMap<String, TokenRollingState>> = Arc::new(DashMap::new());

    // Phase 5: Create channel for database writes
    let (writer_tx, writer_rx) = tokio::sync::mpsc::channel(1000);
    
    // Phase 5: Spawn background write loop
    log::info!("📝 Spawning database write loop");
    tokio::spawn(async move {
        db::run_write_loop(writer_rx).await;
    });

    log::info!("🔧 Building Pipeline with 4 DEX Decoders + Trade Extraction Layer");

    Pipeline::builder()
        .datasource(client)
        .instruction(
            PumpSwapDecoder,
            NetSolFlowProcessor::new(
                seen_signatures.clone(),
                rolling_states.clone(),
                TradeExtractor::extract_from_pumpswap,
                writer_tx.clone(),
            ),
        )
        .instruction(
            MoonshotDecoder,
            NetSolFlowProcessor::new(
                seen_signatures.clone(),
                rolling_states.clone(),
                TradeExtractor::extract_from_moonshot,
                writer_tx.clone(),
            ),
        )
        .instruction(
            BonkswapDecoder,
            NetSolFlowProcessor::new(
                seen_signatures.clone(),
                rolling_states.clone(),
                TradeExtractor::extract_from_bonkswap,
                writer_tx.clone(),
            ),
        )
        .instruction(
            JupiterDcaDecoder,
            NetSolFlowProcessor::new(
                seen_signatures.clone(),
                rolling_states.clone(),
                TradeExtractor::extract_from_jupiter_dca,
                writer_tx.clone(),
            ),
        )
        .build()?
        .run()
        .await?;

    Ok(())
}
