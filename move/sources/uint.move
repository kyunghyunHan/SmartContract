module tester::uint {

    const MIST_PER_SUI: u64 = 1_000_000_000; //1 SUI = 10^9 MIST

    use sui::coin::Coin;
    use sui::sui::SUI;

    public fun handle_sui(payment: &mut Coin<SUI>) {
        let value_in_mist = sui::coin::value(payment);

        //sui단위로 변환
        let value_in_sui = value_in_mist / MIST_PER_SUI;
    }
}
