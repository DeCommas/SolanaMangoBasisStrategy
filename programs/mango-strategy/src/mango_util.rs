use std::num::NonZeroU64;

use anchor_lang::{
    prelude::{AccountInfo, ProgramError, ProgramResult},
    Key, ToAccountMetas,
};
use az::Cast;
use fixed::types::I80F48;
use mango::{
    instruction::{
        consume_events, create_mango_account, create_spot_open_orders, deposit, place_perp_order,
        withdraw, MangoInstruction,
    },
    matching::{OrderType, Side as MangoSide},
    state::MangoCache,
};
use mango_common::Loadable;
use serum_dex::{
    instruction::{NewOrderInstructionV3, SelfTradeBehavior},
    matching::Side as SerumSide,
};
use solana_program::{
    instruction::Instruction,
    program::{invoke, invoke_signed},
};

const NON_ZERO_MAX: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(u64::MAX) };

pub fn create_account<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    account_num: u64,
) -> ProgramResult {
    let instruction = create_mango_account(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &owner.key(),
        &system_program.key(),
        &payer.key(),
        account_num,
    )?;
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            owner.to_owned(),
            system_program.to_owned(),
            payer.to_owned(),
        ],
        seeds,
    )?;
    Ok(())
}

pub fn create_open_orders<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    serum_dex: &AccountInfo<'info>,
    serum_open_orders: &AccountInfo<'info>,
    serum_market: &AccountInfo<'info>,
    mango_signer: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let instruction = create_spot_open_orders(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &owner.key(),
        &serum_dex.key(),
        &serum_open_orders.key(),
        &serum_market.key(),
        &mango_signer.key(),
        &payer.key(),
    )?;
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            owner.to_owned(),
            serum_dex.to_owned(),
            serum_open_orders.to_owned(),
            serum_market.to_owned(),
            mango_signer.to_owned(),
            system_program.to_owned(),
            payer.to_owned(),
        ],
        seeds,
    )?;
    Ok(())
}

pub fn deposit_usdc<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    mango_cache: &AccountInfo<'info>,
    mango_root_bank: &AccountInfo<'info>,
    mango_node_bank: &AccountInfo<'info>,
    mango_vault: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    token_account: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    amount: u64,
) -> ProgramResult {
    let instruction = deposit(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &authority.key(),
        &mango_cache.key(),
        &mango_root_bank.key(),
        &mango_node_bank.key(),
        &mango_vault.key(),
        &token_account.key(),
        amount,
    )
    .expect("deposit instruction failed");
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            authority.to_owned(),
            mango_cache.to_owned(),
            mango_root_bank.to_owned(),
            mango_node_bank.to_owned(),
            mango_vault.to_owned(),
            token_program.to_owned(),
            token_account.to_owned(),
        ],
        seeds,
    )?;
    Ok(())
}

pub fn withdraw_usdc<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    mango_cache: &AccountInfo<'info>,
    mango_root_bank: &AccountInfo<'info>,
    mango_node_bank: &AccountInfo<'info>,
    mango_vault: &AccountInfo<'info>,
    mango_signer: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    token_account: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    amount: u64,
) -> ProgramResult {
    let mango_spot_open_orders = ["11111111111111111111111111111111".parse().unwrap(); 15];
    let instruction = withdraw(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &authority.key(),
        &mango_cache.key(),
        &mango_root_bank.key(),
        &mango_node_bank.key(),
        &mango_vault.key(),
        &token_account.key(),
        &mango_signer.key(),
        &mango_spot_open_orders,
        amount,
        false,
    )?;
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            authority.to_owned(),
            mango_cache.to_owned(),
            mango_root_bank.to_owned(),
            mango_node_bank.to_owned(),
            mango_vault.to_owned(),
            token_account.to_owned(),
            mango_signer.to_owned(),
            authority.to_owned(),
            token_program.to_owned(),
        ],
        seeds,
    )?;
    Ok(())
}

pub fn get_price<'info>(
    mango_cache: &AccountInfo<'info>,
    market_index: usize,
) -> Result<I80F48, ProgramError> {
    let cache: MangoCache = *MangoCache::load_from_bytes(&mango_cache.data.borrow())?;
    Ok(cache.get_price(market_index))
}

