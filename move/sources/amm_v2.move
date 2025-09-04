module tester::swap_examples {
    use sui::object::{Self, UID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use sui::coin::{Self, Coin};
    use sui::balance::{Self, Balance};
    use sui::clock::{Self, Clock};
    use std::vector;

    // Error codes
    const E_INSUFFICIENT_AMOUNT: u64 = 0;
    const E_EXPIRED: u64 = 1;
    const E_SLIPPAGE_EXCEEDED: u64 = 2;
    const E_INVALID_PATH: u64 = 3;

    // Pool struct to hold liquidity
    public struct Pool<phantom X, phantom Y> has key, store {
        id: UID,
        reserve_x: Balance<X>,
        reserve_y: Balance<Y>,
        lp_supply: u64,
    }

    // Router struct to manage swaps
    public struct Router has key {
        id: UID,
        fee_rate: u64, // Fee in basis points (e.g., 30 = 0.3%)
    }

    // Initialize the router
    public fun create_router(ctx: &mut TxContext): Router {
        Router {
            id: object::new(ctx),
            fee_rate: 30, // 0.3% fee
        }
    }

    // Create a new pool
    public fun create_pool<X, Y>(
        coin_x: Coin<X>,
        coin_y: Coin<Y>,
        ctx: &mut TxContext
    ): Pool<X, Y> {
        Pool {
            id: object::new(ctx),
            reserve_x: coin::into_balance(coin_x),
            reserve_y: coin::into_balance(coin_y),
            lp_supply: 1000, // Initial LP supply
        }
    }

    // Single hop exact input swap (like swapExactTokensForTokens)
    public fun swap_exact_input<X, Y>(
        pool: &mut Pool<X, Y>,
        coin_in: Coin<X>,
        min_amount_out: u64,
        clock: &Clock,
        deadline: u64,
        ctx: &mut TxContext
    ): Coin<Y> {
        // Check deadline
        assert!(
            clock::timestamp_ms(clock) <= deadline,
            E_EXPIRED
        );

        let amount_in = coin::value(&coin_in);
        let amount_out = calculate_output_amount<X, Y>(pool, amount_in);

        // Check slippage
        assert!(
            amount_out >= min_amount_out,
            E_SLIPPAGE_EXCEEDED
        );

        // Add input to pool
        balance::join(
            &mut pool.reserve_x,
            coin::into_balance(coin_in)
        );

        // Take output from pool
        let output_balance = balance::split(&mut pool.reserve_y, amount_out);
        coin::from_balance(output_balance, ctx)
    }

    // Single hop exact output swap (like swapTokensForExactTokens)
    public fun swap_exact_output<X, Y>(
        pool: &mut Pool<X, Y>,
        mut coin_in: Coin<X>,
        amount_out_desired: u64,
        clock: &Clock,
        deadline: u64,
        ctx: &mut TxContext
    ): (Coin<Y>, Coin<X>) {
        // Check deadline
        assert!(
            clock::timestamp_ms(clock) <= deadline,
            E_EXPIRED
        );

        let amount_in_required = calculate_input_amount<X, Y>(pool, amount_out_desired);
        let amount_in_provided = coin::value(&coin_in);

        assert!(
            amount_in_provided >= amount_in_required,
            E_INSUFFICIENT_AMOUNT
        );

        // Split the exact amount needed
        let coin_in_exact = coin::split(
            &mut coin_in,
            amount_in_required,
            ctx
        );

        // Add to pool
        balance::join(
            &mut pool.reserve_x,
            coin::into_balance(coin_in_exact)
        );

        // Take output from pool
        let output_balance = balance::split(
            &mut pool.reserve_y,
            amount_out_desired
        );
        let coin_out = coin::from_balance(output_balance, ctx);

        // Return output coin and remaining input coin (refund)
        (coin_out, coin_in)
    }

    // Multi-hop swap for exact input (A -> B -> C)
    public fun swap_multi_hop_exact_input<A, B, C>(
        pool_ab: &mut Pool<A, B>,
        pool_bc: &mut Pool<B, C>,
        coin_in: Coin<A>,
        min_amount_out: u64,
        clock: &Clock,
        deadline: u64,
        ctx: &mut TxContext
    ): Coin<C> {
        // Check deadline
        assert!(
            clock::timestamp_ms(clock) <= deadline,
            E_EXPIRED
        );

        // First swap: A -> B
        let intermediate_coin = swap_exact_input(
            pool_ab,
            coin_in,
            0, // No minimum for intermediate swap
            clock,
            deadline,
            ctx
        );

        // Second swap: B -> C
        swap_exact_input(
            pool_bc,
            intermediate_coin,
            min_amount_out,
            clock,
            deadline,
            ctx
        )
    }

    // Multi-hop swap for exact output (A -> B -> C)
    public fun swap_multi_hop_exact_output<A, B, C>(
        pool_ab: &mut Pool<A, B>,
        pool_bc: &mut Pool<B, C>,
        mut coin_in: Coin<A>,
        amount_out_desired: u64,
        clock: &Clock,
        deadline: u64,
        ctx: &mut TxContext
    ): (Coin<C>, Coin<A>) {
        // Check deadline
        assert!(
            clock::timestamp_ms(clock) <= deadline,
            E_EXPIRED
        );

        // Calculate required B amount for desired C output
        let amount_b_required = calculate_input_amount<B, C>(pool_bc, amount_out_desired);

        // Calculate required A amount for required B output
        let amount_a_required = calculate_input_amount<A, B>(pool_ab, amount_b_required);

        assert!(
            coin::value(&coin_in) >= amount_a_required,
            E_INSUFFICIENT_AMOUNT
        );

        // Split exact A amount needed
        let coin_a_exact = coin::split(&mut coin_in, amount_a_required, ctx);

        // First swap: A -> B (exact output)
        let (coin_b, refund_a_from_swap) = swap_exact_output(
            pool_ab,
            coin_a_exact,
            amount_b_required,
            clock,
            deadline,
            ctx
        );

        // Handle refund from first swap
        if (coin::value(&refund_a_from_swap) > 0) {
            coin::join(&mut coin_in, refund_a_from_swap);
        } else {
            coin::destroy_zero(refund_a_from_swap);
        };

        // Second swap: B -> C (exact output)
        let (coin_c, refund_b_from_swap) = swap_exact_output(
            pool_bc,
            coin_b,
            amount_out_desired,
            clock,
            deadline,
            ctx
        );

        // Handle refund from second swap
        if (coin::value(&refund_b_from_swap) > 0) {
            // Transfer any B refund to the user since we can't easily convert it back to A
            transfer::public_transfer(refund_b_from_swap, tx_context::sender(ctx));
        } else {
            coin::destroy_zero(refund_b_from_swap);
        };

        (coin_c, coin_in) // Return output and remaining input (refund)
    }

    // Helper function to calculate output amount using constant product formula
    fun calculate_output_amount<X, Y>(pool: &Pool<X, Y>, amount_in: u64): u64 {
        let reserve_in = balance::value(&pool.reserve_x);
        let reserve_out = balance::value(&pool.reserve_y);

        // Apply 0.3% fee
        let amount_in_with_fee = amount_in * 997; // 1000 - 3 (0.3% fee)
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = (reserve_in * 1000) + amount_in_with_fee;

        numerator / denominator
    }

    // Helper function to calculate required input amount for exact output
    fun calculate_input_amount<X, Y>(pool: &Pool<X, Y>, amount_out: u64): u64 {
        let reserve_in = balance::value(&pool.reserve_x);
        let reserve_out = balance::value(&pool.reserve_y);

        let numerator = reserve_in * amount_out * 1000;
        let denominator = (reserve_out - amount_out) * 997; // 997 = 1000 - 3 (0.3% fee)

        (numerator / denominator) + 1 // Add 1 to round up
    }

    // Get pool reserves
    public fun get_reserves<X, Y>(pool: &Pool<X, Y>): (u64, u64) {
        (
            balance::value(&pool.reserve_x),
            balance::value(&pool.reserve_y)
        )
    }

    // Add liquidity to pool
    public fun add_liquidity<X, Y>(
        pool: &mut Pool<X, Y>,
        coin_x: Coin<X>,
        coin_y: Coin<Y>,
        ctx: &mut TxContext
    ) {
        let amount_x = coin::value(&coin_x);
        let amount_y = coin::value(&coin_y);

        // Add coins to pool reserves
        balance::join(
            &mut pool.reserve_x,
            coin::into_balance(coin_x)
        );
        balance::join(
            &mut pool.reserve_y,
            coin::into_balance(coin_y)
        );

        // Update LP supply (simplified)
        pool.lp_supply = pool.lp_supply + ((amount_x + amount_y) / 2);
    }

    // Entry function example for single hop swap
    public entry fun swap_weth_to_dai(
        pool: &mut Pool<WETH, DAI>,
        weth_coin: Coin<WETH>,
        min_dai_out: u64,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let deadline = clock::timestamp_ms(clock) + 300000; // 5 minutes from now

        let dai_out = swap_exact_input(
            pool,
            weth_coin,
            min_dai_out,
            clock,
            deadline,
            ctx
        );

        // Transfer DAI to sender
        transfer::public_transfer(dai_out, tx_context::sender(ctx));
    }

    // Dummy token types for example
    public struct WETH has drop {}

    public struct DAI has drop {}

    public struct USDC has drop {}
}