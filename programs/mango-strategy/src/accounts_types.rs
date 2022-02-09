use crate::mango_strategy;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct Initialize<'info> {
    #[account(mut, signer)]
    pub owner: AccountInfo<'info>,

    pub strategy_id: AccountInfo<'info>,

    pub trigger_server: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::AUTHORITY_PDA_SEED],
        bump=bumps.authority_bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        init,
        seeds=[strategy_id.key().as_ref(), mango_strategy::STARTEGY_PDA_SEED],
        bump,
        payer = owner,
        space = StrategyAccount::LEN
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    // Vault
    pub vault_token_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        seeds=[strategy_id.key().as_ref(), mango_strategy::VAULT_PDA_SEED],
        bump,
        payer = owner,
        token::mint = vault_token_mint,
        token::authority = authority,
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    // Mango
    pub mango_program: AccountInfo<'info>,

    #[account(mut)]
    pub mango_group: AccountInfo<'info>,

    #[account(mut)] // Mango checks for correct PDA
    pub mango_account: AccountInfo<'info>,

    pub mango_signer: AccountInfo<'info>,

    // Spot
    pub serum_dex: AccountInfo<'info>,
    pub serum_market: AccountInfo<'info>,
    #[account(mut)]
    pub serum_open_orders: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct DepositToMango<'info> {
    pub strategy_id: AccountInfo<'info>,

    #[account(signer)]
    pub owner: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::AUTHORITY_PDA_SEED],
        bump=bumps.authority_bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::STARTEGY_PDA_SEED],
        bump=bumps.strategy_bump,
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    // Vault
    pub vault_token_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds=[strategy_id.key().as_ref(), mango_strategy::VAULT_PDA_SEED],
        bump=bumps.vault_bump,
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    // Mango
    pub mango_program: AccountInfo<'info>,

    #[account(mut)]
    pub mango_group: AccountInfo<'info>,

    #[account(mut)] // Mango checks for correct PDA
    pub mango_account: AccountInfo<'info>,

    pub mango_cache: AccountInfo<'info>,
    pub mango_root_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_node_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_vault: AccountInfo<'info>,

    pub mango_signer: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct AdjustPositionPerp<'info> {
    pub strategy_id: AccountInfo<'info>,

    #[account(signer)]
    pub trigger_server: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::AUTHORITY_PDA_SEED],
        bump=bumps.authority_bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::STARTEGY_PDA_SEED],
        bump=bumps.strategy_bump,
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    // Mango
    pub mango_program: AccountInfo<'info>,

    #[account(mut)]
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

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bumps: Bumps)]
pub struct AdjustPositionSpot<'info> {
    pub strategy_id: AccountInfo<'info>,

    #[account(signer)]
    pub trigger_server: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::AUTHORITY_PDA_SEED],
        bump=bumps.authority_bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(
        seeds=[strategy_id.key().as_ref(), mango_strategy::STARTEGY_PDA_SEED],
        bump=bumps.strategy_bump,
    )]
    pub strategy_account: Box<Account<'info, StrategyAccount>>,

    // Mango
    pub mango_program: AccountInfo<'info>,

    #[account(mut)]
    pub mango_group: AccountInfo<'info>,

    #[account(mut)] // Mango checks for correct PDA
    pub mango_account: AccountInfo<'info>,

    pub mango_cache: AccountInfo<'info>,
    pub mango_signer: AccountInfo<'info>,

    // Spot
    pub serum_dex: AccountInfo<'info>,
    #[account(mut)]
    pub serum_market: AccountInfo<'info>,
    #[account(mut)]
    pub serum_open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub serum_asks: AccountInfo<'info>,
    #[account(mut)]
    pub serum_bids: AccountInfo<'info>,
    #[account(mut)]
    pub serum_request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub serum_event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub serum_base: AccountInfo<'info>,
    #[account(mut)]
    pub serum_quote: AccountInfo<'info>,
    pub serum_base_root_bank: AccountInfo<'info>,
    #[account(mut)]
    pub serum_base_node_bank: AccountInfo<'info>,
    #[account(mut)]
    pub serum_base_vault: AccountInfo<'info>,
    pub serum_quote_root_bank: AccountInfo<'info>,
    #[account(mut)]
    pub serum_quote_node_bank: AccountInfo<'info>,
    #[account(mut)]
    pub serum_quote_vault: AccountInfo<'info>,
    pub serum_dex_signer: AccountInfo<'info>,
    pub srm_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]

pub struct Bumps {
    pub authority_bump: u8,
    pub strategy_bump: u8,
    pub mango_bump: u8,
    pub vault_bump: u8,
    pub serum_open_orders_bump: u8,
}

#[account]
#[derive(Debug, Default)]
pub struct StrategyAccount {
    pub owner_pk: Pubkey,
    pub trigger_server: Pubkey,
}

impl StrategyAccount {
    pub const LEN: usize = 2 * 32 + 8;
}
