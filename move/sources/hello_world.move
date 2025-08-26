module tester::hello_world {
    use std::string::{Self, String};

    public struct HelloWorld has key {
        id: UID,
        owner: address,
        value: String,
    }

    public fun create(ctx: &mut TxContext) {
        let content = string::utf8(b"Hello World");

        transfer::share_object(
            HelloWorld {
                id: object::new(ctx),
                owner: tx_context::sender(ctx), // ctx.sender() 대신 tx_context::sender(ctx) 사용
                value: content,
            }
        )
    }

}
