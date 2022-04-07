use anchor_lang::prelude::*;
use mango::error::MangoError;
pub mod accounts_types;
pub mod mango_util;
use crate::accounts_types::*;
use crate::mango_util::calculate_token_price;
use anchor_spl::token::Transfer;
use fixed::types::I80F48;
pub use mango;
pub use mango_common;

declare_id!("DNyxh1hUP2TmLr6mh4yHEyWNPNiZsiUh6vY3snRi9M8F");

#[program]
pub mod mango_strategy {
    use anchor_spl::token::{burn, mint_to, Burn, MintTo};
    use az::{Cast, CheckedCast};
    use solana_program::entrypoint::ProgramResult;

    use crate::mango_util::calculate_tvl;

    use super::*;

    pub const STRATEGY_ACCOUNT_PDA_SEED: &[u8] = b"account";
    pub const VAULT_PDA_SEED: &[u8] = b"vault";
    pub const MINT_PDA_SEED: &[u8] = b"mint";

    pub const MANGO_ACCOUNT_NUM: u64 = 1;
    pub const STRATEGY_TOKEN_DECIMALS: u8 = 6; // same as USDC

    pub fn initialize(
        ctx: Context<Initialize>,
        bumps: Bumps,
        market_info: MarketInfo,
        limits_account: Option<Pubkey>,
    ) -> ProgramResult {
        ctx.accounts.strategy_account.owner = ctx.accounts.deployer.key();
        ctx.accounts.strategy_account.trigger_server_pk = ctx.accounts.trigger_server.key();
        ctx.accounts.strategy_account.vault_token_mint = ctx.accounts.vault_token_mint.key();
        ctx.accounts.strategy_account.mango_program = ctx.accounts.mango_program.key();
        ctx.accounts.strategy_account.mango_group = ctx.accounts.mango_group.key();
        ctx.accounts.strategy_account.limits_account = limits_account;
        ctx.accounts.strategy_account.market_info = market_info;

        let strategy_id = ctx.accounts.strategy_id.key();
        mango_util::create_account(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account.to_account_info(),
            &ctx.accounts.strategy_account.to_account_info(),
            &ctx.accounts.deployer,
            &ctx.accounts.system_program,
            &[&[
                strategy_id.as_ref(),
                STRATEGY_ACCOUNT_PDA_SEED,
                &[bumps.strategy_account_bump],
            ]],
            MANGO_ACCOUNT_NUM,
        )?;

        mango_util::create_open_orders(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account.to_account_info(),
            &ctx.accounts.strategy_account.to_account_info(),
            &ctx.accounts.serum_dex,
            &ctx.accounts.spot_open_orders,
            &ctx.accounts.spot_market,
            &ctx.accounts.mango_signer,
            &ctx.accounts.deployer,
            &ctx.accounts.system_program,
            &[&[
                strategy_id.as_ref(),
                STRATEGY_ACCOUNT_PDA_SEED,
                &[bumps.strategy_account_bump],
            ]],
        )?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, bumps: Bumps, vault_token_amount: u64) -> Result<()> {
        let tvl = calculate_tvl(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.mango_cache,
            &ctx.accounts.strategy_account.market_info,
        )
        .map_err(ErrorCode::register_mango_error)?;
        if let Some(limits_account) = ctx.accounts.strategy_account.limits_account {
            let limits_account_info = ctx
                .remaining_accounts
                .iter()
                .find(|acc| acc.key() == limits_account)
                .ok_or(ErrorCode::InvalidLimitsAccount)?; // check limits account
            let mut limits_account: LimitsAccount =
                LimitsAccount::try_deserialize(&mut &limits_account_info.data.borrow_mut()[..])
                    .map_err(|_| ErrorCode::InvalidLimitsAccount)?;
            if limits_account
                .max_tvl
                .map(|max_tvl_limit| (tvl + I80F48::from_num(vault_token_amount)) >= max_tvl_limit)
                == Some(true)
            {
                return Err(ErrorCode::TvlLimitReached.into());
            }
            let limit = limits_account
                .whitelist
                .iter_mut()
                .find(|x| x.key == ctx.accounts.owner.key());
            if let Some(WhitelistLimit { deposit, .. }) = limit {
                *deposit += vault_token_amount;
                if *deposit > limits_account.max_deposit {
                    return Err(ErrorCode::WhitelistLimitReached.into());
                }
                LimitsAccount::try_serialize(
                    &limits_account,
                    &mut &mut limits_account_info.data.borrow_mut()[..],
                )?;
            } else {
                return Err(ErrorCode::NotInWhitelist.into());
            }
        }
        let token_price = calculate_token_price(&ctx.accounts.strategy_token_mint, tvl)
            .map_err(ErrorCode::register_mango_error)?;
        let strategy_token_amount = I80F48::from_num(vault_token_amount) / token_price;
        let accounts = Transfer {
            authority: ctx.accounts.owner.clone(),
            from: ctx.accounts.deposit_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
        };
        let strategy_id = ctx.accounts.strategy_id.key();
        let bump = &[bumps.strategy_account_bump];
        let seeds = &[&[strategy_id.as_ref(), STRATEGY_ACCOUNT_PDA_SEED, &bump[..]][..]];
        // Mango does not allow direct transfers
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accounts,
            seeds,
        );
        anchor_spl::token::transfer(transfer_ctx, vault_token_amount)?;
        mango_util::deposit_tokens(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.mango_cache,
            &ctx.accounts.mango_root_bank,
            &ctx.accounts.mango_node_bank,
            &ctx.accounts.mango_vault,
            &ctx.accounts.strategy_account.to_account_info(),
            &ctx.accounts.token_program,
            &ctx.accounts.vault_token_account.to_account_info(),
            &[&[
                ctx.accounts.strategy_id.key().as_ref(),
                STRATEGY_ACCOUNT_PDA_SEED,
                &[bumps.strategy_account_bump],
            ]],
            vault_token_amount,
        )?;

