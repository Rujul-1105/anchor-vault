# Anchor Vault

A Solana smart contract built with Anchor that implements a secure, PDA-based vault system for managing SOL deposits and withdrawals. This project demonstrates core Anchor concepts including Program Derived Addresses (PDAs), Cross-Program Invocations (CPIs), and account management.

## Project Overview

The vault program allows users to:

-   **Initialize** a personal vault with associated state account
-   **Deposit** SOL into their vault
-   **Withdraw** SOL from their vault
-   **Close** their vault and recover all remaining SOL

The vault uses PDAs (Program Derived Addresses) to create deterministic, user-specific accounts, ensuring secure and predictable account generation without needing external keypair management.

## Project Structure

```
anchor-vault/
├── Anchor.toml              # Anchor framework configuration
├── Cargo.toml               # Workspace Rust manifest
├── package.json             # Node.js dependencies for testing/migrations
├── rust-toolchain.toml      # Rust version specification
├── tsconfig.json            # TypeScript configuration
├── programs/
│   └── anchor-vault/        # Main Solana program
│       ├── Cargo.toml       # Program-specific dependencies
│       └── src/
│           ├── lib.rs       # Program entry point & instruction handlers
│           ├── state.rs     # Account state definitions
│           ├── constants.rs # Program constants
│           ├── error.rs     # Error codes
│           ├── instructions.rs # Instruction module exports
│           └── instructions/
│               ├── initialize.rs  # Vault initialization logic
│               ├── deposit.rs     # Deposit handler
│               ├── withdraw.rs    # Withdrawal handler
│               └── close.rs       # Vault closure handler
├── tests/
│   └── test_initialize.rs   # Integration tests using LiteSVM
└── app/                     # Frontend application (placeholder)
```

## Core Files Explanation

### Configuration Files

#### `Anchor.toml`

The primary Anchor framework configuration file. Contains:

-   **package_manager**: Set to `yarn` for dependency management
-   **programs.localnet**: Specifies the program ID (`7Pza1mifPuEXNiUZHZnesxM61caxgUtXZ9P4VjkLdRQ7`) for local development
-   **provider**: Configured to use localnet cluster with Solana CLI wallet
-   **scripts**: Test command that runs `cargo test`
-   **features**: Skip linting, enable resolution

#### `Cargo.toml` (Workspace Root)

Rust workspace manifest that:

-   Defines workspace members (all programs in `programs/` directory)
-   Sets release profile optimizations: LTO, single codegen unit, overflow checks for security

#### `rust-toolchain.toml`

Specifies Rust version `1.89.0` with required components:

-   `rustfmt`: Code formatting
-   `clippy`: Linting
-   Minimal profile to reduce download size

#### `package.json`

Node.js configuration for:

-   **dev dependencies**: TypeScript, Mocha (testing), Chai (assertions)
-   **scripts**: Prettier for code formatting (`lint`, `lint:fix`)
-   **dependencies**: Anchor's core package

#### `tsconfig.json`

TypeScript configuration for any TS-based migrations or client code

### Program Source Code

#### `programs/anchor-vault/src/lib.rs`

**Entry point** of the Solana program. Declares:

-   Module imports: `constants`, `error`, `instructions`, `state`
-   Public exports from submodules
-   **Program ID**: `7Pza1mifPuEXNiUZHZnesxM61caxgUtXZ9P4VjkLdRQ7` (tied to the deployed program)
-   **Four instruction handlers**:
    -   `initialize()`: Creates vault state and vault account
    -   `deposit(amount)`: Transfers SOL from user to vault
    -   `withdraw(amount)`: Transfers SOL from vault to user (requires signing with PDA)
    -   `close()`: Closes vault and returns remaining SOL

Each instruction takes a context struct that validates and provides required accounts.

#### `programs/anchor-vault/src/state.rs`

**Account state definitions**. Contains:

```rust
pub struct VaultState {
    pub vault_bump: u8,      // PDA bump seed for vault account
    pub state_bump: u8,      // PDA bump seed for vault_state account
}
```

-   `#[derive(InitSpace)]`: Auto-calculates required account space (8 bytes discriminator + 2 bytes for bumps)
-   Bumps are stored for later use in `withdraw` and `close` operations when signing transactions with the PDA

#### `programs/anchor-vault/src/constants.rs`

Defines the PDA seed prefix:

```rust
pub const SEED: &str = "anchor";
```

Used in conjunction with dynamic data (user key, vault_state key) to generate deterministic PDAs.

#### `programs/anchor-vault/src/error.rs`

Custom error definitions using Anchor's `#[error_code]` macro:

```rust
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
}
```

Allows type-safe error handling and informative error messages on-chain.

#### `programs/anchor-vault/src/instructions.rs`

**Module aggregator** for all instruction handlers. Re-exports from submodules:

-   `close.rs`
-   `deposit.rs`
-   `initialize.rs`
-   `withdraw.rs`

