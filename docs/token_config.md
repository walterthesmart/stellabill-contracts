# Token Configuration

The Subscription Vault supports accepting a standard Stellar token for billing deposits and charges (e.g. USDC). Since Stellar tokens carry varying precisions, the initializer must clearly specify the token and its decimals during setup.

## Initialization

When initializing the Contract, the administrative caller executes:
```rust
pub fn init(env: Env, token: Address, token_decimals: u32, admin: Address, min_topup: i128)
```

The config establishes the following context:
- `token`: The standard Soroban token address.
- `token_decimals`: Most instances use standard decimals (like USDC's 7), but it is stored explicit for user interface interactions or external integrations analyzing the contract state.
- `admin`: The account managing global configurations.
- `min_topup`: The minimum topup accepted for prepaid balance.

### Decimal Arithmetic and Constraints
Internally, all core functions operate natively using `i128`. Operations like:
```rust
   let topup = required.checked_sub(sub.prepaid_balance)
```
Do not invoke any floating-point arithmetic. Thus, integrators using the `SubscriptionVault` must submit `amount` parameters in terms of *atomic* units. 

For example, for 10 USDC (assuming `token_decimals` = 7), the atomic configuration expects `amount` = `100,000,000`.

### Re-Initialization

To prevent misconfiguration, re-initialization exploits, or resetting expected contract bounds, the `init` function will immediately reject re-invocation. An attempt to reset the token triggers an `Error::AlreadyInitialized` (Code `405`).
