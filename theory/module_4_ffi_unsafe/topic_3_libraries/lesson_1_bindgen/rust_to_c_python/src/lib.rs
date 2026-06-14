use core::ffi::{c_char, CStr};

// cargo build --lib

// nm выведет весь список символов
// Linux: nm -D target/debug/librust_to_python.so | grep count_case_ascii
// macOS: nm -gU target/debug/librust_to_python.dylib | grep count_case_ascii 


// repr(C) говорит компилятору, что декларация структуры должна
// происходить по таким же принципам, как и в C, а именно без перестановок
// полей в памяти (компилятор rust по умолчанию может менять порядок
// полей по своему усмотрению, к примеру, чтобы оптимизировать размер структуры
#[repr(C)]
pub struct Cased {
    cstring: *const c_char,
    case: bool, // true for uppercase,
                // false for lowercase
}

#[unsafe(no_mangle)]
pub extern "C" fn count_case_ascii(c: Cased) -> u32 {
    if c.cstring.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(c.cstring) };
    let mut counter = 0u32;
    for &b in s.to_bytes() {
        let is_upper = (65..=90).contains(&b);
        let is_lower = (97..=122).contains(&b);
        if c.case && is_upper || !c.case && is_lower {
            counter += 1;
        }
    }
    counter
} 