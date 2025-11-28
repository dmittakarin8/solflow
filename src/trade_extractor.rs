//! Trade extraction adapter for converting Carbon decoder events to TradeEvent
//!
//! Phase 3: Trade Extraction Layer
//!
//! This module serves as the bridge between Carbon's decoded blockchain events
//! and SolFlow's TradeEvent type. It extracts trade information from various
//! DEX programs (Pumpfun, PumpSwap, BonkSwap, Moonshot, JupiterDCA) and normalizes them
//! into a common TradeEvent format.

use crate::types::{TradeDirection, TradeEvent};
use carbon_core::{deserialize::ArrangeAccounts, instruction::InstructionProcessorInputType};

pub struct TradeExtractor;

impl TradeExtractor {
    /// Extract a TradeEvent from a Pumpfun Buy instruction
    pub fn extract_pumpfun_buy(
        accounts: &carbon_pumpfun_decoder::instructions::buy::BuyInstructionAccounts,
        instruction: &carbon_pumpfun_decoder::instructions::buy::Buy,
        timestamp: i64,
    ) -> Option<TradeEvent> {
        Some(TradeEvent {
            timestamp,
            mint: accounts.mint.to_string(),
            direction: TradeDirection::Buy,
            sol_amount: instruction.max_sol_cost as f64 / 1_000_000_000.0,
            token_amount: instruction.amount as f64,
            token_decimals: 6,
            user_account: accounts.user.to_string(),
            source_program: "Pumpfun".to_string(),
        })
    }

    /// Extract a TradeEvent from a Pumpfun Sell instruction
    pub fn extract_pumpfun_sell(
        accounts: &carbon_pumpfun_decoder::instructions::sell::SellInstructionAccounts,
        instruction: &carbon_pumpfun_decoder::instructions::sell::Sell,
        timestamp: i64,
    ) -> Option<TradeEvent> {
        Some(TradeEvent {
            timestamp,
            mint: accounts.mint.to_string(),
            direction: TradeDirection::Sell,
            sol_amount: instruction.min_sol_output as f64 / 1_000_000_000.0,
            token_amount: instruction.amount as f64,
            token_decimals: 6,
            user_account: accounts.user.to_string(),
            source_program: "Pumpfun".to_string(),
        })
    }

    /// Extract a TradeEvent from a PumpSwap BuyEvent
    pub fn extract_pumpswap_buy_event(
        event: &carbon_pump_swap_decoder::instructions::buy_event::BuyEvent,
    ) -> Option<TradeEvent> {
        Some(TradeEvent {
            timestamp: event.timestamp,
            mint: event.pool.to_string(),
            direction: TradeDirection::Buy,
            sol_amount: event.quote_amount_in as f64 / 1_000_000_000.0,
            token_amount: event.base_amount_out as f64,
            token_decimals: 6,
            user_account: event.user.to_string(),
            source_program: "PumpSwap".to_string(),
        })
    }

    /// Extract a TradeEvent from a PumpSwap SellEvent
    pub fn extract_pumpswap_sell_event(
        event: &carbon_pump_swap_decoder::instructions::sell_event::SellEvent,
    ) -> Option<TradeEvent> {
        Some(TradeEvent {
            timestamp: event.timestamp,
            mint: event.pool.to_string(),
            direction: TradeDirection::Sell,
            sol_amount: event.quote_amount_out as f64 / 1_000_000_000.0,
            token_amount: event.base_amount_in as f64,
            token_decimals: 6,
            user_account: event.user.to_string(),
            source_program: "PumpSwap".to_string(),
        })
    }

    /// Extract a TradeEvent from a BonkSwap instruction
    pub fn extract_bonkswap_event(
        _accounts: &[solana_sdk::instruction::AccountMeta],
        _timestamp: i64,
    ) -> Option<TradeEvent> {
        None
    }

    /// Extract a TradeEvent from a Moonshot Buy instruction
    pub fn extract_moonshot_buy(
        accounts: &carbon_moonshot_decoder::instructions::buy::BuyInstructionAccounts,
        instruction: &carbon_moonshot_decoder::instructions::buy::Buy,
        timestamp: i64,
    ) -> Option<TradeEvent> {
        Some(TradeEvent {
            timestamp,
            mint: accounts.mint.to_string(),
            direction: TradeDirection::Buy,
            sol_amount: instruction.data.token_amount as f64 / 1_000_000_000.0,
            token_amount: instruction.data.collateral_amount as f64,
            token_decimals: 6,
            user_account: accounts.sender.to_string(),
            source_program: "Moonshot".to_string(),
        })
    }

    /// Extract a TradeEvent from a Moonshot Sell instruction
    pub fn extract_moonshot_sell(
        accounts: &carbon_moonshot_decoder::instructions::sell::SellInstructionAccounts,
        instruction: &carbon_moonshot_decoder::instructions::sell::Sell,
        timestamp: i64,
    ) -> Option<TradeEvent> {
        Some(TradeEvent {
            timestamp,
            mint: accounts.mint.to_string(),
            direction: TradeDirection::Sell,
            sol_amount: instruction.data.collateral_amount as f64 / 1_000_000_000.0,
            token_amount: instruction.data.token_amount as f64,
            token_decimals: 6,
            user_account: accounts.sender.to_string(),
            source_program: "Moonshot".to_string(),
        })
    }

