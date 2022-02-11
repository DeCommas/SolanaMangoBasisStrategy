import assert from 'assert';
import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { MangoStrategy } from '../target/types/mango_strategy';
import { SystemProgram, SYSVAR_RENT_PUBKEY, PublicKey } from '@solana/web3.js';
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { utf8 } from '@project-serum/anchor/dist/cjs/utils/bytes';
describe('mango-strategy', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  // owner should have at least 100 USDC devnet
  const owner = anchor.web3.Keypair.fromSecretKey(Uint8Array.from([240, 124, 166, 108, 98, 125, 133, 177, 97, 55, 54, 194, 171, 75, 164, 19, 6, 91, 229, 174, 216, 34, 157, 176, 43, 18, 134, 159, 101, 69, 165, 40, 14, 85, 44, 224, 187, 162, 59, 160, 109, 230, 54, 88, 91, 80, 200, 155, 198, 56, 214, 198, 103, 131, 116, 145, 18, 212, 125, 132, 150, 247, 192, 178]));

  const program = anchor.workspace.MangoStrategy as Program<MangoStrategy>;
  const mangoProgram = new PublicKey("4skJ85cdxQAFVKbcGgfun8iZPL7BadVYXG3kGEGkufqA");
  const mangoGroup = new PublicKey("Ec2enZyoC4nGpEfu2sUNAa2nUGJHWxoUWYSEJ2hNTWTA");
  const accountNum = 1;

  const strategyId = anchor.web3.Keypair.generate();

  const triggerServer = owner;

  const serumDex = new PublicKey("DESVgJVGajEgKGXhb6XmqDHGz3VjdgP7rEVESBgxmroY");
  const spotMarket = new PublicKey("BkAraCyL9TTLbeMY3L1VWrPcv32DvSi5QDDQjik1J6Ac");

  const devnetUsdc = new PublicKey("8FRFC6MoGGkMFQwngccyu69VnYbzykGeez7ignHVAFSN");
  const usdc_token = new Token(anchor.getProvider().connection, devnetUsdc, TOKEN_PROGRAM_ID, owner);

  it('Initialize', async () => {
    const [authority, authorityBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("authority_account")],
      program.programId
    );
    const [strategyAccount, strategyBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("strategy_account")],
      program.programId
    );
    const [mangoAccount, mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        authority.toBytes(),
        new anchor.BN(accountNum).toBuffer('le', 8),
      ],
      mangoProgram,
    );
    const [vaultTokenAccount, vaultBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("vault_account")],
      program.programId
    );
    const [spotOpenOrders, _spotOpenOrdersBump] = await PublicKey.findProgramAddress(
      [
        mangoAccount.toBuffer(),
        new anchor.BN(2).toBuffer('le', 8),
        utf8.encode("OpenOrders")
      ],
      mangoProgram
    );
    console.log("Strategy id:", strategyId.publicKey.toBase58());
    console.log("Authority", authority.toBase58());
    const bumps = {
      authorityBump,
      strategyBump,
      mangoBump,
      vaultBump,
    };

    await program.rpc.initialize(bumps, {
      accounts: {
        owner: owner.publicKey,
        strategyId: strategyId.publicKey,
        triggerServer: triggerServer.publicKey,
        authority,
        strategyAccount,
        vaultTokenMint: devnetUsdc,
        vaultTokenAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoSigner: "CFdbPXrnPLmo5Qrze7rw9ZNiD82R1VeNdoQosooSP1Ax",
        serumDex,
        spotMarket,
        spotOpenOrders,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY
      },
      signers: [owner],
    });
  });

  it('Deposit & withdraw', async () => {
    const [authority, authorityBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("authority_account")],
      program.programId
    );
    const [strategyAccount, strategyBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("strategy_account")],
      program.programId
    );
    const [_mangoAccount, mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        authority.toBytes(),
        new anchor.BN(accountNum).toBuffer('le', 8),
      ],
      mangoProgram,
    );
    const [vaultTokenAccount, vaultBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("vault_account")],
      program.programId
    );

    const bumps = {
      authorityBump,
      strategyBump,
      mangoBump,
      vaultBump,
    };

    const ownerTokenAccount = await usdc_token.getOrCreateAssociatedAccountInfo(owner.publicKey);
    await usdc_token.transfer(ownerTokenAccount.address, vaultTokenAccount, owner, [], 150_000000); // 150 USDC
    const balance = (await usdc_token.getAccountInfo(ownerTokenAccount.address)).amount;
    await program.rpc.withdraw(bumps, new anchor.BN(50_000000), {
      accounts: {
        owner: owner.publicKey,
        strategyId: strategyId.publicKey,
        authority,
        strategyAccount,
        vaultTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        destinationTokenAccount: ownerTokenAccount.address,
      }
    });
    const new_balance = (await usdc_token.getAccountInfo(ownerTokenAccount.address)).amount;
    assert((new_balance.toNumber() - balance.toNumber()) === 50_000000, "Incorrect balance change after withdraw");
  });

  it('Rebalance into mango account', async () => {
    const [authority, authorityBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("authority_account")],
      program.programId
    );
    const [strategyAccount, strategyBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("strategy_account")],
      program.programId
    );
    const [mangoAccount, mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        authority.toBytes(),
        new anchor.BN(accountNum).toBuffer('le', 8),
      ],
      mangoProgram,
    );
    const [vaultTokenAccount, vaultBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("vault_account")],
      program.programId
    );
    const bumps = {
      authorityBump,
      strategyBump,
      mangoBump,
      vaultBump,
    };
    await program.rpc.rebalanceMango(bumps, new anchor.BN(100_000000), {
      accounts: {
        owner: owner.publicKey,
        strategyId: strategyId.publicKey,
        authority,
        strategyAccount,
        vaultTokenMint: devnetUsdc,
        vaultTokenAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoCache: "8mFQbdXsFXt3R3cu3oSNS3bDZRwJRP18vyzd9J278J9z",
        mangoRootBank: "HUBX4iwWEUK5VrXXXcB7uhuKrfT4fpu2T9iZbg712JrN",
        mangoNodeBank: "J2Lmnc1e4frMnBEJARPoHtfpcohLfN67HdK1inXjTFSM",
        mangoVault: "AV4CuwdvnccZMXNhu9cSCx1mkpgHWcwWEJ7Yb8Xh8QMC",
        mangoSigner: "CFdbPXrnPLmo5Qrze7rw9ZNiD82R1VeNdoQosooSP1Ax",
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      }
    });
  });

  it('Adjust position spot', async () => {
    const [authority, authorityBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("authority_account")],
      program.programId
    );
    const [strategyAccount, strategyBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("strategy_account")],
      program.programId
    );
    const [mangoAccount, mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        authority.toBytes(),
        new anchor.BN(accountNum).toBuffer('le', 8),
      ],
      mangoProgram,
    );
    const [spotOpenOrders, _spotOpenOrdersBump] = await PublicKey.findProgramAddress(
      [
        mangoAccount.toBuffer(),
        new anchor.BN(2).toBuffer('le', 8),
        utf8.encode("OpenOrders")
      ],
      mangoProgram
    );

    const [_vaultTokenAccount, vaultBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("vault_account")],
      program.programId
    );

    const bumps = {
      authorityBump,
      strategyBump,
      mangoBump,
      vaultBump,
    };

    await program.rpc.adjustPositionSpot(bumps, new anchor.BN(20), new anchor.BN(1000), { // buy 0.02 ETH
      accounts: {
        strategyId: strategyId.publicKey,
        triggerServer: triggerServer.publicKey,
        authority,
        strategyAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoCache: "8mFQbdXsFXt3R3cu3oSNS3bDZRwJRP18vyzd9J278J9z",
        mangoSigner: "CFdbPXrnPLmo5Qrze7rw9ZNiD82R1VeNdoQosooSP1Ax",
        serumDex,
        spotMarket,
        spotOpenOrders,
        spotAsks: "3pfYeG2GKSh8SSZJEEwjYqgaHwYkq5vvSDET2M33nQAf",
        spotBids: "ETf3PZi9VaBsfpMU5e3SAn4SMjkaM6tyrn2Td9N2kSRx",
        spotRequestQueue: "9hzYZxqP4itrzPPSCSqPGkSbkbSE2gqri4kw5mWQ2Jj1",
        spotEventQueue: "F43gimmdvBPQoGA4eDxt2N2ooiYWHvQ8pEATrtsArKuC",
        spotBase: "AXBJBqj9m9bxLxjyDtfqt19WWna7jijDawjgRDFXXfB3",
        spotQuote: "Dh8w8pwvfQM5zYW1PzEFQNip8vwYVHYuZo53hFPRWTs6",
        spotBaseRootBank: "AxwY5sgwSq5Uh8GD6A6ZtSzGd5fqvW2hwgGLLgZ4v2eW",
        spotBaseNodeBank: "3FPjawEtvrwvwtAetaURTbkkucu9BJofxWZUNPGHJtHg",
        spotBaseVault: "BzNgzZ9o8eAW3KZZ47YutwhrPw24DQz4SqJ2EyvPpxMp",
        spotQuoteRootBank: "HUBX4iwWEUK5VrXXXcB7uhuKrfT4fpu2T9iZbg712JrN",
        spotQuoteNodeBank: "J2Lmnc1e4frMnBEJARPoHtfpcohLfN67HdK1inXjTFSM",
        spotQuoteVault: "AV4CuwdvnccZMXNhu9cSCx1mkpgHWcwWEJ7Yb8Xh8QMC",
        serumDexSigner: "Cxs1KorP4Dwqbn1R9FgZyQ4pT51woNnkg2GxyQgZ3ude",
        srmVault: PublicKey.default,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [triggerServer]
    });
  });

  it('Adjust position perp', async () => {
    const [authority, authorityBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("authority_account")],
      program.programId
    );
    const [strategyAccount, strategyBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("strategy_account")],
      program.programId
    );
    const [mangoAccount, mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        authority.toBytes(),
        new anchor.BN(accountNum).toBuffer('le', 8),
      ],
      mangoProgram,
    );
    const [_vaultTokenAccount, vaultBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("vault_account")],
      program.programId
    );
    const [serumOpenOrders, serumOpenOrdersBump] = await PublicKey.findProgramAddress(
      [
        mangoAccount.toBuffer(),
        new anchor.BN(2).toBuffer('le', 8),
        utf8.encode("OpenOrders")
      ],
      mangoProgram
    );
    const bumps = {
      authorityBump,
      strategyBump,
      mangoBump,
      vaultBump,
      serumOpenOrdersBump
    };

    await program.rpc.adjustPositionPerp(bumps, new anchor.BN(2), new anchor.BN(-20), false, { // short 0.02 ETH
      accounts: {
        strategyId: strategyId.publicKey,
        triggerServer: triggerServer.publicKey,
        authority,
        strategyAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoCache: "8mFQbdXsFXt3R3cu3oSNS3bDZRwJRP18vyzd9J278J9z",
        mangoRootBank: "HUBX4iwWEUK5VrXXXcB7uhuKrfT4fpu2T9iZbg712JrN",
        mangoNodeBank: "J2Lmnc1e4frMnBEJARPoHtfpcohLfN67HdK1inXjTFSM",
        mangoVault: "AV4CuwdvnccZMXNhu9cSCx1mkpgHWcwWEJ7Yb8Xh8QMC",
        mangoMarket: "8jKPf3KJKWvvSbbYnunwZYv62UoRPpyGb93NWLaswzcS",
        mangoAsks: "FXSvghvoaWFHRXzWUHi5tjK9YhgcPgMPpypFXBd4Aq3r",
        mangoBids: "6jGBscmZgRXk6oVLWbnQDpRftmzrDVu82TARci9VHKuW",
        mangoEventQueue: "8WLv5fKLYkyZpFG74kRmp2RALHQFcNKmH7eJn8ebHC13",
        mangoSigner: "CFdbPXrnPLmo5Qrze7rw9ZNiD82R1VeNdoQosooSP1Ax",
        mangoSpotAccount: serumOpenOrders,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [triggerServer]
    });
  });
});