This pattern separates concerns and keeps code organized.

### Instruction Handlers (In `instructions/` Directory)

#### `initialize.rs`

Initializes a new vault for a user. Defines the `Initialize` context struct with:

**Accounts**:

-   `user` (signer, mutable): Pays rent and transaction fees
-   `vault_state` (init, PDA): Stores vault metadata
    -   Seeds: `["state", user.key()]` → deterministic per-user
    -   PDA ensures only one vault_state per user
-   `vault` (PDA): Holds SOL deposits
    -   Seeds: `["vault", vault_state.key()]` → derived from state account
-   `system_program`: Required for account creation

**Handler logic**:

-   Stores the vault and state bump seeds in the `VaultState` account
-   These bumps are essential for signing CPI calls later

#### `deposit.rs`

Allows users to deposit SOL into their vault.

**Accounts**:

-   `user` (signer, mutable): Source of SOL being transferred
-   `vault_state` (mutable): Used to retrieve vault PDA bump
-   `vault` (mutable, PDA): Destination account
-   `system_program`: Required for CPI transfer

**Handler logic**:

-   Uses **CPI** (Cross-Program Invocation) to call the System Program's `transfer` instruction
-   Direct SOL transfer from user to vault PDA
-   User can deposit any amount of SOL

#### `withdraw.rs`

Allows users to withdraw SOL from their vault.

**Accounts**:

-   `user` (signer, mutable): Receives withdrawn SOL
-   `vault_state` (mutable): Provides vault PDA bump for signing
-   `vault` (mutable, PDA): Source of SOL being transferred
-   `system_program`: Required for CPI transfer

**Handler logic**:

-   Uses **CPI with signer seeds** to invoke System Program transfer while signing as the vault PDA
-   The vault PDA alone cannot authorize spending; we provide signer seeds derived from PDA generation
-   Construct signer seeds: `["vault", vault_state.key(), vault_bump]`
-   This proves to the system that we generated the vault PDA legitimately
-   User can withdraw up to the vault balance

#### `close.rs`

Closes the vault and returns all remaining SOL to the user.

**Accounts**:

-   `user` (signer, mutable): Receives remaining SOL and rent reclamation
-   `vault_state` (mutable, PDA): Marked with `close = user` → automatically closed and rent returned
-   `vault` (mutable, PDA): Source of remaining SOL
-   `system_program`: Required for CPI transfer

**Handler logic**:

-   Uses CPI with signer seeds to transfer all vault SOL back to user
-   Vault account is transferred all its lamports, then closed by Anchor (due to `close = user` constraint)
-   State account is also closed, recovering its rent
-   After close, both vault and vault_state accounts cannot be accessed

### Testing

#### `tests/test_initialize.rs`

Comprehensive integration test using **LiteSVM** (lightweight local SVM simulator).

**Test flow**:

1. **Setup**: Creates LiteSVM instance, adds program, airdrops SOL to payer
2. **Derive PDAs**: Calculates expected vault_state and vault addresses
3. **Initialize**: Sends initialize instruction, verifies bumps are stored correctly
4. **Deposit**: Transfers 1 SOL into vault, confirms vault balance
5. **Withdraw**: Withdraws 0.5 SOL from vault (continues beyond shown output)
6. **Close**: Closes vault and recovers SOL

Key techniques demonstrated:

-   Manual instruction construction with `Instruction` struct
-   `ToAccountMetas` trait for account metadata serialization
-   `AccountDeserialize` trait for reading on-chain state
-   PDA derivation and verification
-   Transaction signing and submission to LiteSVM

## Architecture Patterns

### PDA (Program Derived Address) Usage

-   **vault_state PDA**: `seeds = ["state", user.pubkey()]`
    -   One per user, deterministic and collision-free
    -   Stores vault metadata (bumps)
-   **vault PDA**: `seeds = ["vault", vault_state.pubkey()]`
    -   One per vault, derived from the state account
    -   Holds user's SOL deposits

### CPI (Cross-Program Invocation)

-   **Regular CPI** (deposit): Program calls System Program to transfer SOL (user signs)
-   **CPI with Signer Seeds** (withdraw/close): Program calls System Program while signing as a PDA
    -   Proves PDA ownership via cryptographic verification
    -   Required because PDAs cannot hold private keys

### Rent and Account Lifecycle

-   Vault accounts must maintain minimum SOL for rent exemption
-   `close` instruction recovers rent by closing accounts
-   User must pay for `vault_state` account creation during initialize

## Building and Testing

### Build the Program

```bash
cargo build-sbf
```

Compiles to Solana Bytecode Format (SBF), located in `target/deploy/anchor_vault.so`

### Run Tests

```bash
cargo test
```

Executes LiteSVM integration tests

### Deploy Locally

```bash
anchor program deploy
```

Deploys to configured cluster (localnet) with wallet from `Anchor.toml`
