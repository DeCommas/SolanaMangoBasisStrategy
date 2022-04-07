Single program can serve several strategies, each strategy identifies by strategyId.

Other addresses are generated from strategyId.

### PDA addresses

Generate strategyAccount:

```
import { PublicKey } from '@solana/web3.js';
import { utf8 } from '@project-serum/anchor/dist/cjs/utils/bytes';
...
const [strategyAccount, strategyAccountBump] = await PublicKey.findProgramAddress(
      [strategyId.toBuffer(), utf8.encode("account")],
      PROGRAM_ID
);

const bumps = {
    strategyAccountBump
}; // bumps is used for verification
```

#### mangoAccount:

```
const [mangoAccount, _] = await PublicKey.findProgramAddress(
    [
        MANGO_GROUP.toBytes(),
        strategyAccount.toBytes(),
        new anchor.BN(1).toBuffer('le', 8),
    ],
    MANGO_PROGRAM,
);
```

#### vaultTokenAccount:

```
const [vaultTokenAccount, _] = await PublicKey.findProgramAddress(
    [strategyId.publicKey.toBuffer(), utf8.encode("vault")],
    PROGRAM_ID
);
```

#### strategyTokenMint:

```
const [strategyTokenMint, _] = await PublicKey.findProgramAddress(
    [strategyId.publicKey.toBuffer(), utf8.encode("mint")],
    PROGRAM_ID
);
```

### Tokens

```
const strategyTokenAccount = await getOrCreateAssociatedTokenAccount(connection, owner, strategyTokenMint, owner.publicKey);
const usdcTokenAccount = await getOrCreateAssociatedTokenAccount(connection, owner, usdcMint, owner.publicKey);
```

### Deposit

Accounts:

- owner: strategy token buyer

- strategyId: strategy instance id

- strategyAccount: strategy data

- mangoProgram,

- mangoGroup,

- mangoAccount,

- mangoCache,

- mangoRootBank,

- mangoNodeBank,

- mangoVault,

- vaultTokenAccount: intermediate USDC account, since mango does not allow direct deposit (pda)

- depositTokenAccount: USDC associated account

- strategyTokenMint: mint address of strategy token (pda)

- strategyTokenAccount: strategy token associated account

- tokenProgram: TOKEN_PROGRAM_ID, // import { TOKEN_PROGRAM_ID } from '@solana/spl-token'

Example:

```
import { BN } from '@project-serum/anchor';
...
const amount = 10_000000; // 10 USDC
await program.rpc.deposit(bumps, new anchor.BN(depositAmount), {
      accounts: {
        owner,
        strategyId,
        strategyAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoCache,
        mangoRootBank,
        mangoNodeBank,
        mangoVault,
        vaultTokenAccount,
        depositTokenAccount,
        strategyTokenMint,
        strategyTokenAccount,
        tokenProgram,
      },
      remainingAccounts: [{ isSigner: false, isWritable: false, pubkey: limitsAccount.publicKey }], // optional
      signers: [owner],
});
```

LIMITS_ACCOUNT - optional whitelist & max_tvl

### Withdraw

Accounts:

- owner: strategy token buyer

- strategyId: strategy instance id

- strategyAccount: strategy data

- mangoProgram,

- mangoGroup,

- mangoAccount,

- mangoCache,

- mangoRootBank,

- mangoNodeBank,

- mangoVault,

- mangoSigner,

- spotOpenOrders, // pda

- withdrawTokenAccount: USDC associated account,

- strategyTokenMint, // pda

- strategyTokenAccount: strategy token associated account,

- systemProgram: SystemProgram.programId, // import { SystemProgram } from '@solana/web3.js';

- tokenProgram: TOKEN_PROGRAM_ID, // import { TOKEN_PROGRAM_ID } from '@solana/web3.js';

Example:

```
import { BN } from '@project-serum/anchor';

const withdrawAmount = 10_000000; // 10 USDC
const spotMarketIndex = 2; // ETH/USDC

const [spotOpenOrders, _] = await PublicKey.findProgramAddress(
    [
        mangoAccount.toBuffer(),
        new anchor.BN(spotMarketIndex).toBuffer('le', 8),
        utf8.encode("OpenOrders")
    ],
    MANGO_PROGRAM
);

await program.rpc.withdraw(bumps, new anchor.BN(withdrawAmount), {
      accounts: {
        owner: owner.publicKey,
        strategyId: strategyId.publicKey,
        strategyAccount,
        mangoProgram,
        mangoGroup,
        mangoAccount,
        mangoCache,
        mangoRootBank,
        mangoNodeBank,
        mangoVault,
        mangoSigner,
        spotOpenOrders,
        withdrawTokenAccount,
        strategyTokenMint,
        strategyTokenAccount,
        systemProgram,
        tokenProgram,
      },
      signers: [owner],
});
```
