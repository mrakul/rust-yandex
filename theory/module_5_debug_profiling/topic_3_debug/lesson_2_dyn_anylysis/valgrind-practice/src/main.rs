use std::ffi::c_void;

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

fn main() {

    unsafe {
        // Выделяем память через C malloc
        let ptr = malloc(1024);
        assert!(!ptr.is_null(), "malloc failed");
        
        // Используем память (не попадаем в dead code)
        std::ptr::write_bytes(ptr, 0xAB, 1024);
    
        // (!) Делаем утечку - намеренно не вызываем free(ptr)
        // free(ptr); // <-- так правильно
    
    }
 
    println!("Program finished");
}

/*** Вызовы ***/
// Для Rust-программ обычно используют valgrind --leak-check=full --show-leak-kinds=all target/debug/your_program.
// Опция --leak-check=full включает полную проверку утечек памяти, а --show-leak-kinds=all показывает все типы утечек. 

// Вызов: обычный valgrind уже на бинарнике:
// > valgrind --leak-check=full --show-leak-kinds=all target/debug/valgrind-practice

// Выдаёт:

// ==580192== LEAK SUMMARY:
// ==580192==    definitely lost: 1,024 bytes in 1 blocks       => 1024 утекли, всё верно
// ==580192==    indirectly lost: 0 bytes in 0 blocks
// ==580192==      possibly lost: 0 bytes in 0 blocks
// ==580192==    still reachable: 544 bytes in 1 blocks
// ==580192==         suppressed: 0 bytes in 0 blocks
// ==580192== 
// ==580192== For lists of detected and suppressed errors, rerun with: -s
// ==580192== ERROR SUMMARY: 1 errors from 1 contexts (suppressed: 0 from 0)