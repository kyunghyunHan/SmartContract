use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount,Token};
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

#[account]
pub struct Auction {
    pub nft: Pubkey,         // NFT 주소
    pub seller: Pubkey,      // 판매자
    pub nft_account: Pubkey, // 경매에 맡긴 NFT 계정
    pub end_at: i64,         // 종료 시간
    pub started: bool,
    pub ended: bool,
    pub highest_bidder: Pubkey, // 최고 입찰자
    pub highest_bid: u64,       // 최고 입찰 금액 (Lamports)
}

#[account]
pub struct BidAccount {
    pub bidder: Pubkey,
    pub amount: u64,
}

#[derive(Accounts)]
pub struct StartAuction<'info> {
    #[account(mut)]
    pub auction: Account<'info, Auction>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub seller_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub auction_nft_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
