// mod helpers;
//
// use fixed::types::I80F48;
// use helpers::*;
// use mango::instruction::{add_oracle, set_oracle};
// use mango::oracle::StubOracle;
// use mango::{
//     entrypoint::process_instruction,
//     instruction::{
//         add_spot_market, cache_prices, cache_root_banks, deposit, init_mango_account,
//         update_root_bank, withdraw,
//     },
//     state::{MangoAccount, MangoGroup, NodeBank, QUOTE_INDEX},
// };
// use solana_program::account_info::AccountInfo;
// use solana_program_test::*;
// use solana_sdk::{
//     account::Account,
//     pubkey::Pubkey,
//     signature::{Keypair, Signer},
//     transaction::Transaction,
// };
// use std::mem::size_of;
//
// #[tokio::test]
// async fn test_borrow_succeeds() {
//     // Test that the borrow instruction succeeds and the expected side effects occurr
//     let program_id = Pubkey::new_unique();
//
//     let mut test = ProgramTest::new("mango", program_id, processor!(process_instruction));
//
//     // limit to track compute unit increase
//     test.set_bpf_compute_max_units(50_000);
//
//     let quote_index = 0;
//     let quote_decimals = 6;
//     let quote_unit = 10u64.pow(quote_decimals);
//     let user_initial_amount = 100000 * quote_unit;
//
//     let mango_group = add_mango_group_prodlike(&mut test, program_id);
//     let mango_group_pk = mango_group.mango_group_pk;
//
//     let user = Keypair::new();
//     let admin = Keypair::new();
//     test.add_account(user.pubkey(), Account::new(u32::MAX as u64, 0, &user.pubkey()));
//
//     let user_quote_account = add_token_account(
//         &mut test,
//         user.pubkey(),
//         mango_group.tokens[quote_index].pubkey,
//         user_initial_amount,
//     );
//
//     let mango_account_pk = Pubkey::new_unique();
//     test.add_account(
//         mango_account_pk,
//         Account::new(u32::MAX as u64, size_of::<MangoAccount>(), &program_id),
//     );
//
//     let btc_decimals = 6;
//     let btc_unit = 10u64.pow(btc_decimals);
//     let btc_vault_init_amount = 600000 * btc_unit;
//
//     let btc_mint = add_mint(&mut test, 6);
//     let btc_vault =
//         add_token_account(&mut test, mango_group.signer_pk, btc_mint.pubkey, btc_vault_init_amount);
//     let btc_node_bank = add_node_bank(&mut test, &program_id, btc_vault.pubkey);
//     let btc_root_bank = add_root_bank(&mut test, &program_id, btc_node_bank);
//
//     let oracle_pk = add_test_account_with_owner::<StubOracle>(&mut test, &program_id);
//
//     let dex_program_pk = Pubkey::new_unique();
//     let btc_usdt_spot_mkt_idx = 0;
//     let btc_usdt_spot_mkt = add_dex_empty(
//         &mut test,
//         btc_mint.pubkey,
//         mango_group.tokens[quote_index].pubkey,
//         dex_program_pk,
//     );
//
//     let user_btc_account = add_token_account(&mut test, user.pubkey(), btc_mint.pubkey, 0);
//
//     let (mut banks_client, payer, recent_blockhash) = test.start().await;
//
//     let init_leverage = I80F48::from_num(5);
//     let maint_leverage = init_leverage * 2;
//
//     let base_decimals = 6;
//     let base_price = 40000;
//     let base_unit = 10u64.pow(base_decimals);
//     let oracle_price =
//         I80F48::from_num(base_price) * I80F48::from_num(quote_unit) / I80F48::from_num(base_unit);
//
//     let borrow_and_withdraw_amount = 1 * base_unit;
//
//     // setup mango group and mango account, make a deposit, add market to basket
//     {
//         let mut transaction = Transaction::new_with_payer(
//             &[
//                 mango_group.init_mango_group(&admin.pubkey()),
//                 init_mango_account(&program_id, &mango_group_pk, &mango_account_pk, &user.pubkey())
//                     .unwrap(),
//                 cache_root_banks(
//                     &program_id,
//                     &mango_group_pk,
//                     &mango_group.mango_cache_pk,
//                     &[mango_group.root_banks[quote_index].pubkey],
//                 )
//                 .unwrap(),
//                 deposit(
//                     &program_id,
//                     &mango_group_pk,
//                     &mango_account_pk,
//                     &user.pubkey(),
//                     &mango_group.mango_cache_pk,
//                     &mango_group.root_banks[quote_index].pubkey,
//                     &mango_group.root_banks[quote_index].node_banks[quote_index].pubkey,
//                     &mango_group.root_banks[quote_index].node_banks[quote_index].vault,
//                     &user_quote_account.pubkey,
//                     user_initial_amount,
//                 )
//                 .unwrap(),
//                 add_oracle(&program_id, &mango_group_pk, &oracle_pk, &admin.pubkey()).unwrap(),
//                 set_oracle(&program_id, &mango_group_pk, &oracle_pk, &admin.pubkey(), oracle_price)
//                     .unwrap(),
//                 add_spot_market(
//                     &program_id,
//                     &mango_group_pk,
//                     &btc_usdt_spot_mkt.pubkey,
//                     &dex_program_pk,
//                     &btc_mint.pubkey,
//                     &btc_root_bank.node_banks[0].pubkey,
//                     &btc_vault.pubkey,
//                     &btc_root_bank.pubkey,
//                     &admin.pubkey(),
//                     btc_usdt_spot_mkt_idx,
//                     maint_leverage,
//                     init_leverage,
//                 )
//                 .unwrap(),
//             ],
//             Some(&payer.pubkey()),
//         );
//
//         transaction.sign(&[&payer, &admin, &user], recent_blockhash);
//
//         // Test transaction succeeded
//         assert!(banks_client.process_transaction(transaction).await.is_ok());
//
//         let mut node_bank = banks_client.get_account(btc_node_bank.pubkey).await.unwrap().unwrap();
//         let account_info: AccountInfo = (&btc_node_bank.pubkey, &mut node_bank).into();
//         let node_bank = NodeBank::load_mut_checked(&account_info, &program_id).unwrap();
//         assert_eq!(node_bank.borrows, 0);
//
//         let user_quote_balance =
//             get_token_balance(&mut banks_client, user_quote_account.pubkey).await;
//         assert_eq!(user_quote_balance, 0);
//     }
//
//     // make a borrow and withdraw
//     {
//         let mut mango_account = banks_client.get_account(mango_account_pk).await.unwrap().unwrap();
//         let account_info: AccountInfo = (&mango_account_pk, &mut mango_account).into();
//         let mango_account =
//             MangoAccount::load_mut_checked(&account_info, &program_id, &mango_group_pk).unwrap();
//         let mut mango_group = banks_client.get_account(mango_group_pk).await.unwrap().unwrap();
//         let account_info: AccountInfo = (&mango_group_pk, &mut mango_group).into();
//         let mango_group = MangoGroup::load_mut_checked(&account_info, &program_id).unwrap();
//         let borrow_token_index = 0;
//
//         println!("borrow amount: {}", borrow_and_withdraw_amount);
//
//         let mut transaction = Transaction::new_with_payer(
//             &[
//                 cache_prices(&program_id, &mango_group_pk, &mango_group.mango_cache, &[oracle_pk])
//                     .unwrap(),
//                 cache_root_banks(
//                     &program_id,
//                     &mango_group_pk,
//                     &mango_group.mango_cache,
//                     &[mango_group.tokens[QUOTE_INDEX].root_bank, btc_root_bank.pubkey],
//                 )
//                 .unwrap(),
//                 update_root_bank(
//                     &program_id,
//                     &mango_group_pk,
//                     &btc_root_bank.pubkey,
//                     &[btc_root_bank.node_banks[0].pubkey],
//                 )
//                 .unwrap(),
//                 withdraw(
//                     &program_id,
//                     &mango_group_pk,
//                     &mango_account_pk,
//                     &user.pubkey(),
//                     &mango_group.mango_cache,
//                     &mango_group.tokens[borrow_token_index].root_bank,
//                     &btc_root_bank.node_banks[0].pubkey,
//                     &btc_vault.pubkey,
//                     &user_btc_account.pubkey,
//                     &mango_group.signer_key,
//                     &mango_account.spot_open_orders,
//                     borrow_and_withdraw_amount,
//                     true, // allow_borrow
//                 )
//                 .unwrap(),
//             ],
//             Some(&payer.pubkey()),
//         );
//
//         transaction.sign(&[&payer, &user], recent_blockhash);
//
//         let result = banks_client.process_transaction(transaction).await;
//
//         // Test transaction succeeded
//         assert!(result.is_ok());
//
//         let mut mango_account = banks_client.get_account(mango_account_pk).await.unwrap().unwrap();
//         let account_info: AccountInfo = (&mango_account_pk, &mut mango_account).into();
//         let mango_account =
//             MangoAccount::load_mut_checked(&account_info, &program_id, &mango_group_pk).unwrap();
//
//         // Test expected borrow is in mango account
//         assert_eq!(mango_account.borrows[borrow_token_index], borrow_and_withdraw_amount);
//
//         // Test expected borrow is added to total in node bank
//         let mut node_bank = banks_client.get_account(btc_node_bank.pubkey).await.unwrap().unwrap();
//         let account_info: AccountInfo = (&btc_node_bank.pubkey, &mut node_bank).into();
//         let node_bank = NodeBank::load_mut_checked(&account_info, &program_id).unwrap();
//         assert_eq!(node_bank.borrows, borrow_and_withdraw_amount);
//
//         let post_btc_vault_balance = get_token_balance(&mut banks_client, btc_vault.pubkey).await;
//         assert_eq!(post_btc_vault_balance, btc_vault_init_amount - borrow_and_withdraw_amount)
//     }
// }
//
// #[tokio::test]
// async fn test_borrow_fails_overleveraged() {}
