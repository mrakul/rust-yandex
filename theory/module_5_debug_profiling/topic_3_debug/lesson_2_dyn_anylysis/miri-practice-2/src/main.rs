// main с ошибкой use-after-free
fn _main() {
    // data - указатель на вектор, который в конце освободится
    let data = {
        let vec = vec![1, 2, 3, 4, 5];
        vec.as_ptr()  // Возвращаем указатель на данные вектора
    };  // vec освобождается здесь
    
    unsafe {
        // Проблема: используем указатель после освобождения (use-after-free)
        println!("Value: {}", *data);
    }
}

fn main() {
    let vec = vec![1, 2, 3, 4, 5];
    let data = vec.as_ptr();
    
    unsafe {
        // Теперь vec всё ещё жив, указатель валиден => выдаёт 1 по первому элементы (соответствует C, указатель на первый элемент)
        println!("Value: {}", *data);
    }
} 


/*** Запуски ***/

// > cargo +nightly miri run

// error: Undefined Behavior: constructing invalid value of type &i32: encountered a dangling reference (use-after-free)
//   --> src/main.rs:10:31
//    |
// 10 |         println!("Value: {}", *data);
//    |                               ^^^^^ Undefined Behavior occurred here

// > cargo +nightly miri run