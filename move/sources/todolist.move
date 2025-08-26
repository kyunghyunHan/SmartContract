module tester::todolist{
  
    use std::vector;
    use std::string::{Self, String};
    public struct TodoList has key {
        id: UID,
        owner: address,
        value: vector<Task>,
    }

    public struct Task has store,drop,copy{
        content: String,
        completed: bool
    }
    public entry fun create(ctx: &mut TxContext) {
       let todo_list = TodoList {
            id: object::new(ctx),
            owner: tx_context::sender(ctx),
            value: vector::empty() // 빈 vector로 초기화
        };
        // 생성된 TodoList를 트랜잭션 발신자에게 전송
        transfer::transfer(todo_list, tx_context::sender(ctx))
    }
public fun get_task_content(task: &Task): String {
    task.content
}
      public fun get_value(counter: &TodoList): vector<Task> {
        counter.value
    }

  // Task 추가 함수
    public entry fun add_task(
        todo_list: &mut TodoList,
        content: String,
        ctx: &mut TxContext
    ) {
        assert!(todo_list.owner == tx_context::sender(ctx), 0); // 소유자 확인
        let new_task = Task {
            content,
            completed: false
        };
        vector::push_back(&mut todo_list.value, new_task);
    }
       // Task 완료 상태 토글 함수
    public entry fun toggle_task(
        todo_list: &mut TodoList,
        task_index: u64,
        ctx: &mut TxContext
    ) {
        assert!(todo_list.owner == tx_context::sender(ctx), 0);
        let task = vector::borrow_mut(&mut todo_list.value, task_index);
        task.completed = !task.completed;
    }
}