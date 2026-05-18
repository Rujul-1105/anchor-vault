use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}

// impl VaultState {
//     pub const LEN: usize = core::mem::size_of::<VaultState>(); // discriminator + vault_bump + state_bump
// }
