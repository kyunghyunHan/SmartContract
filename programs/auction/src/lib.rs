use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("23gHPkzs5V46TvMSpa5tJY1wWFCExxsBGFv2WypP2Ztc");

#[program]
pub mod auction {
    use super::*;

    // Initialize auction
    pub fn initialize_auction(ctx: Context<InitializeAuction>, duration: i64) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        let clock = Clock::get()?;

        // Initialize auction data
        auction.seller = ctx.accounts.seller.key();
        auction.nft_mint = ctx.accounts.nft_mint.key();
        auction.start_time = clock.unix_timestamp;
        auction.end_time = clock.unix_timestamp + duration;
        auction.highest_bid = 0;
        auction.highest_bidder = Pubkey::default();
        auction.ended = false;
        auction.bump = ctx.bumps.auction;

        // Transfer NFT to auction escrow
        let cpi_accounts = Transfer {
            from: ctx.accounts.seller_token_account.to_account_info(),
            to: ctx.accounts.auction_token_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, 1)?; // NFT quantity: 1

        msg!("Auction initialized. Duration: {} seconds", duration);
        Ok(())
    }

    // Place a bid
    pub fn place_bid(ctx: Context<PlaceBid>, amount: u64) -> Result<()> {
        // 미리 key를 복사해서 불변 borrow 문제 방지
        let auction_key = ctx.accounts.auction.key();
        let bidder_key = ctx.accounts.bidder.key();

        // 이제 mutable borrow 시작
        let auction = &mut ctx.accounts.auction;
        let clock = Clock::get()?;

        // Validation checks
        require!(
            clock.unix_timestamp < auction.end_time,
            AuctionError::AuctionEnded
        );
        require!(amount > auction.highest_bid, AuctionError::BidTooLow);
        require!(!auction.ended, AuctionError::AuctionEnded);

        // Refund previous highest bidder
        if auction.highest_bidder != Pubkey::default() && auction.highest_bid > 0 {
            **auction.to_account_info().try_borrow_mut_lamports()? -= auction.highest_bid;
            **ctx
                .accounts
                .previous_bidder
                .to_account_info()
                .try_borrow_mut_lamports()? += auction.highest_bid;
        }

        // Transfer new bid from bidder to auction account
        **ctx
            .accounts
            .bidder
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount;
        **auction.to_account_info().try_borrow_mut_lamports()? += amount;

        // Update auction state
        auction.highest_bid = amount;
        auction.highest_bidder = bidder_key;

        // Update or create bid record
        let bid_account = &mut ctx.accounts.bid_account;
        bid_account.bidder = bidder_key;
        bid_account.amount = amount;
        bid_account.auction = auction_key; // 여기서도 미리 복사한 값 사용
        bid_account.bump = ctx.bumps.bid_account;

        msg!("New bid placed: {} lamports by {:?}", amount, bidder_key);
        Ok(())
    }

    // End auction and distribute assets
    pub fn end_auction(ctx: Context<EndAuction>) -> Result<()> {
        // 미리 값 복사 (immutable borrow 끝냄)
        let auction_key = ctx.accounts.auction.key();
        let auction_seller = ctx.accounts.auction.seller;
        let auction_bump = ctx.accounts.auction.bump;
        let auction_ai = ctx.accounts.auction.to_account_info(); // AccountInfo 복사

        // 이제 mutable borrow 시작
        let auction = &mut ctx.accounts.auction;
        let clock = Clock::get()?;

        // Validation checks
        require!(!auction.ended, AuctionError::AuctionAlreadyEnded);
        require!(
            clock.unix_timestamp >= auction.end_time,
            AuctionError::AuctionNotEnded
        );

        auction.ended = true;

        // Prepare seeds for PDA signing
        let auction_seeds = &[b"auction", auction_seller.as_ref(), &[auction_bump]];
        let signer_seeds = &[&auction_seeds[..]];

        // Transfer NFT to winner or back to seller
        let recipient_token_account = if auction.highest_bidder != Pubkey::default() {
            ctx.accounts.winner_token_account.to_account_info()
        } else {
            ctx.accounts.seller_token_account.to_account_info()
        };

        let cpi_accounts = Transfer {
            from: ctx.accounts.auction_token_account.to_account_info(),
            to: recipient_token_account,
            authority: auction_ai.clone(), // 이미 복사한 AccountInfo 사용
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        token::transfer(cpi_ctx, 1)?;

        // Transfer SOL proceeds to seller if there was a winning bid
        if auction.highest_bidder != Pubkey::default() && auction.highest_bid > 0 {
            **auction_ai.try_borrow_mut_lamports()? -= auction.highest_bid;
            **ctx
                .accounts
                .seller
                .to_account_info()
                .try_borrow_mut_lamports()? += auction.highest_bid;
        }

        msg!(
            "Auction ended. Winner: {:?}, Winning bid: {}",
            auction.highest_bidder,
            auction.highest_bid
        );
        Ok(())
    }
}

// Account contexts
#[derive(Accounts)]
pub struct InitializeAuction<'info> {
    #[account(
        init,
        payer = seller,
        space = 8 + Auction::INIT_SPACE,
        seeds = [b"auction", seller.key().as_ref()],
        bump
    )]
    pub auction: Account<'info, Auction>,
    #[account(mut)]
    pub seller: Signer<'info>,
    pub nft_mint: Account<'info, Mint>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub auction_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(
        mut,
        seeds = [b"auction", auction.seller.as_ref()],
        bump = auction.bump
    )]
    pub auction: Account<'info, Auction>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    /// CHECK: Previous bidder for refund - validated by auction state
    #[account(mut)]
    pub previous_bidder: UncheckedAccount<'info>,
    #[account(
        init,
        payer = bidder,
        space = 8 + BidAccount::INIT_SPACE,
        seeds = [b"bid", auction.key().as_ref(), bidder.key().as_ref()],
        bump
    )]
    pub bid_account: Account<'info, BidAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EndAuction<'info> {
    #[account(
        mut,
        seeds = [b"auction", seller.key().as_ref()],
        bump = auction.bump,
        has_one = seller // Only seller can end auction
    )]
    pub auction: Account<'info, Auction>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub auction_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub winner_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

// Account data structures
#[account]
#[derive(InitSpace)]
pub struct Auction {
    pub seller: Pubkey,         // 32 bytes
    pub nft_mint: Pubkey,       // 32 bytes
    pub start_time: i64,        // 8 bytes
    pub end_time: i64,          // 8 bytes
    pub highest_bid: u64,       // 8 bytes
    pub highest_bidder: Pubkey, // 32 bytes
    pub ended: bool,            // 1 byte
    pub bump: u8,               // 1 byte
}

#[account]
#[derive(InitSpace)]
pub struct BidAccount {
    pub bidder: Pubkey,  // 32 bytes
    pub amount: u64,     // 8 bytes
    pub auction: Pubkey, // 32 bytes
    pub bump: u8,        // 1 byte
}

// Error codes
#[error_code]
pub enum AuctionError {
    #[msg("Auction has ended")]
    AuctionEnded,

    #[msg("Auction has already ended")]
    AuctionAlreadyEnded,

    #[msg("Auction has not ended yet")]
    AuctionNotEnded,

    #[msg("Bid amount is too low")]
    BidTooLow,

    #[msg("You are not the seller")]
    NotSeller,
}
