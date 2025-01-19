use anchor_lang::prelude::*;

declare_id!("FqYREJjB4ys9nxi3CGPxHwoRWEvcphswodYQuMoWQay3");

#[program]
pub mod counter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
