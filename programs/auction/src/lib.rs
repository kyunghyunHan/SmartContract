use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::Transfer;
use anchor_spl::token::{Token, TokenAccount};
declare_id!("23gHPkzs5V46TvMSpa5tJY1wWFCExxsBGFv2WypP2Ztc");

#[program]
pub mod auction {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
    pub fn start_auction(ctx: Context<StartAuction>) -> Result<()> {
        let auction = &mut ctx.accounts.auction;

        // NFT 전송
        let cpi_accounts = Transfer {
            from: ctx.accounts.seller_nft_account.to_account_info(),
            to: ctx.accounts.auction_nft_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, 1)?; // NFT 수량 1개
        auction.started = true;
        auction.end_at = Clock::get()?.unix_timestamp + 7 * 24 * 60 * 60;

        Ok(())
    }
    pub fn bid(ctx: Context<PlaceBid>, amount: u64) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        require!(auction.started, AuctionError::NotStarted);
        require!(
            Clock::get()?.unix_timestamp < auction.end_at,
            AuctionError::Ended
        );
        require!(amount > auction.highest_bid, AuctionError::BidTooLow);
        // 이전 최고 입찰자에게 환불
        if auction.highest_bidder != Pubkey::default() {
            let prev_bid = &mut ctx.accounts.prev_bid;
            **ctx
                .accounts
                .prev_bid
                .to_account_info()
                .try_borrow_mut_lamports()? += auction.highest_bid;
            auction.highest_bid = 0;
        }

        // 현재 입찰자 기록
        auction.highest_bidder = ctx.accounts.bidder.key();
        auction.highest_bid = amount;

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
#[error_code]
pub enum AuctionError {
    #[msg("Auction has already started")]
    AlreadyStarted,

    #[msg("You are not the seller")]
    NotSeller,

    #[msg("Auction has not started yet")]
    NotStarted,

    #[msg("Auction has already ended")]
    AlreadyEnded,

    #[msg("Auction has ended")]
    Ended,

    #[msg("Bid amount is too low")]
    BidTooLow,
}
#[derive(Accounts)]
pub struct Bid<'info> {
    #[account(mut)]
    pub auction: Account<'info, Auction>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(mut)]
    pub prev_bid: Account<'info, BidAccount>, // 이전 최고 입찰자 계정
    #[account(init_if_needed, payer = bidder, space = 8 + 32 + 8,
        seeds = [b"bid", bidder.key().as_ref(), auction.key().as_ref()],
        bump)]
    pub new_bid: Account<'info, BidAccount>,  // 새 입찰자 계정
    pub system_program: Program<'info, System>,
}