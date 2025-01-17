use anchor_lang::prelude::*;

pub mod error; 
pub mod state; 
pub mod instructions;
//온체인 주소 저장
declare_id!("3gFvCTqH47nAXNVYN4bNfxYCDLhUP3BBNtQisV8cZCSg");
use instructions::*;
//프로그램의 명령어 논리를 포함하는 모듈을 지정
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
