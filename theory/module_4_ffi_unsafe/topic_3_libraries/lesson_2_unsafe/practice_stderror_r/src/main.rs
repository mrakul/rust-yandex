use std::{ffi::c_uchar, os::raw::{c_char, c_int}, process};
use std::ffi::CStr;

unsafe extern "C" {
    pub fn strerror_r(errnum: c_int, buf: *mut c_char, n: usize) -> *mut c_char;
}

const BUFFER_SIZE: usize = 1024;

fn main() {

    let error_num: c_int = 1;
    let mut buffer: [i8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    // Сырой указатель и длина
    let buffer_raw_mut_ptr: *mut c_char = buffer.as_mut_ptr(); // если буфер u8 => as *mut c_char;
    let buffer_len: usize = buffer.len();

    // The GNU version returns a pointer to the error message.
    
    // SAFETY: указатель валидный и длина в рамках буфера
    // Делаем вызов под unsafe
    let result_ptr = unsafe { strerror_r(error_num, buffer_raw_mut_ptr, buffer_len) };

    // Проверка на всякий случай (вроде бы, для GNU-версии может и избыточна)
    if result_ptr.is_null() {
        println!("Error {}: strerror_r зафейлилось и вернуло NULL.", error_num);
        process::exit(1);
    }

    // CStr::from_ptr небезопасно, доверяет, что buffer это строка из C c NULL-terminating символом
    let c_str = unsafe { CStr::from_ptr(result_ptr) };
    // Перевод в Rust'овскую &str с проверкой UTF-8

    let rust_str = c_str.to_str().expect("Есть не UTF-8 символы в строке");
        
    // let c_str = unsafe { CStr::from_ptr(buffer.as_ptr()) };    
    // let rust_str = c_str.to_str().expect("");
    
    println!("Error #{}: {}", error_num, rust_str);

}
