import assert from 'assert';
import * as anchor from '@project-serum/anchor';
import { Program, BN } from '@project-serum/anchor';
import { MangoStrategy } from '../target/types/mango_strategy';
import { SystemProgram, SYSVAR_RENT_PUBKEY, PublicKey } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } from '@solana/spl-token';
import { utf8 } from '@project-serum/anchor/dist/cjs/utils/bytes';
import fs from 'fs';

const addresses = './addresses_mainnet.json';
const limitsKey = './limits_key.json';
const ownerKey = './key.json';

const enableLimits = false;
const maxTvl = 50000_000000; // 50k
const maxPerAddress = 1000_000000; // 1k

const initOnly = true; // do not run tests


describe('mango-strategy', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const accounts = JSON.parse(fs.readFileSync(addresses).toString());

  // To run tests owner should have at least 100 USDC (devnet)
  const owner = anchor.web3.Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync(ownerKey).toString())));

  const program = anchor.workspace.MangoStrategy as Program<MangoStrategy>;
  const mangoProgram = new PublicKey(accounts.MANGO_PROGRAM);
  const mangoGroup = new PublicKey(accounts.MANGO_GROUP);
  const accountNum = 1;

  const strategyId = anchor.web3.Keypair.generate();

  const triggerServer = owner;

  const serumDex = new PublicKey(accounts.SERUM_DEX_PROGRAM);
  const spotMarket = new PublicKey(accounts.MANGO_SPOT);

  const usdcMint = new PublicKey(accounts.USDC_MINT);

  const limitsAccount = anchor.web3.Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync(limitsKey).toString())));

  // Test position
  const positionSize = 15; // 0.015 ETH
  const depositAmount = 80_000000; // 80 USDC

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
    console.log("Limits account:", limitsAccount.publicKey.toBase58());
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
        deployer: owner.publicKey,
        strategyId: strategyId.publicKey,
        triggerServer: triggerServer.publicKey,
        strategyAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoSigner: accounts.MANGO_SIGNER,
        serumDex,
        spotMarket,
        spotOpenOrders,
        vaultTokenMint: usdcMint,
        vaultTokenAccount,
        strategyTokenMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY
      },
      signers: [owner, strategyId],
    });

    if (enableLimits) {
      const whitelist = [owner.publicKey, "HYrDbdxtyiHotcSsWjGyce3ACwcSJB3vZr4UYExXbsKk", "7XbABKPhiMEp4LCGn6VB5juExzPoPssvqjr9f4gNG2Fg"];

      await program.rpc.setLimits(
        bumps,
        new BN(maxTvl),
        new BN(maxPerAddress),
        whitelist.map(k => ({
          key: new PublicKey(k), deposit: new BN(0)
        })),
        {
          accounts: {
            strategyId: strategyId.publicKey,
            owner: owner.publicKey,
            strategyAccount,
            limitsAccount: limitsAccount.publicKey,
            systemProgram: SystemProgram.programId,
          },
          signers: [owner, limitsAccount],
        });
    }
    /*await program.rpc.dropLimits(
      bumps,
      {
        accounts: {
          strategyId: strategyId.publicKey,
          owner: owner.publicKey,
          strategyAccount,
          limitsAccount: limitsAccount.publicKey,
        },
        signers: [owner],
      });*/
  });

  if (!initOnly) {
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

      const strategyTokenAccount = await getOrCreateAssociatedTokenAccount(anchor.getProvider().connection, owner, strategyTokenMint, owner.publicKey);
      const strategyTokenBalanceBefore = strategyTokenAccount.amount;

      const usdcTokenAccount = await getOrCreateAssociatedTokenAccount(anchor.getProvider().connection, owner, usdcMint, owner.publicKey);
      const usdcBalanceBefore = usdcTokenAccount.amount;
      assert(usdcBalanceBefore >= depositAmount, "Account balance < 100 USDC");


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
        remainingAccounts: [{ isSigner: false, isWritable: true, pubkey: limitsAccount.publicKey }],
        signers: [owner],
      });
      const balanceAfter = (await getOrCreateAssociatedTokenAccount(anchor.getProvider().connection, owner, usdcMint, owner.publicKey)).amount;
      assert((usdcBalanceBefore - balanceAfter) == BigInt(depositAmount), "Invalid balance change after deposit");

      const strategyTokenBalanceAfter = (await getOrCreateAssociatedTokenAccount(anchor.getProvider().connection, owner, strategyTokenMint, owner.publicKey)).amount;
      const received = strategyTokenBalanceAfter - strategyTokenBalanceBefore;
      console.log("Received", received / BigInt(1_000000), "strategy tokens");
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

      const usdcTokenAccount = await getOrCreateAssociatedTokenAccount(anchor.getProvider().connection, owner, usdcMint, owner.publicKey);
      const strategyTokenAccount = await getOrCreateAssociatedTokenAccount(anchor.getProvider().connection, owner, strategyTokenMint, owner.publicKey);

      const strategyTokenBalanceBefore = strategyTokenAccount.amount;
      const usdcBalanceBefore = usdcTokenAccount.amount;

      const withdrawAmount = 10_000000;

      await program.rpc.withdraw(bumps, new anchor.BN(withdrawAmount), {
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
        remainingAccounts: [{ isSigner: false, isWritable: true, pubkey: limitsAccount.publicKey }],
        signers: [owner],
      });

      const strategyTokenBalanceAfter = (await getOrCreateAssociatedTokenAccount(anchor.getProvider().connection, owner, strategyTokenMint, owner.publicKey)).amount;
      const usdcBalanceAfter = (await await getOrCreateAssociatedTokenAccount(anchor.getProvider().connection, owner, usdcMint, owner.publicKey)).amount;
      console.log("Received", ((usdcBalanceAfter - usdcBalanceBefore) / BigInt(1_000000)), "USDC");
      assert(
        (strategyTokenBalanceBefore - strategyTokenBalanceAfter) == BigInt(withdrawAmount),
        "Invalid token balance change after withdraw"
      );
    });
  }
});
