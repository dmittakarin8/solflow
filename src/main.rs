mod processor;
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
    carbon_pumpfun_decoder::{PumpfunDecoder, PROGRAM_ID as PUMPFUN_PID},
    carbon_pump_swap_decoder::{PumpSwapDecoder, PROGRAM_ID as PUMPSWAP_PID},
    carbon_moonshot_decoder::{MoonshotDecoder, PROGRAM_ID as MOONSHOT_PID},
    carbon_bonkswap_decoder::{BonkswapDecoder, PROGRAM_ID as BONKSWAP_PID},
    carbon_jupiter_dca_decoder::{JupiterDcaDecoder, PROGRAM_ID as JUPITER_DCA_PID},
    yellowstone_grpc_proto::geyser::{CommitmentLevel, SubscribeRequestFilterTransactions},
    crate::processor::NetSolFlowProcessor,
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
                PUMPFUN_PID.to_string(),
                PUMPSWAP_PID.to_string(),
                MOONSHOT_PID.to_string(),
                BONKSWAP_PID.to_string(),
                JUPITER_DCA_PID.to_string(),
            ],
            ..Default::default()
        },
    );

    log::info!("🎯 Filtering for 5 DEX Program IDs (Conviction List)");

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

    log::info!("🔧 Building Pipeline with 5 DEX Decoders");

    Pipeline::builder()
        .datasource(client)
        .instruction(PumpfunDecoder, NetSolFlowProcessor::new(seen_signatures.clone()))
        .instruction(PumpSwapDecoder, NetSolFlowProcessor::new(seen_signatures.clone()))
        .instruction(MoonshotDecoder, NetSolFlowProcessor::new(seen_signatures.clone()))
        .instruction(BonkswapDecoder, NetSolFlowProcessor::new(seen_signatures.clone()))
        .instruction(JupiterDcaDecoder, NetSolFlowProcessor::new(seen_signatures.clone()))
        .build()?
        .run()
        .await?;

    Ok(())
}
