use std::ffi::{CStr, c_char};
use std::panic::catch_unwind;

// Параметры отражения (flip), дебаг и сравнение для тестов
#[derive(Debug, PartialEq)]
struct BlurParams {
    radius: u32,
    iterations: u32
}

const RGBA_BYTES_PER_PIXEL: u32 = 4;

// Парсинг параметров в формате: "radius=1,iterations=3"
fn parse_blur_params(params_str: &str) -> Result<BlurParams, String> 
{
    // Дефолтные значения, обе по 1
    let mut radius_to_use = 1;
    let mut iterations_to_use = 1;

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
            "radius" => {
                radius_to_use = match value_str.parse::<u32>() {
                    Ok(radius_value) => radius_value,
                    Err(_) => return Err(format!("Неверное значение для пары '{}': '{}'", key_str, value_str).into()),
                };
            },
            "iterations" => {
                // Меняем значение по дефолту
                iterations_to_use = match value_str.parse::<u32>() {
                    Ok(iterations_value) => iterations_value,
                    Err(_) => return Err(format!("Неверное значение для пары '{}': '{}'", key_str, value_str).into()),
                };
            },
            _ => {return Err(format!("Неподдерживаемый параметр '{}'", key_str));}
        }
    }

    Ok(BlurParams{radius: radius_to_use, iterations: iterations_to_use})
}

// Применить blur к rgba-буферу [u8]
fn apply_blur_logic(rgba_data: &mut [u8], width: u32, height: u32, params: BlurParams)
{
    todo!()
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
            let params_to_use = parse_blur_params(params_str)?;

            // (!) unsafe-получение слайса &mut [u8] из *mut u8 такого же размера
            let data_slice = std::slice::from_raw_parts_mut(rgba_data, (width * height * RGBA_BYTES_PER_PIXEL) as usize);

            // Отражение по запрошенным параметрам
            apply_blur_logic(data_slice, width, height, params_to_use);

            Ok(())
        }
    });

    // Тут чуть сложнее получилось: Result<Result<(), String> ...Box dyn >>
    match result {
        Ok(Ok(())) => {},
        Ok(Err(parse_error)) => {
            eprintln!("Ошибка плагина Blur (парсинг, вероятно): {}.\nОбработанное изображение == исходному", parse_error);
        },
        Err(_) => {
            eprintln!("Плагин mirror запаниковал!");
            // Тут можно сделать восстановление исходной, понимаю
        }
    }

}

/*** Секция тестов ***/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_params_1_3() {
        let params_str = "radius=1,iterations=3";
        let expected_params = BlurParams { radius: 1, iterations: 3 };
        // Здесь и дальше могу применить следующее допущение из задания:
        // Задание:"Обработка ошибок: все возможные ошибки обрабатываются, нет паник (unwrap() только в обоснованных местах).")
        assert_eq!(parse_blur_params(params_str).unwrap(), expected_params);
    }

    #[test]
    fn test_parse_params_radius_only() {
        let params_str = "radius=3";
        let expected_params = BlurParams { radius: 3, iterations: 1};
        assert_eq!(parse_blur_params(params_str).unwrap(), expected_params);
    }

    #[test]
    fn test_parse_params_iterations_only() {
        let params_str = "iterations=3";
        let expected_params = BlurParams { radius: 1, iterations: 3 };
        assert_eq!(parse_blur_params(params_str).unwrap(), expected_params);
    }

    #[test]
    fn test_parse_params_defaults_missing_keys() {
        let params_str = "";
        let expected_params = BlurParams { radius: 1, iterations: 1 };
        assert_eq!(parse_blur_params(params_str).unwrap(), expected_params);
    }

    #[test]
    fn test_parse_params_unknown_key() {
        let params_str = "radius=1,unknown_key=some_value,iterations=3";
        let _expected = BlurParams { radius: 1, iterations: 3 };

        // Проверили на ошибку
        assert_eq!(parse_blur_params(params_str).is_err(), true);
        // Можно unwrap() на err, будет без паники
        assert_eq!(parse_blur_params(params_str).unwrap_err(), "Неподдерживаемый параметр 'unknown_key'".to_string());

    }

    #[test]
    fn test_parse_params_invalid_value() {
        let params_str = "radius=wrong_value";
        // TODO: везде можно проверить сообщение
        assert!(parse_blur_params(params_str).is_err());
    }

    #[test]
    fn test_parse_params_invalid_format() {
        let params_str = "radius:true";
        assert!(parse_blur_params(params_str).is_err());
    }

    /*** Тесты с буфером 3x3 ***/

    const WIDTH: usize = 3;
    const HEIGHT: usize = 3;
    // Опорный для тестов 2x2
    const RGBA_3X3: [u8; WIDTH * HEIGHT * RGBA_BYTES_PER_PIXEL as usize] = 
    [0, 0 ,0 ,0, 0, 0 ,0 ,0, 0, 0 ,0 ,0,
     0, 0 ,0 ,0, 0, 0 ,0 ,0, 0, 0 ,0 ,0,
     0, 0 ,0 ,0, 0, 0 ,0 ,0, 0, 0 ,0 ,0];  

    
    // #[test]
    // fn test_apply_mirror_logic_horizontal() {
    //     // Клонируем опорный:       [Red   Green]
    //     //                          [Blue Yellow]
    //     let mut rgba_data: Vec<u8> = RGBA_2X2.to_vec();
    //     // Горизонтальное отражение
    //     let params = BlurParams { radius: true, iterations: false };

    //     // [Green Red]
    //     // [Yellow Blue]
    //     let expected_result = vec![0, 255, 0, 255,   
    //                                         255, 0, 0, 255,  
    //                                         255, 255, 0, 255, 
    //                                         0, 0, 255, 255];

    //     apply_mirror_logic(&mut rgba_data, WIDTH as u32, HEIGHT as u32, params);

    //     assert_eq!(rgba_data, expected_result);
    // }

    // #[test]
    // fn test_apply_mirror_logic_vertical() {
    //     // Клонируем опорный:       [Red   Green]
    //     //                          [Blue Yellow]
    //     let mut rgba_data: Vec<u8> = RGBA_2X2.to_vec();
    //     // Только вертикальное отражение
    //     let params = BlurParams { radius: false, iterations: true };


    //     let expected_result = vec![0, 0, 255, 255,
    //                                       255, 255, 0, 255,
    //                                       255, 0, 0, 255,
    //                                         0, 255, 0, 255];

    //     apply_mirror_logic(&mut rgba_data, WIDTH as u32, HEIGHT as u32, params);

    //     assert_eq!(rgba_data, expected_result);
    // }

    // #[test]
    // fn test_apply_mirror_logic_both() {
    //     // Клонируем опорный:       [Red   Green]
    //     //                          [Blue Yellow]
    //     let mut rgba_data: Vec<u8> = RGBA_2X2.to_vec();
    //     let params = BlurParams { radius: true, iterations: true };
    //     // [Yellow Blue]
    //     // [Green Red]
    //     let expected_result = vec![255, 255, 0, 255,
    //                                         0, 0, 255, 255,
    //                                         0, 255, 0, 255,
    //                                         255, 0, 0, 255];

    //     apply_mirror_logic(&mut rgba_data, WIDTH as u32, HEIGHT as u32, params);

    //     assert_eq!(rgba_data, expected_result);
    // }
}