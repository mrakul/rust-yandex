use std::ffi::c_void;

// Импорт функций стандартной библиотеки: malloc, free, puts
unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn puts(s: *const i8) -> i32;
}

fn main() {
    // C-строка (обязательно с '\0' в конце)
    let msg = b"Hello from real FFI (malloc/free + puts)\0";

    unsafe {
        // Выделяем память через C malloc
        let ptr = malloc(msg.len());
        assert!(!ptr.is_null(), "malloc failed");

        // Копируем байты в выделенную область
        std::ptr::copy_nonoverlapping(msg.as_ptr(), ptr as *mut u8, msg.len());

        // Вызываем C-функцию puts
        puts(ptr as *const i8);

        // Освобождаем память через C free (Valgrind должен быть доволен)
        free(ptr);
    }
}

/*** Вызов ***/
// valgrind --leak-check=full --show-leak-kinds=all target/debug/valgrind-practice-2

// Отчёт => всё ок:
// ==601377== LEAK SUMMARY:
// ==601377==    definitely lost: 0 bytes in 0 blocks
// ==601377==    indirectly lost: 0 bytes in 0 blocks
// ==601377==      possibly lost: 0 bytes in 0 blocks
// ==601377==    still reachable: 544 bytes in 1 blocks
// ==601377==         suppressed: 0 bytes in 0 blocks
// ==601377== 
// ==601377== For lists of detected and suppressed errors, rerun with: -s
// ==601377== ERROR SUMMARY: 0 errors from 0 contexts (suppressed: 0 from 0