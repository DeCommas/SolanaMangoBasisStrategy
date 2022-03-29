// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

import { BN } from "@blockworks-foundation/mango-client";
import { Program } from "@project-serum/anchor";
import { utf8 } from '@project-serum/anchor/dist/cjs/utils/bytes';
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { MangoStrategy } from '../target/types/mango_strategy';
import { SystemProgram, SYSVAR_RENT_PUBKEY, PublicKey } from '@solana/web3.js';
const anchor = require("@project-serum/anchor");

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

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

  const maxTvl = new BN(50000_000000); // 50k
  const maxDeposit = new BN(1000_000000); // 1k

  const limitsAccount = anchor.web3.Keypair.fromSecretKey(Uint8Array.from([174, 34, 158, 36, 199, 123, 196, 80, 115, 38, 105, 66, 230, 66, 105, 29, 106, 246, 39, 61, 231, 202, 111, 157, 106, 62, 57, 150, 97, 62, 248, 37, 25, 6, 203, 18, 167, 84, 217, 3, 41, 219, 237, 39, 80, 142, 25, 193, 53, 230, 94, 46, 152, 235, 156, 184, 128, 144, 231, 125, 27, 128, 181, 232]));

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
  await program.rpc.setLimits(
    bumps,
    maxTvl,
    maxDeposit,
    [
      {
        key: owner.publicKey, deposit: new BN(0)
      },
      {
        key: new PublicKey("HYrDbdxtyiHotcSsWjGyce3ACwcSJB3vZr4UYExXbsKk"), deposit: new BN(0)
      },
      {
        key: new PublicKey("7XbABKPhiMEp4LCGn6VB5juExzPoPssvqjr9f4gNG2Fg"), deposit: new BN(0)
      },
    ],
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
