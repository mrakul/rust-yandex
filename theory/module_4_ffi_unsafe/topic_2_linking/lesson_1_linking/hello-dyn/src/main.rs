// zlibVersion возвращает указатель на нуль-терминированную C-строку (const char *)
// Чтобы получить из неё Rust-овский &str, удобно использовать std::ffi::CStr::from_ptr(...).to_str()
use std::ffi::{CStr, c_char};

unsafe extern "C" {
    // Объявление внешней функции zlibVersion, возвращающую указатель на C-строку
    // Функция из libz: возвращает указатель на нуль-терминированную строку с версией
    fn zlibVersion() -> *const c_char;
}

fn main() {
    // Это покажет указатель (const char *)
    let zlib_version_ptr = unsafe { zlibVersion() };

    let zlib_version_str = unsafe { CStr::from_ptr(zlib_version_ptr) }
        .to_str()
        .expect("zlib должен вернуть валидный UTF-8");

    println!("zlib version: {:?}", zlib_version_str);

    // Интересно, что Debug просто может вывести, без to_str() и expect("...")
    println!("zlib version: {:?}", unsafe { CStr::from_ptr(zlib_version_ptr)});
}
