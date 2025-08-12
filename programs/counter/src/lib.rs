use anchor_lang::prelude::*;

declare_id!("ArJRUEnriCkSkGvUi1gwfi6aoBKseYjsZ7aXmraqCFq5");

#[program]
pub mod counter {
    use super::*;
   

   //init
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        
        let counter = &mut ctx.accounts.counter;
        counter.count = 0;
        //카운터 소지자만 할수있게
        counter.authority = ctx.accounts.user.key();
   
        msg!("Counter initialized with count : {:?}", counter.count);
        Ok(())
    }
    //
    pub fn increment(ctx: Context<Update>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;

        //checked_add:만약 덧셈 결과가 해당 타입의 최대값을 넘어가면 **None**을 반환하고,
        //정상 범위 내면 **Some(결과값)**을 반환합니다.
        counter.count = counter.count.checked_add(1).unwrap();
        msg!("Counter incremented to: {}", counter.count);
        Ok(())
    }

    pub fn decrement(ctx: Context<Update>) -> Result<()> {

        let counter = &mut ctx.accounts.counter;
        counter.count = counter.count.checked_sub(1).unwrap();
        msg!("Counter decremented to: {}", counter.count);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize <'info>{
    #[account(
        init,
        payer = user,//계정생성비용은 유저가 냄
        space = 8+ Counter::INIT_SPACE
    )]
    pub counter :Account<'info,Counter>,
    #[account(mut)]
    pub user:Signer<'info>,
    pub system_program:Program<'info,System>,
}
#[derive(Accounts)]//Anchor가 이 구조체를 “이 명령어에 필요한 계정 목록”으로 인식하게 하는 매크로예요
pub struct Update<'info> {
    #[account(
        mut,
        has_one = authority//authority 필드 값과, authority 계정의 공개키가 같은 지 검사
    )]
    pub counter: Account<'info, Counter>,
    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)] //공간할당 auto
pub struct Counter {
    pub authority: Pubkey, //authority:32 byte
    pub count: i64, //8byte
}

