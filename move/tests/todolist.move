
#[test_only]
module tester::todolist_tests {
   use std::string;
   use sui::test_scenario;
    use tester::todolist::{Self,TodoList,Task};

       // Test주소 만들기
    fun create_test_address(): address {
        @0xA
    }

    //ERROR CODE

const E_NOT_OWNER :u64= 0;
  #[test]
    fun test_todolist_create() {
        let owner = create_test_address();

        let mut scenario = test_scenario::begin(owner);
       
        { 
            let ctx = test_scenario::ctx(&mut scenario);
            todolist::create(ctx);
        };

        // // Verify counter was created with initial value
        test_scenario::next_tx(&mut scenario, owner);
        {
            let todolist = test_scenario::take_from_sender<TodoList>(&scenario);
            assert!(todolist::get_value(&todolist) == vector[], 0);
            test_scenario::return_to_sender(&scenario, todolist);
        };

        test_scenario::end(scenario);
    }
#[test]
fun test_todolist_insert() {
    let owner = create_test_address();
    let mut scenario = test_scenario::begin(owner);
    
    // 초기 TodoList 생성
    {
        let ctx = test_scenario::ctx(&mut scenario);
        todolist::create(ctx);
    };
    
    // Task 추가 및 검증
    test_scenario::next_tx(&mut scenario, owner);
    {
        let mut todolist_val = test_scenario::take_from_sender<TodoList>(&scenario);
        let ctx = test_scenario::ctx(&mut scenario);
        
        // 초기 상태 검증 - 빈 vector
        assert!(todolist::get_value(&todolist_val) == vector[], 0);
        
        // Task 추가
        let content = string::utf8(b"Test Task");
        todolist::add_task(&mut todolist_val, content, ctx);
        
        // Task 추가 검증
        let tasks = todolist::get_value(&todolist_val);
        assert!(vector::length(&tasks) == 1, 1);
        //참조
       let task = vector::borrow(&tasks, 0);
        assert!(todolist::get_task_content(task) == string::utf8(b"Test Task1"), E_NOT_OWNER);
        // assert!(!task.completed, 3); // completed는 초기값이 false
        
        test_scenario::return_to_sender(&scenario, todolist_val);
    };

    test_scenario::end(scenario);
}
}