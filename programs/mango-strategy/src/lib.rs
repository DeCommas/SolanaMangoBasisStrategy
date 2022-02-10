use anchor_lang::prelude::*;
pub mod accounts_types;
pub mod mango_util;
use crate::accounts_types::*;

declare_id!("J8heqiEwQJs265mrMiCXjCZdDy8xAqNpBoyBRbnV3wmy");

#[program]
pub mod mango_strategy {

    pub const AUTHORITY_PDA_SEED: &[u8] = b"authority_account";
    pub const STARTEGY_PDA_SEED: &[u8] = b"strategy_account";
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
            &ctx.accounts.serum_open_orders,
            &ctx.accounts.serum_market,
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

    pub fn withdraw(ctx: Context<Withdraw>, bumps: Bumps, amount: u64) -> ProgramResult {
        let accounts = Transfer {
            authority: ctx.accounts.authority.clone(),
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
        };
        let strategy_id = ctx.accounts.strategy_id.key();
        let bumps = &[bumps.authority_bump];
        let seeds = &[&[strategy_id.as_ref(), AUTHORITY_PDA_SEED, &bumps[..]][..]];
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accounts,
            seeds,
        );
        anchor_spl::token::transfer(transfer_ctx, amount)?;
        Ok(())
    }

    /// amount > 0: deposit from vault to mango account, amount < 0: withdraw from mango account to vault
    pub fn rebalance_mango(
        ctx: Context<RebalanceMango>,
        bumps: Bumps,
        amount: i64,
    ) -> ProgramResult {
        if amount > 0 {
            mango_util::deposit_usdc(
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
                amount.abs() as u64,
            )?;
        } else if amount < 0 {
            mango_util::withdraw_usdc(
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
                &ctx.accounts.vault_token_account.to_account_info(),
                &[&[
                    ctx.accounts.strategy_id.key().as_ref(),
                    AUTHORITY_PDA_SEED,
                    &[bumps.authority_bump],
                ]],
                amount.abs() as u64,
            )?;
        }
        Ok(())
    }

    pub fn adjust_position_perp(
        ctx: Context<AdjustPositionPerp>,
        bumps: Bumps,
        mango_market_index: u8,
        amount: i64,
    ) -> ProgramResult {
        let side = if amount > 0 {
            mango::matching::Side::Ask
        } else {
            mango::matching::Side::Bid
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
            &[&[
                ctx.accounts.strategy_id.key().as_ref(),
                AUTHORITY_PDA_SEED,
                &[bumps.authority_bump],
            ]],
            side,
            amount as i64,
            mango_market_index as usize,
        )?;
        Ok(())
    }

    pub fn adjust_position_spot(
        ctx: Context<AdjustPositionSpot>,
        bumps: Bumps,
        amount: i64,
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
            &ctx.accounts.serum_market,
            &ctx.accounts.serum_bids,
            &ctx.accounts.serum_asks,
            &ctx.accounts.serum_request_queue,
            &ctx.accounts.serum_event_queue,
            &ctx.accounts.serum_base,
            &ctx.accounts.serum_quote,
            &ctx.accounts.serum_base_root_bank,
            &ctx.accounts.serum_base_node_bank,
            &ctx.accounts.serum_base_vault,
            &ctx.accounts.serum_quote_root_bank,
            &ctx.accounts.serum_quote_node_bank,
            &ctx.accounts.serum_quote_vault,
            &ctx.accounts.serum_dex_signer,
            &ctx.accounts.serum_open_orders,
            &ctx.accounts.srm_vault,
            &ctx.accounts.token_program,
            &[&[
                ctx.accounts.strategy_id.key().as_ref(),
                AUTHORITY_PDA_SEED,
                &[bumps.authority_bump],
            ]],
            side,
            amount.abs() as u64,
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
