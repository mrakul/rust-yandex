// Подключается код cJSON: https://github.com/DaveGamble/cJSON
// Оставил только два нужных файлика там: cJSON.h и cJSON.c, остальные не нужны для линковки

include!(concat!(env!("OUT_DIR"), "/bindgen.rs"));

use std::ffi::{CStr, CString};

const TEST_JSON: &CStr = c"{\"meaning_of_life\": 42}";

fn main() {
    // Сырой указатель
    let json: *mut cJSON = unsafe { cJSON_Parse(TEST_JSON.as_ptr()) };

    // Указатель -> CString -> &str
    let json_str = unsafe { cJSON_PrintUnformatted(json) };
    let json_str = unsafe { CString::from_raw(json_str) };
    let json_str = json_str.to_str().unwrap();
    assert_eq!(json_str, r#"{"meaning_of_life":42}"#);

    // Получение значения
    let meaning_of_life = unsafe { cJSON_GetObjectItem(json, c"meaning_of_life".as_ptr()) };
    let meaning_of_life = unsafe { cJSON_GetNumberValue(meaning_of_life) };
    println!("Meaning of life: {}", meaning_of_life);
    assert_eq!(meaning_of_life, 42f64);

    unsafe { cJSON_Delete(json) };
}