Одна программа может обслуживать несколько стратегий, каждая стратегия определяется адресом strategyId.

Из strategyId генерируются другие адреса.

### PDA адреса

Генерация strategyAccount:

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

Генерация mangoAccount:

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

Генерация vaultTokenAccount:

```
const [vaultTokenAccount, _] = await PublicKey.findProgramAddress(
    [strategyId.publicKey.toBuffer(), utf8.encode("vault")],
    PROGRAM_ID
);
```

Генерация strategyTokenMint:

```
const [strategyTokenMint, _] = await PublicKey.findProgramAddress(
    [strategyId.publicKey.toBuffer(), utf8.encode("mint")],
    PROGRAM_ID
);
```

### Токены

Для USDC:

```
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
...
const usdc = new Token(<connection>, USDC_MINT, TOKEN_PROGRAM_ID, user_wallet);
const depositTokenAccount = await usdc.getOrCreateAssociatedAccountInfo(
    user_wallet.publicKey
);
```

Для токена стратегии:

```
const strategy_token = new Token(<connection>, strategyTokenMint, TOKEN_PROGRAM_ID, user_wallet);
const strategyTokenAccount = await strategy_token.getOrCreateAssociatedAccountInfo(
    user_wallet.publicKey
);
```

### Deposit

Адреса:

- owner: основной кошелек, с которого происходит депозит (он же подписывает транзакцию)

- strategyId: стратегия

- strategyAccount: аккаунт данных стратегии (генерируемый)

- mangoProgram,

- mangoGroup,

- mangoAccount,

- mangoCache,

- mangoRootBank,

- mangoNodeBank,

- mangoVault,

- vaultTokenAccount: промежуточный аккаунт для приема USDC, тк манго на прямую не принимает от чужих адресов

- depositTokenAccount: токен-аккаунт с USDC пользователя

- strategyTokenMint: минт-адрес токена стратегии

- strategyTokenAccount: токен-аккаунт для получения токенов стратегии

- tokenProgram: TOKEN_PROGRAM_ID, // import { TOKEN_PROGRAM_ID } from '@solana/spl-token'

Пример вызова:

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

LIMITS_ACCOUNT - опциональный аккаунт где записаны whitelist и max_tvl

### Withdraw

Адреса:

- owner: основной кошелек, с которого происходит депозит (он же подписывает транзакцию)

- strategyId: стратегия

- strategyAccount: аккаунт данных стратегии (генерируемый)

- mangoProgram,

- mangoGroup,

- mangoAccount,

- mangoCache,

- mangoRootBank,

- mangoNodeBank,

- mangoVault,

- mangoSigner,

- spotOpenOrders, // генерируется

- withdrawTokenAccount: токен-аккаунт для вывода USDC,

- strategyTokenMint, // генерируется

- strategyTokenAccount: токен-аккаунт с которого продавать токены стратегии,

- systemProgram: SystemProgram.programId, // import { SystemProgram } from '@solana/web3.js';

- tokenProgram: TOKEN_PROGRAM_ID, // import { TOKEN_PROGRAM_ID } from '@solana/web3.js';

Пример вызова:

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