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
}; // bumps передается первым аргументом при вызове методов и используется для верификации адресов уже на стороне программы
```

Generate mangoAccount:

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

Generate vaultTokenAccount:

```
const [vaultTokenAccount, _] = await PublicKey.findProgramAddress(
    [strategyId.publicKey.toBuffer(), utf8.encode("vault")],
    PROGRAM_ID
);
```

Gerenate strategyTokenMint:

```
const [strategyTokenMint, _] = await PublicKey.findProgramAddress(
    [strategyId.publicKey.toBuffer(), utf8.encode("mint")],
    PROGRAM_ID
);
```

### Tokens

For USDC:

```
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
...
const usdc = new Token(<connection>, USDC_MINT, TOKEN_PROGRAM_ID, user_wallet);
const depositTokenAccount = await usdc.getOrCreateAssociatedAccountInfo(
    user_wallet.publicKey
);
```

For strategy token:

```
const strategy_token = new Token(<connection>, strategyTokenMint, TOKEN_PROGRAM_ID, user_wallet);
const strategyTokenAccount = await strategy_token.getOrCreateAssociatedAccountInfo(
    user_wallet.publicKey
);
```

### Deposit

Addresses:

- owner: main wallet with strategy assets (and signer as-well)

- strategyId: strategy

- strategyAccount: strategy data account (generated)

- mangoProgram,

- mangoGroup,

- mangoAccount,

- mangoCache,

- mangoRootBank,

- mangoNodeBank,

- mangoVault,

- vaultTokenAccount: intermidiate account for USDC, needed as Mango won't accept asstets from external addresses

- depositTokenAccount: token acount with user's USDC

- strategyTokenMint: strategy token mint address

- strategyTokenAccount: token account to get strategy token

- tokenProgram: TOKEN_PROGRAM_ID, // import { TOKEN_PROGRAM_ID } from '@solana/spl-token'

Example call:

```
import { BN } from '@project-serum/anchor';
...
const amount = 10_000000; // 10 USDC
await program.rpc.deposit(bumps, new BN(amount), {
      accounts: {
        owner: userAccount,
        strategyId: strategyId,
        strategyAccount, // генерируется
        mangoProgram: MANGO_PROGRAM,
        mangoGroup: MANGO_GROUP,
        mangoAccount, // генерируется
        mangoCache: MANGO_CACHE,
        mangoRootBank: MANGO_ROOT_BANK,
        mangoNodeBank: MANGO_NODE_BANK,
        mangoVault: MANGO_VAULT,
        vaultTokenAccount, // генерируется
        depositTokenAccount,
        strategyTokenMint, // генерируется
        strategyTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      remainingAccounts: [{ isSigner: false, isWritable: false, pubkey: LIMITS_ACCOUNT }],
});
```

LIMITS_ACCOUNT - optianal account with allowlist and max_tvl

### Withdraw

Adresses:

- owner: main wallet with strategy assets (and signer as-well)

- strategyId: strategy

- strategyAccount: strategy data account (generated)

- mangoProgram,

- mangoGroup,

- mangoAccount,

- mangoCache,

- mangoRootBank,

- mangoNodeBank,

- mangoVault,

- mangoSigner,

- spotOpenOrders, // generated

- withdrawTokenAccount: token account to call USDC,

- strategyTokenMint, // generated

- strategyTokenAccount: token account for selling strategy token,

- systemProgram: SystemProgram.programId, // import { SystemProgram } from '@solana/web3.js';

- tokenProgram: TOKEN_PROGRAM_ID, // import { TOKEN_PROGRAM_ID } from '@solana/web3.js';

Example call:

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

await program.rpc.withdraw(bumps, new BN(withdrawAmount), {
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
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
    }
});
```