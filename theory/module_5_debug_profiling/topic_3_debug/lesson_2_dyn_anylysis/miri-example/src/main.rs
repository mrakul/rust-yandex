// 1. Установка, требует nighlty:
// > rustup toolchain install nightly --component miri --force

// Запуск:
// > cargo +nightly miri run

// Расширенный вариант с backtrace'ом:
// > MIRIFLAGS=-Zmiri-env-forward=RUST_BACKTRACE RUST_BACKTRACE=1 cargo +nightly miri run


// Запуск тестов:
// > cargo +nightly miri test

// Выдаст такую ошибку:
// error: Undefined Behavior: memory access failed: attempting to access 4 bytes, but got alloc238+0x14 which is at or beyond the end of the allocation of size 20 bytes

fn find_index(numbers: &[i32], target: i32) -> Option<usize> {
    // Получаем сырой указатель на начало массива
    let ptr = numbers.as_ptr();

    // (!) Выход за границы массива - этот вариант запаникует всё равно, под капотом проверка
    // for i in 0..7 {
    //     if numbers[i] == target {
    //         return Some(i);
    //     }
    // }

    // Специально выходим за границы (0..7 вместо 0..5)
    for i in 0..7 {
        // Использование unsafe обходит проверки Rust. 
        // На индексах 5 и 6 мы читаем память, которая нам не принадлежит.
        let val = unsafe { *ptr.add(i) }; 
        
        if val == target {
            return Some(i);
        }
    }
    
    None
}

fn process_batch(numbers: Vec<i32>) {
    for i in 0..100 {
        if let Some(idx) = find_index(&numbers, i) {
            println!("Found {} at index {}", i, idx);
        }
    }
}

fn main() {
    let data = vec![10, 20, 30, 40, 50];
    process_batch(data);
}