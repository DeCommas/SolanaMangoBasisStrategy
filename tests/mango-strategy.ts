import assert from 'assert';
import * as anchor from '@project-serum/anchor';
import { Program, BN } from '@project-serum/anchor';
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

  const positionSize = 15; // 0.015 ETH

  const depositAmount = 80_000000; // 80 USDC

  const limitsAccount = anchor.web3.Keypair.generate();

  it('Initialize', async () => {
    const [strategyAccount, strategyAccountBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("account")],
      program.programId
    );
    const [mangoAccount, _mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        strategyAccount.toBytes(),
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
    const [vaultTokenAccount, _vaultBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("vault")],
      program.programId
    );

    const [strategyTokenMint, _] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("mint")],
      program.programId
    );

    console.log("Strategy id:", strategyId.publicKey.toBase58());
    console.log("Strategy account:", strategyAccount.toBase58());
    const bumps = {
      strategyAccountBump,
    };

    const marketInfo = {
      perpMarketIndex: 2,
      spotMarketIndex: 2,
      spotMarketLotSize: new BN(1000),
      spotTokenIndex: 2,
    };
    await program.rpc.initialize(bumps, marketInfo, null, {
      accounts: {
        owner: owner.publicKey,
        strategyId: strategyId.publicKey,
        triggerServer: triggerServer.publicKey,
        strategyAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoSigner: "CFdbPXrnPLmo5Qrze7rw9ZNiD82R1VeNdoQosooSP1Ax",
        serumDex,
        spotMarket,
        spotOpenOrders,
        vaultTokenMint: devnetUsdc,
        vaultTokenAccount,
        strategyTokenMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY
      },
      signers: [owner, strategyId],
    });
    await program.rpc.setLimits(bumps, new BN(depositAmount + 1_000000), [owner.publicKey], {
      accounts: {
        strategyId: strategyId.publicKey,
        owner: owner.publicKey,
        strategyAccount,
        limitsAccount: limitsAccount.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [owner, limitsAccount],
    });
  });

  it('Deposit', async () => {
    const [strategyAccount, strategyAccountBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("account")],
      program.programId
    );
    const [mangoAccount, _mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        strategyAccount.toBytes(),
        new anchor.BN(accountNum).toBuffer('le', 8),
      ],
      mangoProgram,
    );
    const [vaultTokenAccount, _vaultBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("vault")],
      program.programId
    );
    const [strategyTokenMint, _] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("mint")],
      program.programId
    );

    const bumps = {
      strategyAccountBump,
    };

    const strategy_token = new Token(anchor.getProvider().connection, strategyTokenMint, TOKEN_PROGRAM_ID, owner);
    const strategyTokenAccount = await strategy_token.getOrCreateAssociatedAccountInfo(owner.publicKey);
    const strategyTokenBalanceBefore = strategyTokenAccount.amount;

    const usdcTokenAccount = await usdc_token.getOrCreateAssociatedAccountInfo(owner.publicKey);
    const usdcBalanceBefore = usdcTokenAccount.amount;
    assert(usdcBalanceBefore.toNumber() >= depositAmount, "Account balance < 100 USDC");


    await program.rpc.deposit(bumps, new anchor.BN(depositAmount), {
      accounts: {
        owner: owner.publicKey,
        strategyId: strategyId.publicKey,
        strategyAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoCache: "8mFQbdXsFXt3R3cu3oSNS3bDZRwJRP18vyzd9J278J9z",
        mangoRootBank: "HUBX4iwWEUK5VrXXXcB7uhuKrfT4fpu2T9iZbg712JrN",
        mangoNodeBank: "J2Lmnc1e4frMnBEJARPoHtfpcohLfN67HdK1inXjTFSM",
        mangoVault: "AV4CuwdvnccZMXNhu9cSCx1mkpgHWcwWEJ7Yb8Xh8QMC",
        vaultTokenAccount,
        depositTokenAccount: usdcTokenAccount.address,
        strategyTokenMint,
        strategyTokenAccount: strategyTokenAccount.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      remainingAccounts: [{ isSigner: false, isWritable: false, pubkey: limitsAccount.publicKey }],
      signers: [owner],
    });
    const balanceAfter = (await usdc_token.getOrCreateAssociatedAccountInfo(owner.publicKey)).amount;
    assert((usdcBalanceBefore.toNumber() - balanceAfter.toNumber()) == depositAmount, "Invalid balance change after deposit");

    const strategyTokenBalanceAfter = (await strategy_token.getOrCreateAssociatedAccountInfo(owner.publicKey)).amount;
    const received = strategyTokenBalanceAfter.toNumber() - strategyTokenBalanceBefore.toNumber();
    console.log("Received", received / Math.pow(10, (await strategy_token.getMintInfo()).decimals), "strategy tokens");
  });

  it('Adjust position spot', async () => {
    const [strategyAccount, strategyAccountBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("account")],
      program.programId
    );
    const [mangoAccount, _mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        strategyAccount.toBytes(),
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

    const bumps = {
      strategyAccountBump
    };

    await program.rpc.adjustPositionSpot(bumps, new anchor.BN(positionSize), { // long
      accounts: {
        strategyId: strategyId.publicKey,
        triggerServer: triggerServer.publicKey,
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
    const [strategyAccount, strategyAccountBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("account")],
      program.programId
    );
    const [mangoAccount, _mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        strategyAccount.toBytes(),
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
    const bumps = {
      strategyAccountBump,
    };

    await program.rpc.adjustPositionPerp(bumps, new anchor.BN(-positionSize), false, { // short
      accounts: {
        strategyId: strategyId.publicKey,
        triggerServer: triggerServer.publicKey,
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
        spotOpenOrders,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [triggerServer]
    });
  });

  it('Withdraw', async () => {
    const [strategyAccount, strategyAccountBump] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("account")],
      program.programId
    );
    const [mangoAccount, _mangoBump] = await PublicKey.findProgramAddress(
      [
        (mangoGroup as PublicKey).toBytes(),
        strategyAccount.toBytes(),
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
    const [strategyTokenMint, _] = await PublicKey.findProgramAddress(
      [strategyId.publicKey.toBuffer(), utf8.encode("mint")],
      program.programId
    );
    const bumps = {
      strategyAccountBump
    };

    const usdcTokenAccount = await usdc_token.getOrCreateAssociatedAccountInfo(owner.publicKey);
    const strategy_token = new Token(anchor.getProvider().connection, strategyTokenMint, TOKEN_PROGRAM_ID, owner);
    const strategyTokenAccount = await strategy_token.getOrCreateAssociatedAccountInfo(owner.publicKey);

    const strategyTokenBalanceBefore = strategyTokenAccount.amount;
    const usdcBalanceBefore = usdcTokenAccount.amount;

    const WITHDRAW_AMOUNT = 10_000000;

    await program.rpc.withdraw(bumps, new anchor.BN(WITHDRAW_AMOUNT), {
      accounts: {
        owner: owner.publicKey,
        strategyId: strategyId.publicKey,
        strategyAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoCache: "8mFQbdXsFXt3R3cu3oSNS3bDZRwJRP18vyzd9J278J9z",
        mangoRootBank: "HUBX4iwWEUK5VrXXXcB7uhuKrfT4fpu2T9iZbg712JrN",
        mangoNodeBank: "J2Lmnc1e4frMnBEJARPoHtfpcohLfN67HdK1inXjTFSM",
        mangoVault: "AV4CuwdvnccZMXNhu9cSCx1mkpgHWcwWEJ7Yb8Xh8QMC",
        mangoSigner: "CFdbPXrnPLmo5Qrze7rw9ZNiD82R1VeNdoQosooSP1Ax",
        spotOpenOrders,
        withdrawTokenAccount: usdcTokenAccount.address,
        strategyTokenMint: strategyTokenMint,
        strategyTokenAccount: strategyTokenAccount.address,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [owner],
    });

    const strategyTokenBalanceAfter = (await strategy_token.getOrCreateAssociatedAccountInfo(owner.publicKey)).amount;
    const usdcBalanceAfter = (await usdc_token.getOrCreateAssociatedAccountInfo(owner.publicKey)).amount;
    console.log("Received", (usdcBalanceAfter.toNumber() - usdcBalanceBefore.toNumber()) / Math.pow(10, 6), "USDC");
    assert(
      (strategyTokenBalanceBefore.toNumber() - strategyTokenBalanceAfter.toNumber()) == WITHDRAW_AMOUNT,
      "Invalid token balance change after withdraw"
    );
  });
});
