use bank_system::storage::{*};

fn main () {
    let operations_to_apply = vec![Operation::Deposit(100), Operation::Withdraw(150), Operation::CloseAccount];
    // Сделал конструктор
    let mut balance = Balance::new(100);
    
    // Первая должна выполниться, остаться -250 и закрытие аккаунта
    let bad_ops = balance.process_operations(operations_to_apply);

    assert_eq!(bad_ops.len(), 1);

    println!("{:?}", bad_ops);
} 