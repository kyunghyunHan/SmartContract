use anchor_lang::prelude::*;

declare_id!("7yWB3fAzXsMzj4Z1uU516Tvvh5k5rjUiucM46Q6zHYZD");

#[program]
pub mod study {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, start: u32) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = start;

        Ok(())
    }

    pub fn loops(ctx: Context<Increment>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;

        loop {
            counter.count += 1;
            if counter.count >= 10 {
                break;
            }
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [b"counter"],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<Counter>()
    )]
    pub counter: Account<'info, Counter>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,
}

#[account]
pub struct Counter {
    pub count: u32,
}
