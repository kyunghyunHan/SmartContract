use anchor_lang::prelude::*;

use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};
declare_id!("EsZBnxyMfSefm5SfHB53P5J3sA7fRCt94UgTQxCxJhK");

#[program]
pub mod ammv2 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, nonce: u8, open_time: u64) -> Result<()> {
        let pool = &mut ctx.accounts.amm_info;
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}
#[account]
pub struct Pool {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub fee: u64, // basis points
    pub bump: u8,
}
impl Pool {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 32 + 8 + 1;
}
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = AmmInfo::LEN,
        seeds = [coin_mint.key().as_ref(), pc_mint.key().as_ref()],
        bump
    )]
    pub amm_info: Account<'info, AmmInfo>,

    /// CHECK: AMM authority
    #[account(
        seeds = [coin_mint.key().as_ref(), pc_mint.key().as_ref()],
        bump
    )]
    pub amm_authority: AccountInfo<'info>,

    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        token::mint = coin_mint,
        token::authority = amm_authority,
        seeds = [b"coin_vault", coin_mint.key().as_ref()],
        bump
    )]
    pub coin_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        token::mint = pc_mint,
        token::authority = amm_authority,
        seeds = [b"pc_vault", pc_mint.key().as_ref()],
        bump
    )]
    pub pc_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 6,
        mint::authority = amm_authority,
        seeds = [b"lp_mint", coin_mint.key().as_ref(), pc_mint.key().as_ref()],
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct AmmInfo {
    /// Initialized status.
    pub status: u64,
    /// Nonce used in program address.
    pub nonce: u64,
    /// Max order count
    pub order_num: u64,
    /// Pool depth
    pub depth: u64,
    /// The mint address of coin
    pub coin_mint: Pubkey,
    /// The mint address of pc
    pub pc_mint: Pubkey,
    /// The vault address of coin
    pub coin_vault: Pubkey,
    /// The vault address of pc  
    pub pc_vault: Pubkey,
    /// LP mint
    pub lp_mint: Pubkey,
    /// Pool open time
    pub open_time: u64,
    /// Pool punish coin amount
    pub punish_coin_amount: u64,
    /// Pool punish pc amount
    pub punish_pc_amount: u64,
    /// Coin amount
    pub pool_coin_amount: u64,
    /// Pc amount
    pub pool_pc_amount: u64,
    /// Pool lp amount
    pub pool_lp_amount: u64,
    /// Switch from orderbooklimit to amm
    pub min_size: u64,
    /// The max volume ratio
    pub vol_max_cut_ratio: u64,
    /// The volume wave amount ratio
    pub amount_wave_ratio: u64,
    /// Coin lot size
    pub coin_lot_size: u64,
    /// Pc lot size  
    pub pc_lot_size: u64,
    /// Min price ratio
    pub min_price_multiplier: u64,
    /// Max price ratio
    pub max_price_multiplier: u64,
    /// system decimal value, used to normalize the value of coin and pc amount
    pub sys_decimal_value: u64,
    /// Coin vault account
    pub amm_coin_account: Pubkey,
    /// Pc vault account
    pub amm_pc_account: Pubkey,
}

impl AmmInfo {
    pub const LEN: usize = 8 + 29 * 8 + 5 * 32; // 대략 400바이트
}
