use anchor_lang::prelude::*;
/*Game */
declare_id!("7h7bXbsYsshNZhVrvvw27JyxUfWmV3XVCtRv1sZpUta3");

#[program]
pub mod game_example {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let avatar = &mut ctx.accounts.avatar;
        avatar.level = 0;
        //카운터 소지자만 할수있게
        avatar.authority = ctx.accounts.user.key();

        msg!("Avatar initialized with count : {:?}", avatar.level);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,//새 주소를만들도록
        payer = user,//계정생성비용은 유저가 냄
        space = 8 + 32 + 8
    )]
    pub avatar: Account<'info, Avatar>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

//
#[account]
pub struct Avatar {
    pub authority: Pubkey, //authority:32 byte
    pub level: i64,
}
