// Cargo.toml 설정 (생략)

// 필요한 라이브러리들을 import
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

// 프로그램 ID
declare_id!("EsZBnxyMfSefm5SfHB53P5J3sA7fRCt94UgTQxCxJhK");

#[program]
pub mod solana_dex {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, open_time: u64) -> Result<()> {
        let pool = &mut ctx.accounts.amm_info;

        // 기본 상태 초기화 — 모든 주요 필드 명시적으로 설정
        pool.status = 0;
        pool.order_num = 0;
        pool.depth = 0;

        pool.coin_mint = ctx.accounts.coin_mint.key();
        pool.pc_mint = ctx.accounts.pc_mint.key();
        pool.coin_vault = ctx.accounts.coin_vault.key();
        pool.pc_vault = ctx.accounts.pc_vault.key();
        pool.lp_mint = ctx.accounts.lp_mint.key();

        pool.open_time = open_time;
        pool.punish_coin_amount = 0;
        pool.punish_pc_amount = 0;

        // 풀 통계 초기화
        pool.pool_coin_amount = 0;
        pool.pool_pc_amount = 0;
        pool.pool_lp_amount = 0;

        // 거래/설정 기본값
        pool.min_size = 1;
        pool.vol_max_cut_ratio = 0;
        pool.amount_wave_ratio = 0;
        pool.coin_lot_size = 1;
        pool.pc_lot_size = 1;

        // 가격 제한 기본값
        pool.min_price_multiplier = 0;
        pool.max_price_multiplier = 0;
        pool.sys_decimal_value = 0;

        pool.amm_coin_account = ctx.accounts.coin_vault.key();
        pool.amm_pc_account = ctx.accounts.pc_vault.key();

        msg!(
            "Pool initialized with coin: {} and pc: {}",
            ctx.accounts.coin_mint.key(),
            ctx.accounts.pc_mint.key()
        );
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        max_coin_amount: u64,
        max_pc_amount: u64,
        base_side: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.amm_info;
        let clock = Clock::get()?;

        require!(
            clock.unix_timestamp >= pool.open_time as i64,
            DexError::PoolNotOpen
        );

        let coin_reserve = ctx.accounts.coin_vault.amount;
        let pc_reserve = ctx.accounts.pc_vault.amount;

        let (deposit_coin, deposit_pc, mint_lp) = if coin_reserve == 0 && pc_reserve == 0 {
            require!(max_coin_amount > 0 && max_pc_amount > 0, DexError::InvalidAmount);
            let lp_amount = sqrt(max_coin_amount as u128 * max_pc_amount as u128)?;
            (max_coin_amount, max_pc_amount, lp_amount as u64)
        } else {
            let lp_supply = ctx.accounts.lp_mint.supply;
            let (coin_amount, pc_amount) = if base_side == 0 {
                let pc_amount =
                    (max_coin_amount as u128 * pc_reserve as u128 / coin_reserve as u128) as u64;
                require!(pc_amount <= max_pc_amount, DexError::SlippageExceeded);
                (max_coin_amount, pc_amount)
            } else {
                let coin_amount =
                    (max_pc_amount as u128 * coin_reserve as u128 / pc_reserve as u128) as u64;
                require!(coin_amount <= max_coin_amount, DexError::SlippageExceeded);
                (coin_amount, max_pc_amount)
            };

            let lp_amount = std::cmp::min(
                (coin_amount as u128 * lp_supply as u128 / coin_reserve as u128) as u64,
                (pc_amount as u128 * lp_supply as u128 / pc_reserve as u128) as u64,
            );
            (coin_amount, pc_amount, lp_amount)
        };

