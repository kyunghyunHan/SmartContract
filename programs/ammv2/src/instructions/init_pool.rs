use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self,Mint, Token, TokenAccount},
};

use crate::state::PoolState;

pub fn handler(
    ctx: Context<InitializePool>, 
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<()> {

    let pool_state = &mut ctx.accounts.pool_state;
    pool_state.fee_numerator = fee_numerator;
    pool_state.fee_denominator = fee_denominator;
    pool_state.total_amount_minted = 0; 

    Ok(())
}
#[derive(Accounts)]
#[instruction(fee_numerator: u64, fee_denominator: u64)]
pub struct InitializePool<'info> {
    // pool for token_x -> token_y 
    #[account(mut)]
    pub mint0: Account<'info, Mint>,
    #[account(mut)]
    pub mint1: Account<'info, Mint>,

    #[account(
        init, 
        payer=payer, 
        seeds=[b"pool_state", mint0.key().as_ref(), mint1.key().as_ref()], 
        bump,
        space = 8 + std::mem::size_of::<PoolState>()
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    /// CHECK: PDA authority
    #[account(seeds=[b"authority", pool_state.key().as_ref()], bump)]
    pub pool_authority: AccountInfo<'info>,

    #[account(
        init, 
        payer=payer, 
        seeds=[b"vault0", pool_state.key().as_ref()], 
        bump,
        token::mint = mint0,
        token::authority = pool_authority
    )]
    pub vault0: Account<'info, token::TokenAccount>,  // Box 제거, token:: 추가

    #[account(
        init, 
        payer=payer, 
        seeds=[b"vault1", pool_state.key().as_ref()],
        bump,
        token::mint = mint1,
        token::authority = pool_authority
    )]
    pub vault1: Account<'info, token::TokenAccount>, 

    #[account(
        init, 
        payer=payer,
        seeds=[b"pool_mint", pool_state.key().as_ref()], 
        bump, 
        mint::decimals = 9,
        mint::authority = pool_authority
    )] 
    pub pool_mint: Account<'info, token::Mint>, 

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}
