// cargo +nightly miri run

// Miri выдаёт ошибку:
// error: Undefined Behavior: memory access failed: attempting to access 4 bytes, but got alloc238+0x14 which is at or beyond the end of the allocation of size 20 bytes

fn main() {
    let mut vec = vec![1, 2, 3, 4, 5];
    
    // (!) Здесь выдаст ошибку, нужен unsafe код для доступа out-of-bounds
    // unsafe {
    //     // Проблема: выход за границы массива
    //     let ptr = vec.as_mut_ptr();
    //     *ptr.add(vec.len()) = 42;  // Записываем за пределами вектора
    // }

    // Без ошибки
    if vec.len() >= 2 {
        vec[0] = 1;
        vec[1] = 2;
        // Примечание: как я понял, такой доступ за границами всё равно запаникует и покажет ошибку
        // Или использовать методы без проверки с паникой
    }

    
    println!("Vector: {:?}", vec);
} 