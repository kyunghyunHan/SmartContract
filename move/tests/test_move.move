#[test_only]
module tester::counter_tests {
    use sui::test_scenario;
    use tester::counter::{Self, Counter};

    fun create_test_address(): address {
        @0xA
    }
    //counter 생성되는지 확인
    #[test]
    fun test_counter_create() {
        let owner = create_test_address();
        let mut scenario = test_scenario::begin(owner);

        // Test initialization
        {
            let ctx = test_scenario::ctx(&mut scenario);
            counter::create(ctx);
        };

        // Verify counter was created with initial value
        test_scenario::next_tx(&mut scenario, owner);
        {
            let counter_val = test_scenario::take_shared<Counter>(&scenario);
            assert!(
                counter::get_value(&counter_val) == 0,
                0
            );
            test_scenario::return_shared(counter_val); // return_to_sender -> return_shared
        };

        test_scenario::end(scenario);
    }
    //값이 증가하는지 확인
    #[test]
    fun test_counter_add() {
        let owner = create_test_address();
        let mut scenario = test_scenario::begin(owner);
        {
            let ctx = test_scenario::ctx(&mut scenario);
            counter::create(ctx);
        };
        test_scenario::next_tx(&mut scenario, owner);
        {
            let mut counter_val = test_scenario::take_shared<Counter>(&scenario);
            let ctx = test_scenario::ctx(&mut scenario);

            assert!(
                counter::get_value(&counter_val) == 0,
                0
            );

            counter::increment(&mut counter_val);
            counter::increment(&mut counter_val);
            counter::increment(&mut counter_val);

            assert!(
                counter::get_value(&counter_val) == 3,
                1
            );

            test_scenario::return_shared(counter_val);
        };

        test_scenario::end(scenario);

    }

}
