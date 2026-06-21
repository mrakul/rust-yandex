use libloading::{Library, Symbol};
use std::ffi::{CString, c_char};
use std::path::Path;

// Предложено в задании
type ProcessImageFn = unsafe extern "C" fn(u32, u32, *mut u8, *const c_char);

pub fn load_and_run_plugin(plugin_path: &Path,
                           width: u32,
                           height: u32,
                           // По мутабельной ссылке, чтобы передать как uint8_t *
                           rgba_data: &mut Vec<u8>,
                           // Вот это более общий вариант через view/slice для последовательных u8, здесь сработает одинаково
                           // rgba_data: &mut [u8],
                           params_str: &str) -> Result<(), Box<dyn std::error::Error>>
{
    // Загружаем библиотеку из пути
    let lib_plugin = unsafe { Library::new(plugin_path)? };

    // Получение символа из библиотеки
    let process_func_symbol: Symbol<ProcessImageFn> = 
        unsafe {
            // Ищем символ "process image" (хочет NULL-terminated строку), оба плагина должны содержать такую функцию с #[no_mangle]
            lib_plugin.get(b"process_image\0")?
        };

    // C-шная NULL-terminated строка (важно, поскольку в unsafe нужна NULL-terminated CStr, разумеется)
    let c_params_string = CString::new(params_str)?;

    // Делаем FFI-вызов в unsafe, передаём сырые указательи .as_mut_ptr() и .as_ptr() для параметров
    unsafe {
        // Вызываем, rgba_data будет изменена
        process_func_symbol(width, height, rgba_data.as_mut_ptr(), c_params_string.as_ptr());
    }

    // Сишная строка (CString) и lib_plugin дропаются здесь, для библиотеки вызывается dlclose по дропу и библиотека выгружается

    Ok(())
}