        // 사용자 → Vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_coin_account.to_account_info(),
                    to: ctx.accounts.coin_vault.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info(),
                },
            ),
            deposit_coin,
        )?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_pc_account.to_account_info(),
                    to: ctx.accounts.pc_vault.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info(),
                },
            ),
            deposit_pc,
        )?;

        // LP mint_to (PDA signer)
        let coin_mint_key = ctx.accounts.coin_mint.key();
        let pc_mint_key = ctx.accounts.pc_mint.key();
        let seeds: &[&[u8]] = &[
            b"amm_authority",
            coin_mint_key.as_ref(),
            pc_mint_key.as_ref(),
            &[ctx.bumps.amm_authority],
        ];
        let signer = &[&seeds[..]];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.user_lp_account.to_account_info(),
                    authority: ctx.accounts.amm_authority.to_account_info(),
                },
                signer,
            ),
            mint_lp,
        )?;

        // 풀 통계 갱신 (중요)
        pool.pool_coin_amount = pool.pool_coin_amount.checked_add(deposit_coin).unwrap_or(pool.pool_coin_amount);
        pool.pool_pc_amount = pool.pool_pc_amount.checked_add(deposit_pc).unwrap_or(pool.pool_pc_amount);
        pool.pool_lp_amount = pool.pool_lp_amount.checked_add(mint_lp).unwrap_or(pool.pool_lp_amount);

        msg!(
            "Deposited: {} coin, {} pc, {} LP",
            deposit_coin,
            deposit_pc,
            mint_lp
        );
        Ok(())
    }

    pub fn swap_base_in(
        ctx: Context<SwapBaseIn>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.amm_info;
        require!(pool.status == 3, DexError::PoolNotReady);

        let source_reserve = ctx.accounts.source_vault.amount;
        let destination_reserve = ctx.accounts.destination_vault.amount;

        require!(
            source_reserve > 0 && destination_reserve > 0,
            DexError::InsufficientLiquidity
        );

        let amount_out = calculate_amount_out(
            amount_in,
            source_reserve,
            destination_reserve,
            pool.min_size,
        )?;

        require!(amount_out >= minimum_amount_out, DexError::SlippageExceeded);

        // 사용자 → source_vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_source_account.to_account_info(),
                    to: ctx.accounts.source_vault.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info(),
                },
            ),
            amount_in,
        )?;

        // PDA signer (amm_authority)
        let coin_mint_key = ctx.accounts.coin_mint.key();
        let pc_mint_key = ctx.accounts.pc_mint.key();
        let seeds: &[&[u8]] = &[
            b"amm_authority",
            coin_mint_key.as_ref(),
            pc_mint_key.as_ref(),
            &[ctx.bumps.amm_authority],
        ];
        let signer = &[&seeds[..]];

        // pool → 사용자 (destination)
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.destination_vault.to_account_info(),
                    to: ctx.accounts.user_destination_account.to_account_info(),
                    authority: ctx.accounts.amm_authority.to_account_info(),
                },
                signer,
            ),
            amount_out,
        )?;

        // (선택) 풀 통계 갱신: pool.pool_coin_amount/pc_amount 등 (생략 가능 — 구현 필요시 추가)
        msg!("Swapped {} for {}", amount_in, amount_out);
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.amm_info;

        let lp_supply = ctx.accounts.lp_mint.supply;
        require!(lp_supply > 0, DexError::NoLiquidity);

        let coin_reserve = ctx.accounts.coin_vault.amount;
        let pc_reserve = ctx.accounts.pc_vault.amount;

        let withdraw_coin = (coin_reserve as u128 * amount as u128 / lp_supply as u128) as u64;
        let withdraw_pc = (pc_reserve as u128 * amount as u128 / lp_supply as u128) as u64;

        // 사용자의 LP 소각
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    from: ctx.accounts.user_lp_account.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info(),
                },
            ),
            amount,
        )?;

        // PDA signer (amm_authority)
        let coin_mint_key = ctx.accounts.coin_mint.key();
        let pc_mint_key = ctx.accounts.pc_mint.key();
        let signer_seeds: &[&[u8]] = &[
            b"amm_authority",
            coin_mint_key.as_ref(),
            pc_mint_key.as_ref(),
            &[ctx.bumps.amm_authority],
        ];
        let signer = &[&signer_seeds[..]];

        // token 반환 (coin)
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.coin_vault.to_account_info(),
                    to: ctx.accounts.user_coin_account.to_account_info(),
                    authority: ctx.accounts.amm_authority.to_account_info(),
                },
                signer,
            ),
            withdraw_coin,
        )?;

        // token 반환 (pc)
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pc_vault.to_account_info(),
                    to: ctx.accounts.user_pc_account.to_account_info(),
                    authority: ctx.accounts.amm_authority.to_account_info(),
                },
                signer,
            ),
            withdraw_pc,
        )?;

        // 풀 통계 갱신
        pool.pool_coin_amount = pool.pool_coin_amount.saturating_sub(withdraw_coin);
        pool.pool_pc_amount = pool.pool_pc_amount.saturating_sub(withdraw_pc);
        pool.pool_lp_amount = pool.pool_lp_amount.saturating_sub(amount);

        msg!(
            "Withdrawn: {} coin, {} pc for {} LP",
            withdraw_coin,
            withdraw_pc,
            amount
        );
        Ok(())
    }
}

// =========== Helper, Accounts, Errors (same as before, 단 LEN 수정) ===========

