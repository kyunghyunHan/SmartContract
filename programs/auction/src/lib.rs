use anchor_lang::prelude::*;

declare_id!("23gHPkzs5V46TvMSpa5tJY1wWFCExxsBGFv2WypP2Ztc");

#[program]
pub mod auction {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
