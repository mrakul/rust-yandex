// rust-gdb target/debug/debug-practice
// Надо с именем крата: break debug_practice_1::process_data, как в теме 1 (там все комментарии)
// ... 

fn sum_numbers(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..numbers.len() {
        sum += numbers[i];
    }
    sum
}

fn process_data(data: Vec<i32>) -> i32 {
    let result = sum_numbers(&data);
    result * 2  // намеренная ошибка: должно быть просто result
}

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    for i in 0..10 {
        // break src/main.rs:18 if i == 5

        // Или можно установить breakpoint на функции process_data с условием:
        // (gdb) break process_data if i == 5
        let result = process_data(data.clone());

        // (gdb) set variable result = 100
        // (gdb) p result
        // $2 = 100
        println!("Iteration {}: Sum: {}", i, result);
    }
} 