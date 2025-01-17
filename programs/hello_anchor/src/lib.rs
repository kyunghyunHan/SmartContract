use anchor_lang::prelude::*;

declare_id!("DhJcqp5xpc9LdpzgogLd4b9AyARP4M96j24qSACZQrjR");

#[program]
pub mod hello_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,data:u64) -> Result<()> {
        // ctx.accounts.new_account.data = data;
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init,payer= signer,space= 8+8)]
    pub new_account: Account<'info, NewAccount>,
    #[account(mut)]
    pub signer:Signer<'info>,
    pub system_program:Program<'info,System>

}

#[account]

pub struct NewAccount {
    data:u64,
}