pub fn calculate_amount_out(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    _fee_numerator: u64,
) -> Result<u64> {
    require!(amount_in > 0, DexError::InvalidAmount);
    require!(reserve_in > 0 && reserve_out > 0, DexError::InsufficientLiquidity);

    let amount_in_with_fee = (amount_in as u128) * 9975;
    let numerator = amount_in_with_fee * (reserve_out as u128);
    let denominator = (reserve_in as u128) * 10000 + amount_in_with_fee;

    Ok((numerator / denominator) as u64)
}

pub fn sqrt(y: u128) -> Result<u64> {
    if y == 0 {
        return Ok(0);
    }
    let mut z = y;
    let mut x = y / 2 + 1;
    while x < z {
        z = x;
        x = (y / x + x) / 2;
    }
    Ok(z as u64)
}

#[account]
pub struct AmmInfo {
    pub status: u64,
    // pub nonce: u64, // 제거
    pub order_num: u64,
    pub depth: u64,

    pub coin_mint: Pubkey,
    pub pc_mint: Pubkey,

    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,

    pub lp_mint: Pubkey,

    pub open_time: u64,

    pub punish_coin_amount: u64,
    pub punish_pc_amount: u64,

    pub pool_coin_amount: u64,
    pub pool_pc_amount: u64,
    pub pool_lp_amount: u64,

    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    pub coin_lot_size: u64,
    pub pc_lot_size: u64,

    pub min_price_multiplier: u64,
    pub max_price_multiplier: u64,

    pub sys_decimal_value: u64,

    pub amm_coin_account: Pubkey,
    pub amm_pc_account: Pubkey,
}

impl AmmInfo {
    // 계산: discriminator(8) + u64_count*8 + pubkey_count*32
    // u64_count = 17 (status, order_num, depth, open_time, punish_* 2, pool_* 3, min_size, vol_max_cut_ratio, amount_wave_ratio,
    // coin_lot_size, pc_lot_size, min_price_multiplier, max_price_multiplier, sys_decimal_value) => 17
    // pubkey_count = 7 (coin_mint, pc_mint, coin_vault, pc_vault, lp_mint, amm_coin_account, amm_pc_account) => 7
    pub const LEN: usize = 8 + 17 * 8 + 7 * 32; // = 8 + 136 + 224 = 368
}

// Context structs (InitializePool, Deposit, SwapBaseIn, Withdraw) - 동일하게 유지
// Errors - 동일하게 유지

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = payer,
        space = AmmInfo::LEN,
        seeds = [b"amm_info", coin_mint.key().as_ref(), pc_mint.key().as_ref()],
        bump
    )]
    pub amm_info: Account<'info, AmmInfo>,
    /// CHECK: This is the PDA authority for the AMM, verified via seeds in the instruction.
    #[account(
        seeds = [b"amm_authority", coin_mint.key().as_ref(), pc_mint.key().as_ref()],
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

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"amm_info", coin_mint.key().as_ref(), pc_mint.key().as_ref()], bump)]
    pub amm_info: Account<'info, AmmInfo>,
    /// CHECK: This is the PDA authority for the AMM, verified via seeds in the instruction.
    #[account(seeds = [b"amm_authority", coin_mint.key().as_ref(), pc_mint.key().as_ref()], bump)]
    pub amm_authority: AccountInfo<'info>,

    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,

    #[account(mut)]
    pub coin_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pc_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_coin_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_pc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_lp_account: Account<'info, TokenAccount>,

    pub user_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SwapBaseIn<'info> {
    #[account(seeds = [b"amm_info", coin_mint.key().as_ref(), pc_mint.key().as_ref()], bump)]
    pub amm_info: Account<'info, AmmInfo>,
    /// CHECK: This is the PDA authority for the AMM, verified via seeds in the instruction.
    #[account(seeds = [b"amm_authority", coin_mint.key().as_ref(), pc_mint.key().as_ref()], bump)]
    pub amm_authority: AccountInfo<'info>,

    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,

    #[account(mut)]
    pub source_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_source_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_destination_account: Account<'info, TokenAccount>,

    pub user_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub amm_info: Account<'info, AmmInfo>,

    /// CHECK: This is the PDA authority for the AMM, verified via seeds in the instruction.
    #[account(
        seeds = [b"amm_authority", coin_mint.key().as_ref(), pc_mint.key().as_ref()],
        bump
    )]
    pub amm_authority: AccountInfo<'info>,

    pub coin_mint: Account<'info, Mint>,
    pub pc_mint: Account<'info, Mint>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub coin_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pc_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_coin_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_pc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_lp_account: Account<'info, TokenAccount>,

    pub user_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum DexError {
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Pool not ready")]
    PoolNotReady,
    #[msg("Pool not open")]
    PoolNotOpen,
    #[msg("No liquidity")]
    NoLiquidity,
}
