use anchor_lang::prelude::*;

pub mod error; 
pub mod state; 
pub mod instructions;

declare_id!("3gFvCTqH47nAXNVYN4bNfxYCDLhUP3BBNtQisV8cZCSg");
use instructions::*;

#[program]
pub mod ammv2 {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>, 
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<()> {
        init_pool::handler(ctx, fee_numerator, fee_denominator)
    }
}

// #[derive(Accounts)]
// pub struct Initialize {}
