fn main() {
    let mut data = vec![0u8; 10];
    
    unsafe {
        let ptr = data.as_mut_ptr();
        
        // Проблема 1: выход за границы и использование неинициализированной памяти
        // *ptr.add(10) = 42;  // Записываем за пределами вектора
        // Miri => error: Undefined Behavior: memory access failed: attempting to access 1 byte, but got alloc255+0xa which is at or beyond the end of the allocation of size 10 bytes
        //        --> src/main.rs:8:9
        // Valgrind =>
        // ==589633== Invalid write of size 1
        // ==589633==    at 0x11CBA5: miri_valgrind_comparison::main (main.rs:8)

        // (!) Дальше Miri остановится, надо заккоментировать верхнюю ошибку

        // 2. Симулируем чтение: пытаемся прочитать значение по указателю.
        // let value = *ptr.add(10);  // (!) Читаем неинициализированную память (в теории неверно)

        // Создаем переменную, которая резервирует место под u8 в стеке,
        // но внутри нее находится абсолютно неинициализированный мусор.
        let mut uninit_var: std::mem::MaybeUninit<u8> = std::mem::MaybeUninit::uninit();

        // Получаем сырой указатель на эту память
        let ptr = uninit_var.as_mut_ptr();

        // Мы НЕ записали туда ничего, память «грязная».
        let value: u8 = *ptr; 
        // => error: Undefined Behavior: reading memory at alloc295[0x0..0x1], but memory is uninitialized at [0x0..0x1], and this operation requires initialized memory
        //   --> src/main.rs:21:25

        // (!) Valgrind не нашёл эту ошибку, ему нормалёк прочитать из стека отсюда

        // Использование переменной, созданной из неинициализированной памяти
        println!("Value: {}", value);
    }
} 