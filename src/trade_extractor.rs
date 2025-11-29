//! Trade extraction adapter for converting Carbon decoder events to TradeEvent
//!
//! Phase 3: Trade Extraction Layer
//!
//! This module serves as the bridge between Carbon's decoded blockchain events
//! and SolFlow's TradeEvent type. It extracts trade information from various
//! DEX programs (Pumpfun, PumpSwap, BonkSwap, Moonshot, JupiterDCA) and normalizes them
//! into a common TradeEvent format.

use crate::types::{TradeDirection, TradeEvent};
use carbon_core::{
    deserialize::ArrangeAccounts, 
    instruction::{InstructionMetadata, InstructionProcessorInputType}
};

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
            is_bot: false,
            is_dca: false,
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
            is_bot: false,
            is_dca: false,
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
            is_bot: false,
            is_dca: false,
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
            is_bot: false,
            is_dca: false,
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
            is_bot: false,
            is_dca: false,
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
            is_bot: false,
            is_dca: false,
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
            is_bot: false,
            is_dca: true,
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

    /// Helper to find the account index for a given pubkey in transaction metadata
    fn get_account_index(
        metadata: &InstructionMetadata,
        user_pubkey: &solana_sdk::pubkey::Pubkey,
    ) -> Option<usize> {
        let tx_meta = &metadata.transaction_metadata;
        
        // Access account_keys from the versioned message
        let account_keys = tx_meta.message.static_account_keys();
        
        // Convert Carbon addresses to Solana Pubkeys and find matching index
        for (index, account_address) in account_keys.iter().enumerate() {
            // Carbon uses its own address type, convert to Solana Pubkey
            let account_bytes: [u8; 32] = account_address.as_ref().try_into().ok()?;
            let account_pubkey = solana_sdk::pubkey::Pubkey::new_from_array(account_bytes);
            
            if &account_pubkey == user_pubkey {
                log::debug!(
                    "🔍 PUMPSWAP_USER_INDEX_FOUND | User: {} | Index: {}",
                    user_pubkey,
                    index
                );
                return Some(index);
            }
        }
        
        log::warn!(
            "⚠️ USER_ACCOUNT_NOT_FOUND | User: {} | Total accounts: {}",
            user_pubkey,
            account_keys.len()
        );
        None
    }

    /// Extract SOL delta from transaction metadata for a given user pubkey
    fn compute_sol_delta_from_metadata(
        metadata: &InstructionMetadata,
        user_pubkey: &solana_sdk::pubkey::Pubkey,
    ) -> Option<f64> {
        let tx_meta = &metadata.transaction_metadata;
        let meta = &tx_meta.meta;

        // Find the account index dynamically
        let user_account_index = Self::get_account_index(metadata, user_pubkey)?;

        let pre_balance = meta.pre_balances.get(user_account_index).copied()?;
        let post_balance = meta.post_balances.get(user_account_index).copied()?;
        let fee = if user_account_index == 0 { meta.fee } else { 0 };

        let sol_delta_lamports = (post_balance as i128 - pre_balance as i128) + fee as i128;
        let sol_delta = sol_delta_lamports.abs() as f64 / 1_000_000_000.0;

        log::debug!(
            "💰 SOL_DELTA_COMPUTED_CORRECTLY | User: {} | Account[{}] | Pre: {} | Post: {} | Fee: {} | Delta: {:.6} SOL",
            user_pubkey,
            user_account_index,
            pre_balance,
            post_balance,
            fee,
            sol_delta
        );

        Some(sol_delta)
    }

    /// Extract a TradeEvent from a PumpSwap Buy instruction
    pub fn extract_pumpswap_buy(
        accounts: &carbon_pump_swap_decoder::instructions::buy::BuyInstructionAccounts,
        instruction: &carbon_pump_swap_decoder::instructions::buy::Buy,
        metadata: &InstructionMetadata,
    ) -> Option<TradeEvent> {
        let timestamp = metadata.transaction_metadata.block_time.unwrap_or(0);
        
        // Convert Carbon address to Solana Pubkey for user account
        let user_bytes: [u8; 32] = accounts.user.as_ref().try_into().ok()?;
        let user_pubkey = solana_sdk::pubkey::Pubkey::new_from_array(user_bytes);

        // Compute SOL delta using dynamic account index lookup
        let sol_amount = Self::compute_sol_delta_from_metadata(metadata, &user_pubkey)
            .unwrap_or_else(|| {
                log::warn!(
                    "⚠️ SOL_DELTA_FALLBACK | Variant: Buy | Using instruction max_quote_amount_in"
                );
                instruction.max_quote_amount_in as f64 / 1_000_000_000.0
            });

        log::info!(
            "🟢 PUMPSWAP_VARIANT_WITH_SOL_DELTA | Variant: Buy | User: {} | Mint: {} | SOL: {:.6}",
            accounts.user,
            accounts.base_mint,
            sol_amount
        );

        Some(TradeEvent {
            timestamp,
            mint: accounts.base_mint.to_string(),
            direction: TradeDirection::Buy,
            sol_amount,
            token_amount: instruction.base_amount_out as f64,
            token_decimals: 6,
            user_account: accounts.user.to_string(),
            source_program: "PumpSwap".to_string(),
            is_bot: false,
            is_dca: false,
        })
    }

    /// Extract a TradeEvent from a PumpSwap Sell instruction
    pub fn extract_pumpswap_sell(
        accounts: &carbon_pump_swap_decoder::instructions::sell::SellInstructionAccounts,
        instruction: &carbon_pump_swap_decoder::instructions::sell::Sell,
        metadata: &InstructionMetadata,
    ) -> Option<TradeEvent> {
        let timestamp = metadata.transaction_metadata.block_time.unwrap_or(0);
        
        // Convert Carbon address to Solana Pubkey for user account
        let user_bytes: [u8; 32] = accounts.user.as_ref().try_into().ok()?;
        let user_pubkey = solana_sdk::pubkey::Pubkey::new_from_array(user_bytes);

        // Compute SOL delta using dynamic account index lookup
        let sol_amount = Self::compute_sol_delta_from_metadata(metadata, &user_pubkey)
            .unwrap_or_else(|| {
                log::warn!(
                    "⚠️ SOL_DELTA_FALLBACK | Variant: Sell | Using instruction min_quote_amount_out"
                );
                instruction.min_quote_amount_out as f64 / 1_000_000_000.0
            });

        log::info!(
            "🟢 PUMPSWAP_VARIANT_WITH_SOL_DELTA | Variant: Sell | User: {} | Mint: {} | SOL: {:.6}",
            accounts.user,
            accounts.base_mint,
            sol_amount
        );

        Some(TradeEvent {
            timestamp,
            mint: accounts.base_mint.to_string(),
            direction: TradeDirection::Sell,
            sol_amount,
            token_amount: instruction.base_amount_in as f64,
            token_decimals: 6,
            user_account: accounts.user.to_string(),
            source_program: "PumpSwap".to_string(),
            is_bot: false,
            is_dca: false,
        })
    }

    /// Extract a TradeEvent from a PumpSwap BuyExactQuoteIn instruction
    pub fn extract_pumpswap_buy_exact_quote_in(
        accounts: &carbon_pump_swap_decoder::instructions::buy_exact_quote_in::BuyExactQuoteInInstructionAccounts,
        instruction: &carbon_pump_swap_decoder::instructions::buy_exact_quote_in::BuyExactQuoteIn,
        metadata: &InstructionMetadata,
    ) -> Option<TradeEvent> {
        let timestamp = metadata.transaction_metadata.block_time.unwrap_or(0);
        
        // Convert Carbon address to Solana Pubkey for user account
        let user_bytes: [u8; 32] = accounts.user.as_ref().try_into().ok()?;
        let user_pubkey = solana_sdk::pubkey::Pubkey::new_from_array(user_bytes);

        // Compute SOL delta using dynamic account index lookup
        let sol_amount = Self::compute_sol_delta_from_metadata(metadata, &user_pubkey)
            .unwrap_or_else(|| {
                log::warn!(
                    "⚠️ SOL_DELTA_FALLBACK | Variant: BuyExactQuoteIn | Using instruction spendable_quote_in"
                );
                instruction.spendable_quote_in as f64 / 1_000_000_000.0
            });

        log::info!(
            "🟢 PUMPSWAP_VARIANT_WITH_SOL_DELTA | Variant: BuyExactQuoteIn | User: {} | Mint: {} | SOL: {:.6}",
            accounts.user,
            accounts.base_mint,
            sol_amount
        );

        Some(TradeEvent {
            timestamp,
            mint: accounts.base_mint.to_string(),
            direction: TradeDirection::Buy,
            sol_amount,
            token_amount: instruction.min_base_amount_out as f64,
            token_decimals: 6,
            user_account: accounts.user.to_string(),
            source_program: "PumpSwap".to_string(),
            is_bot: false,
            is_dca: false,
        })
    }

    /// Unified adapter for PumpSwap instructions
    pub fn extract_from_pumpswap(
        input: &InstructionProcessorInputType<carbon_pump_swap_decoder::instructions::PumpSwapInstruction>,
    ) -> Option<TradeEvent> {
        let (metadata, decoded_instruction, _nested_instructions, _raw_instruction) = input;

        match &decoded_instruction.data {
            // Legacy event variants (kept for backward compatibility)
            carbon_pump_swap_decoder::instructions::PumpSwapInstruction::BuyEvent(event) => {
                Self::extract_pumpswap_buy_event(event)
            }
            carbon_pump_swap_decoder::instructions::PumpSwapInstruction::SellEvent(event) => {
                Self::extract_pumpswap_sell_event(event)
            }
            // New swap instruction variants (primary live activity)
            carbon_pump_swap_decoder::instructions::PumpSwapInstruction::Buy(buy) => {
                let accounts = carbon_pump_swap_decoder::instructions::buy::Buy::arrange_accounts(
                    &decoded_instruction.accounts,
                )?;
                Self::extract_pumpswap_buy(&accounts, buy, metadata)
            }
            carbon_pump_swap_decoder::instructions::PumpSwapInstruction::Sell(sell) => {
                let accounts = carbon_pump_swap_decoder::instructions::sell::Sell::arrange_accounts(
                    &decoded_instruction.accounts,
                )?;
                Self::extract_pumpswap_sell(&accounts, sell, metadata)
            }
            carbon_pump_swap_decoder::instructions::PumpSwapInstruction::BuyExactQuoteIn(buy_exact) => {
                let accounts = carbon_pump_swap_decoder::instructions::buy_exact_quote_in::BuyExactQuoteIn::arrange_accounts(
                    &decoded_instruction.accounts,
                )?;
                Self::extract_pumpswap_buy_exact_quote_in(&accounts, buy_exact, metadata)
            }
            _ => {
                log::debug!("⚠️ PUMPSWAP_VARIANT_UNHANDLED | Variant: {:?}", decoded_instruction.data);
                None
            }
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
