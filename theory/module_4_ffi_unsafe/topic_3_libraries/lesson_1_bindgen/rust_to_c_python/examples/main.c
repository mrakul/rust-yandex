#include <stdint.h>
#include <stdio.h>  

// Собрать этот код с подключением библиотеки Rust

// gcc ./examples/main.c target/debug/librust_to_other_lang.a -lpthread -ldl -lm -o ./c_use_rust_lib
// Note: We still need -lpthread -ldl -lm because the Rust standard library depends on these system libraries, and they must be linked dynamically.

// Define the struct exactly as it is in Rust:
// #[repr(C)]
// pub struct Cased {
//     cstring: *const c_char, // This is a pointer (8 bytes on 64-bit)
//     case: bool,             // This is 1 byte
// }
// Note: The compiler might add padding after 'case' to align the struct size, 
// but since it's the last field, it often doesn't matter for passing by value 
// if we match the total size. However, to be safe, let's look at the layout.
// On x86_64 Linux:
// pointer: 8 bytes
// bool: 1 byte
// padding: 7 bytes (to align to 8 bytes if needed for array/stack alignment)
// Total size: 16 bytes? Or 9? 
// Actually, Rust #[repr(C)] for this struct:
// offset of cstring: 0
// offset of case: 8
// size: 16 (aligned to 8 bytes because of the pointer)

// Структура, соответствующая Rust
typedef struct {
    const char* cstring;
    int case_flag; // Use int for bool to avoid header dependencies and match alignment/padding easily
} Cased;

// Объявление
uint32_t count_case_ascii(Cased c);

int main() {
    // Test string
    const char* test_str = "Hello World";
    
    // Prepare the struct for Uppercase counting
    Cased upper_input;
    upper_input.cstring = test_str;
    upper_input.case_flag = 1; // true

    uint32_t upper_count = count_case_ascii(upper_input);
    printf("Uppercase letters in '%s': %d\n", test_str, upper_count);

    // Prepare the struct for Lowercase counting
    Cased lower_input;
    lower_input.cstring = test_str;
    lower_input.case_flag = 0; // false

    uint32_t lower_count = count_case_ascii(lower_input);
    printf("Lowercase letters in '%s': %d\n", test_str, lower_count);

    return 0;
}