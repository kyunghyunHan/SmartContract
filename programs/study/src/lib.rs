use anchor_lang::prelude::*;

declare_id!("7yWB3fAzXsMzj4Z1uU516Tvvh5k5rjUiucM46Q6zHYZD");

#[program]
pub mod study {
    use super::*;
    pub fn initialize_value(
        ctx: Context<InitializeValue>,
        key: Pubkey,
        value: u64,
    ) -> Result<()> {
        let value_account = &mut ctx.accounts.value_account;
        value_account.key = key;
        value_account.value = value;
        Ok(())
    }
    pub fn update_value(
        ctx: Context<UpdateValue>,
        value: u64,
    ) -> Result<()> {
        let value_account = &mut ctx.accounts.value_account;
        value_account.value = value;
        Ok(())
    }
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
#[instruction(key: Pubkey)]
pub struct InitializeValue<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<ValueAccount>(),
        seeds = [b"value", key.as_ref()],
        bump
    )]
    pub value_account: Account<'info, ValueAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct UpdateValue<'info> {
    #[account(mut)]
    pub value_account: Account<'info, ValueAccount>,
    pub authority: Signer<'info>,
}
#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,
}

#[account]
pub struct ValueAccount {
    pub key: Pubkey,
    pub value: u64,
}

#[account]
pub struct Counter {
    pub count: u32,
}
