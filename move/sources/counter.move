module tester::counter {
    use sui::object::{Self, UID};  // UID를 위한 import
    use sui::transfer;             // transfer::share_object를 위한 import
    use sui::tx_context::{Self, TxContext};  // TxContext와 sender() 함수를 위한 import

    public struct Counter has key {
        id: UID,
        owner: address,
        value: u64
    }

    public fun create(ctx: &mut TxContext) {
        transfer::share_object(Counter {
            id: object::new(ctx),
            owner: tx_context::sender(ctx),  // ctx.sender() 대신 tx_context::sender(ctx) 사용
            value: 0
        })
    }

    public fun increment(counter: &mut Counter) {
        counter.value = counter.value + 1;
    }
    public fun decrement(counter :&mut Counter){
        counter.value = counter.value - 1;
    }
    public fun get_value(counter: &Counter): u64 {
        counter.value
    }

    public fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
        assert!(counter.owner == tx_context::sender(ctx), 0);  // ctx.sender() 대신 tx_context::sender(ctx) 사용
        counter.value = value;
    }
}