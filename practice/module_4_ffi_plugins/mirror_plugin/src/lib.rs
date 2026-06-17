use std::ffi::{CStr, c_char};
// Чтобы отловить паники, из задания:
// `panic!` не должен уходить через FFI-границу: если паника возникает внутри экспортируемой функции, 
//  поведение программы становится неопределённым (UB). 
// Оборачивать тело `process_image` в `std::panic::catch_unwind` нужно именно в самом плагине — перехват паники на стороне хоста не помогает, 
// потому что к моменту пересечения FFI-границы UB уже произошло. При перехвате паники логируйте ошибку и оставляйте буфер изображения без изменений. 
// Ошибки парсинга параметров и обработки не должны приводить к панике — их нужно обрабатывать явно.
use std::panic::catch_unwind;

// Параметры отражения (flip), дебаг и сравнение для тестов
#[derive(Debug, PartialEq)]
struct MirrorParams {
    horizontal: bool,
    vertical: bool
}


// Парсинг параметров в формате: "horizontal=true,vertical=false"
fn parse_mirror_params(params_str: &str) -> Result<MirrorParams, String> 
{
    // Дефолтные значения
    let mut horizontal_to_use = false;
    let mut vertical_to_use = false;

    for pair in params_str.split(',') {
        let trimmed_pair = pair.trim();
        
        if trimmed_pair.is_empty() {
            // При пустой строке => Ok(false, false)
            continue;
        }

        let parts: Vec<&str> = trimmed_pair.split('=').collect();
        if parts.len() != 2 {
            return Err(format!("Неверный формат [ключ]=[значение], строка '{}'", trimmed_pair).into());
        }

        // Параметр=значение, строки
        let key_str = parts[0].trim();
        let value_str = parts[1].trim();

        match key_str {
            "horizontal" => {
                // Меняем значение по дефолту, удобно можно сделать parse::<bool> для true/false
                horizontal_to_use = match value_str.parse::<bool>() {
                    Ok(horizontal_value) => horizontal_value,
                    Err(_) => return Err(format!("Неверное значение для пары '{}': '{}'", key_str, value_str).into()),
                };
            },
            "vertical" => {
                // Меняем значение по дефолту
                vertical_to_use = match value_str.parse::<bool>() {
                    Ok(vertical_value) => vertical_value,
                    Err(_) => return Err(format!("Неверное значение для пары '{}': '{}'", key_str, value_str).into()),
                };
            },
            _ => {return Err(format!("Неподдерживаемый параметр '{}'", key_str));}
        }
    }

    Ok(MirrorParams{horizontal: horizontal_to_use, vertical: vertical_to_use})
}

// Применить flip к rgba-буферу [u8]
fn apply_mirror_logic(rgba_data: &mut [u8], width: u32, height: u32, params: MirrorParams)
{
    // Горизонтальное отражение
    if params.horizontal {
        // По всем строкам
        for row in 0..height {
            // До середины по горизонтали (вертикальной линии посередине :) )
            for column in 0..(width / 2) {
                let pixel_left = (row * width + column) * 4;
                // Симметричный относительно вертикальной середины
                let pixel_right = (row * width + (width - 1 - column)) * 4;

                // Копируем RGBA пикселя побайтно
                for byte_offset in 0..4 {
                    // Хочет usize в параметрах
                    rgba_data.swap((pixel_left + byte_offset) as usize, (pixel_right + byte_offset) as usize);
                }
            }
        }
    }

    // Вертикальное отражение
    if params.vertical {
        // Тоже по строкам, но идём до середины
        for row in 0..(height / 2) {
            // По всем столбцам
            for column in 0..width {
                // Верхний
                let pixel_up = (row * width + column) * 4;
                // Симметричный относительно горизонтальной середины
                let pixel_down = ((height - 1 - row) * width + column) * 4;
                // Аналогично
                for byte_offset in 0..4 {
                    rgba_data.swap((pixel_up + byte_offset) as usize, (pixel_down + byte_offset) as usize);
                }
            }
        }
    }
}


// Без мэнглирования символов, экспорт через C ABI
#[unsafe(no_mangle)]
pub extern "C" fn process_image(width: u32,
                                height: u32,
                                rgba_data: *mut u8,      // uint8_t *rgba_data
                                params: *const c_char)   // const char *params
{
    // Ловим панику, чтобы не было UB через FFI границу
    // Нужно указать тип как в apply_mirror_logic для catch_unwind (сделал String)
    let result = catch_unwind(|| -> Result<(), String> {

        // unsafe-секция для работы с сырыми указателями на params и rgba-буфер
        unsafe {
            // CStr из const char *params params
            let params_cstr = CStr::from_ptr(params);
            
            // CStr -> &str
            let params_str = params_cstr.to_str()
                .map_err(|_| "Есть не UTF-8 символы")
                // Оставляю так, поскольку панику ловим, но понимаю, в прод бы сделал получше
                .unwrap();
            
            // Парсим параметры, ошибку пропагируем, тип - в catch_unwind-замыкании
            let params_to_use = parse_mirror_params(params_str)?;

            // (!) unsafe-получение слайса &mut [u8] из *mut u8 такого же размера
            let data_slice = std::slice::from_raw_parts_mut(rgba_data, (width * height * 4) as usize);

            // Отражение по запрошенным параметрам
            apply_mirror_logic(data_slice, width, height, params_to_use);

            Ok(())
        }
    });

    // Если паника, ловится catch_unwind, логируем
    // Буфер меняю in-place, понимаю, можно предусмотреть откат буфера при ошибке
    if result.is_err() {
        eprintln!("Плагин mirror запаниковал!");
    }


}

/*** Секция тестов ***/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_params_all_true() {
        let params_str = "horizontal=true,vertical=true";
        let expected_params = MirrorParams { horizontal: true, vertical: true };
        // Здесь и дальше:
        // Задание:"Обработка ошибок: все возможные ошибки обрабатываются, нет паник (unwrap() только в обоснованных местах)."
        // От себя: очень много попрактиковался с юнит-тестами в практическом задании #1
        assert_eq!(parse_mirror_params(params_str).unwrap(), expected_params);
    }

    // TODO: 
    //  - 2x2 три тесты, константный буфер RGBA (посмотреть, как для тестов делал константу в работе #1)
    //  - остальные по парсингу параметров
    //  - Константа 4 байта и другие
}