        let cpi_accounts = MintTo {
            mint: ctx.accounts.strategy_token_mint.to_account_info(),
            to: ctx.accounts.strategy_token_account.to_account_info(),
            authority: ctx.accounts.strategy_account.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            seeds,
        );
        mint_to(
            cpi_context,
            strategy_token_amount
                .checked_cast()
                .expect("strategy_token_amount cast failed"),
        )?;
        Ok(())
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        bumps: Bumps,
        strategy_token_amount: u64,
    ) -> Result<()> {
        let tvl = calculate_tvl(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.mango_cache,
            &ctx.accounts.strategy_account.market_info,
        )
        .map_err(ErrorCode::register_mango_error)?;
        let token_price = calculate_token_price(&ctx.accounts.strategy_token_mint, tvl)
            .map_err(ErrorCode::register_mango_error)?;
        let vault_token_amount = I80F48::from_num(strategy_token_amount) * token_price;
        if let Some(limits_account) = ctx.accounts.strategy_account.limits_account {
            let limits_account_info = ctx
                .remaining_accounts
                .iter()
                .find(|acc| acc.key() == limits_account)
                .ok_or(ErrorCode::InvalidLimitsAccount)?; // check limits account
            let mut limits_account: LimitsAccount =
                LimitsAccount::try_deserialize(&mut &limits_account_info.data.borrow_mut()[..])
                    .map_err(|_| ErrorCode::InvalidLimitsAccount)?;
            let limit = limits_account
                .whitelist
                .iter_mut()
                .find(|x| x.key == ctx.accounts.owner.key());
            if let Some(WhitelistLimit { deposit, .. }) = limit {
                if vault_token_amount >= *deposit {
                    *deposit = 0;
                } else {
                    let vault_token_amount_u64: u64 = vault_token_amount.cast();
                    *deposit -= vault_token_amount_u64;
                }
                LimitsAccount::try_serialize(
                    &limits_account,
                    &mut &mut limits_account_info.data.borrow_mut()[..],
                )?;
            } else {
                return Err(ErrorCode::NotInWhitelist.into());
            }
        }
        mango_util::withdraw_tokens(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.mango_cache,
            &ctx.accounts.mango_root_bank,
            &ctx.accounts.mango_node_bank,
            &ctx.accounts.mango_vault,
            &ctx.accounts.mango_signer,
            &ctx.accounts.strategy_account.to_account_info(),
            &ctx.accounts.token_program,
            &ctx.accounts.withdraw_token_account.to_account_info(),
            &ctx.accounts.spot_open_orders,
            &[&[
                ctx.accounts.strategy_id.key().as_ref(),
                STRATEGY_ACCOUNT_PDA_SEED,
                &[bumps.strategy_account_bump],
            ]],
            vault_token_amount
                .checked_cast()
                .expect("vault_token_amount cast failed"),
            ctx.accounts.strategy_account.market_info.spot_market_index as usize,
        )?;
        let cpi_accounts = Burn {
            mint: ctx.accounts.strategy_token_mint.to_account_info(),
            to: ctx.accounts.strategy_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_context =
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        burn(cpi_context, strategy_token_amount)?;
        Ok(())
    }

    /// amount > 0: increase long (or decrease short), amount < 0: increase short (or decrese long)
    pub fn adjust_position_perp(
        ctx: Context<AdjustPositionPerp>,
        bumps: Bumps,
        amount: i64,
        reduce_only: bool,
    ) -> ProgramResult {
        assert_ne!(amount, 0, "Amount should not be zero");
        let side = if amount > 0 {
            mango::matching::Side::Bid
        } else {
            mango::matching::Side::Ask
        };
        mango_util::adjust_position_perp(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.strategy_account.to_account_info(),
            &ctx.accounts.mango_cache,
            &ctx.accounts.mango_market,
            &ctx.accounts.mango_bids,
            &ctx.accounts.mango_asks,
            &ctx.accounts.mango_event_queue,
            &ctx.accounts.spot_open_orders,
            &[&[
                ctx.accounts.strategy_id.key().as_ref(),
                STRATEGY_ACCOUNT_PDA_SEED,
                &[bumps.strategy_account_bump],
            ]],
            side,
            amount.abs(),
            ctx.accounts.strategy_account.market_info.perp_market_index as usize,
            reduce_only,
        )?;
        Ok(())
    }

    pub fn adjust_position_spot(
        ctx: Context<AdjustPositionSpot>,
        bumps: Bumps,
        amount: i64,
    ) -> ProgramResult {
        assert_ne!(amount, 0, "Amount should not be zero");
        let side = if amount > 0 {
            serum_dex::matching::Side::Bid
        } else {
            serum_dex::matching::Side::Ask
        };
        mango_util::adjust_position_spot(
            &ctx.accounts.mango_program,
            &ctx.accounts.mango_group,
            &ctx.accounts.mango_account,
            &ctx.accounts.strategy_account.to_account_info(),
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
                STRATEGY_ACCOUNT_PDA_SEED,
                &[bumps.strategy_account_bump],
            ]],
            side,
            amount.abs() as u64,
            ctx.accounts
                .strategy_account
                .market_info
                .spot_market_lot_size,
        )?;
        Ok(())
    }

    pub fn set_limits(
        ctx: Context<SetLimits>,
        bumps: Bumps,
        max_tvl: Option<u64>,
        max_deposit: u64,
        whitelist: Vec<WhitelistLimit>,
    ) -> ProgramResult {
        ctx.accounts.limits_account.max_tvl = max_tvl;
        ctx.accounts.limits_account.max_deposit = max_deposit;
        ctx.accounts.limits_account.whitelist = whitelist;
        ctx.accounts.strategy_account.limits_account = Some(ctx.accounts.limits_account.key());
        let _ = bumps; // bumps used in validation
        Ok(())
    }

    pub fn drop_limits(ctx: Context<DropLimits>, bumps: Bumps) -> ProgramResult {
        ctx.accounts.strategy_account.limits_account = None;
        let _ = bumps; // bumps used in validation
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarketInfo {
    pub perp_market_index: u8,
    pub spot_market_index: u8,
    pub spot_market_lot_size: u64,
    pub spot_token_index: u8,
}

#[error_code]
pub enum ErrorCode {
    InvalidLimitsAccount,
    TvlLimitReached,
    WhitelistLimitReached,
    NotInWhitelist,
    MangoError,
}

impl ErrorCode {
    pub fn register_mango_error(e: MangoError) -> Self {
        solana_program::log::sol_log(&format!("Mango error: {}", e));
        ErrorCode::MangoError
    }
}
