use anchor_lang::prelude::*;
pub mod accounts_types;
pub mod mango_util;
use crate::accounts_types::*;
pub use mango;
pub use mango_common;

declare_id!("J8heqiEwQJs265mrMiCXjCZdDy8xAqNpBoyBRbnV3wmy");

#[program]
pub mod mango_strategy {

    pub const AUTHORITY_PDA_SEED: &[u8] = b"authority_account";
    pub const STRATEGY_DATA_PDA_SEED: &[u8] = b"strategy_account";
    pub const SPOT_PDA_SEED: &[u8] = b"spot_account";
    pub const VAULT_PDA_SEED: &[u8] = b"vault_account";
    pub const SERUM_PDA_SEED: &[u8] = b"serum_account";

    pub const MANGO_ACCOUNT_NUM: u64 = 1;

    use anchor_spl::token::Transfer;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bumps: Bumps) -> ProgramResult {
        ctx.accounts.strategy_account.owner_pk = ctx.accounts.owner.key();
        ctx.accounts.strategy_account.trigger_server_pk = ctx.accounts.trigger_server.key();

        let strategy_id = ctx.accounts.strategy_id.key();
        mango_util::create_account(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account.to_account_info(),
            &ctx.accounts.authority,
            &ctx.accounts.owner,
            &ctx.accounts.system_program,
            &[&[
                strategy_id.as_ref(),
                AUTHORITY_PDA_SEED,
                &[bumps.authority_bump],
            ]],
            MANGO_ACCOUNT_NUM,
        )?;

        mango_util::create_open_orders(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account.to_account_info(),
            &ctx.accounts.authority,
            &ctx.accounts.serum_dex,
            &ctx.accounts.spot_open_orders,
            &ctx.accounts.spot_market,
            &ctx.accounts.mango_signer,
            &ctx.accounts.owner,
            &ctx.accounts.system_program,
            &[&[
                strategy_id.as_ref(),
                AUTHORITY_PDA_SEED,
                &[bumps.authority_bump],
            ]],
        )?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, bumps: Bumps, amount: u64) -> ProgramResult {
        let accounts = Transfer {
            authority: ctx.accounts.owner.clone(),
            from: ctx.accounts.source_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
        };
        let strategy_id = ctx.accounts.strategy_id.key();
        let _bumps = &[bumps.authority_bump];
        let seeds = &[&[strategy_id.as_ref(), AUTHORITY_PDA_SEED, &_bumps[..]][..]];
        // Mango does not allow direct transfers
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accounts,
            seeds,
        );
        anchor_spl::token::transfer(transfer_ctx, amount)?;
        mango_util::deposit_tokens(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.mango_cache,
            &ctx.accounts.mango_root_bank,
            &ctx.accounts.mango_node_bank,
            &ctx.accounts.mango_vault,
            &ctx.accounts.authority,
            &ctx.accounts.token_program,
            &ctx.accounts.vault_token_account.to_account_info(),
            &[&[
                ctx.accounts.strategy_id.key().as_ref(),
                AUTHORITY_PDA_SEED,
                &[bumps.authority_bump],
            ]],
            amount,
        )?;
        Ok(())
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        bumps: Bumps,
        amount: u64,
        spot_market_index: u8,
    ) -> ProgramResult {
        mango_util::withdraw_tokens(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.mango_cache,
            &ctx.accounts.mango_root_bank,
            &ctx.accounts.mango_node_bank,
            &ctx.accounts.mango_vault,
            &ctx.accounts.mango_signer,
            &ctx.accounts.authority,
            &ctx.accounts.token_program,
            &ctx.accounts.destination_token_account.to_account_info(),
            &ctx.accounts.spot_open_orders,
            &[&[
                ctx.accounts.strategy_id.key().as_ref(),
                AUTHORITY_PDA_SEED,
                &[bumps.authority_bump],
            ]],
            amount,
            spot_market_index as usize,
        )?;
        Ok(())
    }

    /// amount > 0: increase long (or decrease short), amount < 0: increase short (or decrese long)
    pub fn adjust_position_perp(
        ctx: Context<AdjustPositionPerp>,
        bumps: Bumps,
        mango_market_index: u8,
        amount: i64,
        reduce_only: bool,
    ) -> ProgramResult {
        let side = if amount > 0 {
            mango::matching::Side::Bid
        } else {
            mango::matching::Side::Ask
        };
        mango_util::adjust_position_perp(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.authority,
            &ctx.accounts.mango_cache,
            &ctx.accounts.mango_market,
            &ctx.accounts.mango_bids,
            &ctx.accounts.mango_asks,
            &ctx.accounts.mango_event_queue,
            &ctx.accounts.spot_open_orders,
            &[&[
                ctx.accounts.strategy_id.key().as_ref(),
                AUTHORITY_PDA_SEED,
                &[bumps.authority_bump],
            ]],
            side,
            amount.abs(),
            mango_market_index as usize,
            reduce_only,
        )?;
        Ok(())
    }

    pub fn adjust_position_spot(
        ctx: Context<AdjustPositionSpot>,
        bumps: Bumps,
        amount: i64,
        market_lot_size: u64,
    ) -> ProgramResult {
        let side = if amount > 0 {
            serum_dex::matching::Side::Bid
        } else {
            serum_dex::matching::Side::Ask
        };
        mango_util::adjust_position_spot(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.authority,
            &ctx.accounts.mango_cache,
            &ctx.accounts.mango_signer,
            &ctx.accounts.serum_dex,
            &ctx.accounts.spot_market,
            &ctx.accounts.spot_bids,
            &ctx.accounts.spot_asks,
            &ctx.accounts.spot_request_queue,
            &ctx.accounts.spot_event_queue,
            &ctx.accounts.spot_base,
            &ctx.accounts.spot_quote,
            &ctx.accounts.spot_base_root_bank,
            &ctx.accounts.spot_base_node_bank,
            &ctx.accounts.spot_base_vault,
            &ctx.accounts.spot_quote_root_bank,
            &ctx.accounts.spot_quote_node_bank,
            &ctx.accounts.spot_quote_vault,
            &ctx.accounts.serum_dex_signer,
            &ctx.accounts.spot_open_orders,
            &ctx.accounts.srm_vault,
            &ctx.accounts.token_program,
            &[&[
                ctx.accounts.strategy_id.key().as_ref(),
                AUTHORITY_PDA_SEED,
                &[bumps.authority_bump],
            ]],
            side,
            amount.abs() as u64,
            market_lot_size,
        )?;
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Side {
    Long,
    Short,
}

#[error]
pub enum ErrorCode {
    InvalidAccount,
}
