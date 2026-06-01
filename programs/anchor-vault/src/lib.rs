pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7Pza1mifPuEXNiUZHZnesxM61caxgUtXZ9P4VjkLdRQ7");

#[program]
pub mod anchor_vault {
    use super::*;

    // initialize the vault state and vault account
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.init(&ctx.bumps)
    }

    // deposit lamports into the vault
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Implementation for depos iting lamports
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    // withdraw lamports from the vault
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // Implementation for withdrawing lamports
        ctx.accounts.withdraw(amount)?;
        Ok(())
    }

    // close the vault and transfer remaining lamports to the owner
    pub fn close(ctx: Context<Close>) -> Result<()> {
        // Implementation for closing the vault
        ctx.accounts.close()?;
        Ok(())
    }
}
