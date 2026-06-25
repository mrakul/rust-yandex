fn factorial(n: u32) -> u32 {
    // if n == 1 { // ошибка: базовый случай должен учитывать n == 0
    if n == 0 { // ошибка: базовый случай должен учитывать n == 0
        return 1;
    }
    n * factorial(n - 1)
}

fn main() {
    let result = factorial(0);
    println!("Factorial: {}", result);
} 