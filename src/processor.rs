use {
    crate::{state::TokenRollingState, types::TradeEvent},
    async_trait::async_trait,
    carbon_core::{
        error::CarbonResult,
        instruction::InstructionProcessorInputType,
        metrics::MetricsCollection,
        processor::Processor,
    },
    dashmap::DashMap,
    std::{marker::PhantomData, sync::Arc},
};

pub struct NetSolFlowProcessor<T> {
    pub seen_signatures: Arc<DashMap<String, bool>>,
    pub rolling_states: Arc<DashMap<String, TokenRollingState>>,
    pub extractor: fn(&InstructionProcessorInputType<T>) -> Option<TradeEvent>,
    _phantom: PhantomData<T>,
}

impl<T> NetSolFlowProcessor<T> {
    pub fn new(
        seen_signatures: Arc<DashMap<String, bool>>,
        rolling_states: Arc<DashMap<String, TokenRollingState>>,
        extractor: fn(&InstructionProcessorInputType<T>) -> Option<TradeEvent>,
    ) -> Self {
        Self {
            seen_signatures,
            rolling_states,
            extractor,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T> Processor for NetSolFlowProcessor<T>
where
    T: Send + Sync + 'static,
{
    type InputType = InstructionProcessorInputType<T>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (metadata, _decoded_instruction, _nested_instructions, _raw_instruction) = &data;

        let tx_meta = &metadata.transaction_metadata;
        let sig_str = tx_meta.signature.to_string();

        if self.seen_signatures.contains_key(&sig_str) {
            return Ok(());
        }
        self.seen_signatures.insert(sig_str.clone(), true);

        let meta = &tx_meta.meta;

        let pre_balance = meta.pre_balances.get(0).copied().unwrap_or(0);
        let post_balance = meta.post_balances.get(0).copied().unwrap_or(0);
        let fee = meta.fee;

        let net_flow_lamports = (post_balance as i128 - pre_balance as i128) + fee as i128;
        let net_flow_sol = net_flow_lamports as f64 / 1_000_000_000.0;

        if net_flow_sol.abs() > 0.01 {
            log::info!(
                "✅ NET FLOW | Slot: {} | Sig: {} | Amount: {:.4} SOL",
                tx_meta.slot,
                sig_str,
                net_flow_sol
            );
        }

        if let Some(trade_event) = (self.extractor)(&data) {
            let mint = trade_event.mint.clone();
            let current_timestamp = trade_event.timestamp;

            let mut rolling_state = self
                .rolling_states
                .entry(mint.clone())
                .or_insert_with(|| TokenRollingState::new(mint.clone()));

            rolling_state.add_trade(trade_event.clone());

            rolling_state.evict_old_trades(current_timestamp);

            log::info!(
                "📊 TRADE EXTRACTED | Mint: {} | Direction: {:?} | SOL: {:.4}",
                mint,
                trade_event.direction,
                trade_event.sol_amount
            );
        }

        Ok(())
    }
}