pub fn adjust_position_perp<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    mango_cache: &AccountInfo<'info>,
    mango_market: &AccountInfo<'info>,
    mango_bids: &AccountInfo<'info>,
    mango_asks: &AccountInfo<'info>,
    mango_event_queue: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    side: MangoSide,
    amount: i64,
    market_index: usize,
) -> ProgramResult {
    let mango_spot_open_orders = ["11111111111111111111111111111111".parse().unwrap(); 15];
    let price = get_price(&mango_cache, market_index)?;
    //solana_program::log::sol_log(&format!("Price: {}", price));
    let adjusted_price = match side {
        MangoSide::Ask => (price * I80F48::from(98)) / I80F48::from(100),
        MangoSide::Bid => (price * I80F48::from(102)) / I80F48::from(100),
    };
    let instruction = place_perp_order(
        &mango_program.key(),
        &mango_group.key(),
        &mango_account.key(),
        &authority.key(),
        &mango_cache.key(),
        &mango_market.key(),
        &mango_bids.key(),
        &mango_asks.key(),
        &mango_event_queue.key(),
        &mango_spot_open_orders,
        side,
        i64::MAX,
        (I80F48::from(amount) / (adjusted_price * I80F48::from(1000))).cast(), // todo: decimals
        1,
        OrderType::Market,
        side == MangoSide::Bid, // allow only short
    )?;
    invoke_signed(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_account.to_owned(),
            authority.to_owned(),
            mango_cache.to_owned(),
            mango_market.to_owned(),
            mango_bids.to_owned(),
            mango_asks.to_owned(),
            mango_event_queue.to_owned(),
        ],
        seeds,
    )?;
    let instruction = consume_events(
        &mango_program.key(),
        &mango_group.key(),
        &mango_cache.key(),
        &mango_market.key(),
        &mango_event_queue.key(),
        &mut [mango_account.key()],
        64,
    )?;
    invoke(
        &instruction,
        &[
            mango_program.to_owned(),
            mango_group.to_owned(),
            mango_cache.to_owned(),
            mango_market.to_owned(),
            mango_event_queue.to_owned(),
            mango_account.to_owned(),
        ],
    )?;
    Ok(())
}

pub fn adjust_position_spot<'info>(
    mango_program: &AccountInfo<'info>,
    mango_group: &AccountInfo<'info>,
    mango_account: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    mango_cache: &AccountInfo<'info>,
    mango_signer: &AccountInfo<'info>,
    serum_dex: &AccountInfo<'info>,
    serum_market: &AccountInfo<'info>,
    serum_bids: &AccountInfo<'info>,
    serum_asks: &AccountInfo<'info>,
    serum_request_queue: &AccountInfo<'info>,
    serum_event_queue: &AccountInfo<'info>,
    serum_base: &AccountInfo<'info>,
    serum_quote: &AccountInfo<'info>,
    serum_base_root_bank: &AccountInfo<'info>,
    serum_base_node_bank: &AccountInfo<'info>,
    serum_base_vault: &AccountInfo<'info>,
    serum_quote_root_bank: &AccountInfo<'info>,
    serum_quote_node_bank: &AccountInfo<'info>,
    serum_quote_vault: &AccountInfo<'info>,
    serum_dex_signer: &AccountInfo<'info>,
    serum_open_orders: &AccountInfo<'info>,
    srm_vault: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    side: SerumSide,
    amount: u64,
) -> ProgramResult {
    let (limit_price, max_coin_qty, max_native_pc_qty_including_fees) = match side {
        SerumSide::Bid => (NON_ZERO_MAX, NON_ZERO_MAX, NonZeroU64::new(amount).unwrap()),
        SerumSide::Ask => (
            NonZeroU64::new(1).unwrap(),
            NonZeroU64::new(amount).unwrap(),
            NON_ZERO_MAX,
        ),
    };
    let accounts = vec![
        mango_program.to_owned(),
        //
        mango_group.to_owned(),
        mango_account.to_owned(),
        authority.to_owned(),
        mango_cache.to_owned(),
        serum_dex.to_owned(),
        serum_market.to_owned(),
        serum_bids.to_owned(),
        serum_asks.to_owned(),
        serum_request_queue.to_owned(),
        serum_event_queue.to_owned(),
        serum_base.to_owned(),
        serum_quote.to_owned(),
        serum_base_root_bank.to_owned(),
        serum_base_node_bank.to_owned(),
        serum_base_vault.to_owned(),
        serum_quote_root_bank.to_owned(),
        serum_quote_node_bank.to_owned(),
        serum_quote_vault.to_owned(),
        token_program.to_owned(),
        mango_signer.to_owned(),
        serum_dex_signer.to_owned(),
        srm_vault.to_owned(),
        serum_open_orders.to_owned(),
    ];
    let meta_accounts = accounts
        .iter()
        .skip(1) // skip program id
        .map(|x| {
            x.to_account_metas(Some(x.key() == authority.key()))
                .pop()
                .unwrap()
        })
        .collect();
    let instruction = Instruction {
        program_id: mango_program.key(),
        accounts: meta_accounts,
        data: (MangoInstruction::PlaceSpotOrder2 {
            order: NewOrderInstructionV3 {
                side,
                limit_price,
                max_coin_qty,
                max_native_pc_qty_including_fees,
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
                order_type: serum_dex::matching::OrderType::ImmediateOrCancel,
                client_order_id: 1,
                limit: 65535,
            },
        })
        .pack(),
    };
    invoke_signed(&instruction, &accounts, seeds)?;
    Ok(())
}
