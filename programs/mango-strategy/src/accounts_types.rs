use crate::{mango_strategy, MarketInfo};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct Initialize<'info> {
    #[account(signer, mut)]
    pub owner: AccountInfo<'info>,

    #[account(signer)]
    pub strategy_id: AccountInfo<'info>,

    pub trigger_server: AccountInfo<'info>,

    #[account(
        init,
        seeds=[strategy_id.key().as_ref(), mango_strategy::STRATEGY_ACCOUNT_PDA_SEED],
        bump,
        payer = owner,
        space = StrategyAccount::LEN
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    // Mango
    pub mango_program: AccountInfo<'info>,

    #[account(mut)]
    pub mango_group: AccountInfo<'info>,

    #[account(mut)] // Mango checks for correct PDA
    pub mango_account: AccountInfo<'info>,

    pub mango_signer: AccountInfo<'info>,

    // Spot
    pub serum_dex: AccountInfo<'info>,
    pub spot_market: AccountInfo<'info>,
    #[account(mut)]
    pub spot_open_orders: AccountInfo<'info>,

    // Vault
    pub vault_token_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        seeds=[strategy_id.key().as_ref(), mango_strategy::VAULT_PDA_SEED],
        bump,
        payer = owner,
        token::mint = vault_token_mint,
        token::authority = strategy_account,
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    // Strategy token
    #[account(
        init,
        payer = owner,
        seeds=[strategy_id.key().as_ref(), mango_strategy::MINT_PDA_SEED],
        bump,
        mint::decimals = mango_strategy::STRATEGY_TOKEN_DECIMALS,
        mint::authority = strategy_account
    )]
    pub strategy_token_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct Deposit<'info> {
    pub strategy_id: AccountInfo<'info>,

    #[account(signer, mut)]
    pub owner: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::STRATEGY_ACCOUNT_PDA_SEED],
        bump=bumps.strategy_account_bump,
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    // Mango
    #[account(address = strategy_account.mango_program)]
    pub mango_program: AccountInfo<'info>,

    #[account(mut, address = strategy_account.mango_group)]
    pub mango_group: AccountInfo<'info>,

    #[account(mut)] // Mango checks for correct PDA
    pub mango_account: AccountInfo<'info>,

    pub mango_cache: AccountInfo<'info>,
    pub mango_root_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_node_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_vault: AccountInfo<'info>,

    // Vault (mango does not allow direct deposit from token accounts not owned by signer)
    #[account(
        mut,
        seeds=[strategy_id.key().as_ref(), mango_strategy::VAULT_PDA_SEED],
        bump
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    // Deposit token
    #[account(
        mut,
        has_one = owner,
        constraint = deposit_token_account.mint == strategy_account.vault_token_mint
    )]
    pub deposit_token_account: Box<Account<'info, TokenAccount>>,

    // Strategy token
    #[account(
        mut,
        seeds=[strategy_id.key().as_ref(), mango_strategy::MINT_PDA_SEED],
        bump,
    )]
    pub strategy_token_mint: Box<Account<'info, Mint>>,

    #[account(mut, constraint = strategy_token_account.mint == strategy_token_mint.to_account_info().key())]
    pub strategy_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct Withdraw<'info> {
    pub strategy_id: AccountInfo<'info>,

    #[account(signer, mut)]
    pub owner: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::STRATEGY_ACCOUNT_PDA_SEED],
        bump=bumps.strategy_account_bump,
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    // Mango
    #[account(address = strategy_account.mango_program)]
    pub mango_program: AccountInfo<'info>,

    #[account(mut, address = strategy_account.mango_group)]
    pub mango_group: AccountInfo<'info>,

    #[account(mut)] // Mango already checks for correct PDA
    pub mango_account: AccountInfo<'info>,

    pub mango_cache: AccountInfo<'info>,
    pub mango_root_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_node_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_vault: AccountInfo<'info>,

    pub mango_signer: AccountInfo<'info>,

    #[account(mut)]
    pub spot_open_orders: AccountInfo<'info>,

    // Withdraw token
    #[account(
        mut,
        has_one = owner,
        constraint = withdraw_token_account.mint == strategy_account.vault_token_mint
    )]
    pub withdraw_token_account: Box<Account<'info, TokenAccount>>,

    // Strategy token
    #[account(
        mut,
        seeds=[strategy_id.key().as_ref(), mango_strategy::MINT_PDA_SEED],
        bump,
    )]
    pub strategy_token_mint: Box<Account<'info, Mint>>,

    #[account(mut, constraint = strategy_token_account.mint == strategy_token_mint.to_account_info().key())]
    pub strategy_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct AdjustPositionPerp<'info> {
    pub strategy_id: AccountInfo<'info>,

    #[account(signer, address = strategy_account.trigger_server_pk)]
    pub trigger_server: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::STRATEGY_ACCOUNT_PDA_SEED],
        bump=bumps.strategy_account_bump,
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    // Mango
    #[account(address = strategy_account.mango_program)]
    pub mango_program: AccountInfo<'info>,

    #[account(mut, address = strategy_account.mango_group)]
    pub mango_group: AccountInfo<'info>,

    #[account(mut)] // Mango checks for correct PDA
    pub mango_account: AccountInfo<'info>,

    pub mango_cache: AccountInfo<'info>,
    pub mango_root_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_node_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_vault: AccountInfo<'info>,
    #[account(mut)]
    pub mango_market: AccountInfo<'info>,
    #[account(mut)]
    pub mango_asks: AccountInfo<'info>,
    #[account(mut)]
    pub mango_bids: AccountInfo<'info>,
    #[account(mut)]
    pub mango_event_queue: AccountInfo<'info>,
    pub mango_signer: AccountInfo<'info>,

    pub spot_open_orders: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct AdjustPositionSpot<'info> {
    pub strategy_id: AccountInfo<'info>,

    #[account(signer, address = strategy_account.trigger_server_pk)]
    pub trigger_server: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::STRATEGY_ACCOUNT_PDA_SEED],
        bump=bumps.strategy_account_bump,
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    // Mango
    #[account(address = strategy_account.mango_program)]
    pub mango_program: AccountInfo<'info>,

    #[account(mut, address = strategy_account.mango_group)]
    pub mango_group: AccountInfo<'info>,

    #[account(mut)] // Mango checks for correct PDA
    pub mango_account: AccountInfo<'info>,

    pub mango_cache: AccountInfo<'info>,
    pub mango_signer: AccountInfo<'info>,

    // Spot
    pub serum_dex: AccountInfo<'info>,
    #[account(mut)]
    pub spot_market: AccountInfo<'info>,
    #[account(mut)]
    pub spot_open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub spot_asks: AccountInfo<'info>,
    #[account(mut)]
    pub spot_bids: AccountInfo<'info>,
    #[account(mut)]
    pub spot_request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub spot_event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub spot_base: AccountInfo<'info>,
    #[account(mut)]
    pub spot_quote: AccountInfo<'info>,
    pub spot_base_root_bank: AccountInfo<'info>,
    #[account(mut)]
    pub spot_base_node_bank: AccountInfo<'info>,
    #[account(mut)]
    pub spot_base_vault: AccountInfo<'info>,
    pub spot_quote_root_bank: AccountInfo<'info>,
    #[account(mut)]
    pub spot_quote_node_bank: AccountInfo<'info>,
    #[account(mut)]
    pub spot_quote_vault: AccountInfo<'info>,
    pub serum_dex_signer: AccountInfo<'info>,
    pub srm_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct SetLimits<'info> {
    pub strategy_id: AccountInfo<'info>,

    #[account(signer, mut, address = strategy_account.owner)]
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds=[strategy_id.key().as_ref(), mango_strategy::STRATEGY_ACCOUNT_PDA_SEED],
        bump=bumps.strategy_account_bump,
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    #[account(
        signer,
        init,
        payer = owner,
        space = LimitsAccount::LEN,
        constraint = strategy_account.limits_account.is_none() || strategy_account.limits_account == Some(limits_account.key()))
    ]
    pub limits_account: Box<Account<'info, LimitsAccount>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct DropLimits<'info> {
    pub strategy_id: AccountInfo<'info>,

    #[account(signer, address = strategy_account.owner)]
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds=[strategy_id.key().as_ref(), mango_strategy::STRATEGY_ACCOUNT_PDA_SEED],
        bump=bumps.strategy_account_bump,
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    #[account(
        mut,
        constraint = strategy_account.limits_account == Some(limits_account.key()))
    ]
    pub limits_account: Box<Account<'info, LimitsAccount>>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]

pub struct Bumps {
    pub strategy_account_bump: u8,
}

#[account]
#[derive(Debug)]
pub struct StrategyAccount {
    pub owner: Pubkey, // Owner can only change limits
    pub trigger_server_pk: Pubkey,
    pub vault_token_mint: Pubkey,
    pub mango_program: Pubkey,
    pub mango_group: Pubkey,
    pub limits_account: Option<Pubkey>,
    pub market_info: MarketInfo,
}

impl StrategyAccount {
    pub const LEN: usize = 6 * 32 + 12 + 8;
}

#[account]
#[derive(Debug, Default)]
pub struct LimitsAccount {
    // in USDC including decimals
    pub max_tvl: Option<u64>,
    pub whitelist: Vec<Pubkey>,
}

impl LimitsAccount {
    pub const WHITELIST_CAP: usize = 16;
    pub const LEN: usize = 17 + LimitsAccount::WHITELIST_CAP * 32 + 8;
}