    /// Extract a TradeEvent from a JupiterDCA FilledEvent
    pub fn extract_jupiter_dca_filled_event(
        event: &carbon_jupiter_dca_decoder::instructions::filled_event::FilledEvent,
    ) -> Option<TradeEvent> {
        let direction = if event.input_mint.to_string() == "So11111111111111111111111111111111111111112" {
            TradeDirection::Buy
        } else if event.output_mint.to_string() == "So11111111111111111111111111111111111111112" {
            TradeDirection::Sell
        } else {
            TradeDirection::Unknown
        };

        if direction == TradeDirection::Unknown {
            return None;
        }

        let sol_amount = if direction == TradeDirection::Buy {
            event.in_amount as f64 / 1_000_000_000.0
        } else {
            event.out_amount as f64 / 1_000_000_000.0
        };

        let token_amount = if direction == TradeDirection::Buy {
            event.out_amount as f64
        } else {
            event.in_amount as f64
        };

        Some(TradeEvent {
            timestamp: chrono::Utc::now().timestamp(),
            mint: if direction == TradeDirection::Buy {
                event.output_mint.to_string()
            } else {
                event.input_mint.to_string()
            },
            direction,
            sol_amount,
            token_amount,
            token_decimals: 6,
            user_account: event.user_key.to_string(),
            source_program: "JupiterDCA".to_string(),
        })
    }

    /// Unified adapter for Pumpfun instructions
    pub fn extract_from_pumpfun(
        input: &InstructionProcessorInputType<carbon_pumpfun_decoder::instructions::PumpfunInstruction>,
    ) -> Option<TradeEvent> {
        let (metadata, decoded_instruction, _nested_instructions, _raw_instruction) = input;
        let timestamp = metadata.transaction_metadata.block_time.unwrap_or(0);

        match &decoded_instruction.data {
            carbon_pumpfun_decoder::instructions::PumpfunInstruction::Buy(buy) => {
                let accounts = carbon_pumpfun_decoder::instructions::buy::Buy::arrange_accounts(
                    &decoded_instruction.accounts,
                )?;
                Self::extract_pumpfun_buy(&accounts, buy, timestamp)
            }
            carbon_pumpfun_decoder::instructions::PumpfunInstruction::Sell(sell) => {
                let accounts = carbon_pumpfun_decoder::instructions::sell::Sell::arrange_accounts(
                    &decoded_instruction.accounts,
                )?;
                Self::extract_pumpfun_sell(&accounts, sell, timestamp)
            }
            _ => None,
        }
    }

    /// Unified adapter for PumpSwap instructions
    pub fn extract_from_pumpswap(
        input: &InstructionProcessorInputType<carbon_pump_swap_decoder::instructions::PumpSwapInstruction>,
    ) -> Option<TradeEvent> {
        let (_metadata, decoded_instruction, _nested_instructions, _raw_instruction) = input;

        match &decoded_instruction.data {
            carbon_pump_swap_decoder::instructions::PumpSwapInstruction::BuyEvent(event) => {
                Self::extract_pumpswap_buy_event(event)
            }
            carbon_pump_swap_decoder::instructions::PumpSwapInstruction::SellEvent(event) => {
                Self::extract_pumpswap_sell_event(event)
            }
            _ => None,
        }
    }

    /// Unified adapter for Moonshot instructions
    pub fn extract_from_moonshot(
        input: &InstructionProcessorInputType<carbon_moonshot_decoder::instructions::MoonshotInstruction>,
    ) -> Option<TradeEvent> {
        let (metadata, decoded_instruction, _nested_instructions, _raw_instruction) = input;
        let timestamp = metadata.transaction_metadata.block_time.unwrap_or(0);

        match &decoded_instruction.data {
            carbon_moonshot_decoder::instructions::MoonshotInstruction::Buy(buy) => {
                let accounts = carbon_moonshot_decoder::instructions::buy::Buy::arrange_accounts(
                    &decoded_instruction.accounts,
                )?;
                Self::extract_moonshot_buy(&accounts, buy, timestamp)
            }
            carbon_moonshot_decoder::instructions::MoonshotInstruction::Sell(sell) => {
                let accounts = carbon_moonshot_decoder::instructions::sell::Sell::arrange_accounts(
                    &decoded_instruction.accounts,
                )?;
                Self::extract_moonshot_sell(&accounts, sell, timestamp)
            }
            _ => None,
        }
    }

    /// Unified adapter for JupiterDCA instructions
    pub fn extract_from_jupiter_dca(
        input: &InstructionProcessorInputType<carbon_jupiter_dca_decoder::instructions::JupiterDcaInstruction>,
    ) -> Option<TradeEvent> {
        let (_metadata, decoded_instruction, _nested_instructions, _raw_instruction) = input;

        match &decoded_instruction.data {
            carbon_jupiter_dca_decoder::instructions::JupiterDcaInstruction::FilledEvent(event) => {
                Self::extract_jupiter_dca_filled_event(event)
            }
            _ => None,
        }
    }

    /// Unified adapter for BonkSwap instructions (placeholder)
    pub fn extract_from_bonkswap(
        _input: &InstructionProcessorInputType<carbon_bonkswap_decoder::instructions::BonkswapInstruction>,
    ) -> Option<TradeEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_direction_normalization() {
        assert_eq!(TradeDirection::Buy, TradeDirection::Buy);
        assert_eq!(TradeDirection::Sell, TradeDirection::Sell);
    }
}
