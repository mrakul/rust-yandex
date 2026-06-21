use std::ffi::{CStr, c_char};
use std::panic::catch_unwind;

// Параметры отражения (flip), дебаг и сравнение для тестов
#[derive(Debug, PartialEq)]
struct BlurParams {
    radius: u32,
    iterations: u32
}

const RGBA_BYTES_PER_PIXEL: u32 = 4;
// const R_OFFSET: usize = 0;
const G_OFFSET: usize = 1;
const B_OFFSET: usize = 2;
const A_OFFSET: usize = 3;

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
    // Меняем в rgba_data, итерации в промежуточный буфер, чтобы иметь неизменённые значения на каждой итерации
    let mut image_iter_snapshot = rgba_data.to_vec();

    // N-итераций по каждому пикселю 
    for _ in 0..params.iterations {
        for row in 0..height {
            for column in 0..width {
                // Текущий пиксель
                let cur_pixel_pos = (row * width + column) * RGBA_BYTES_PER_PIXEL;

                // Аккумуляторы побайтно и счётчик области вокруг
                let mut area_accum_r: u32 = 0;
                let mut area_accum_g: u32 = 0;
                let mut area_accum_b: u32 = 0;
                let mut area_accum_a: u32 = 0;
                let mut area_pixel_cnt: u32 = 0;

                // Границы текущего пикселя
                let y_min = row.saturating_sub(params.radius);
                let y_max = std::cmp::min(row + params.radius, height - 1);
                let x_min = column.saturating_sub(params.radius);
                let x_max = std::cmp::min(column + params.radius, width - 1);

                // Просматриваем область, аккумулируем по цветам, сам пиксель включается
                for area_y in y_min..= y_max {
                    for area_x in x_min..= x_max {
                        let area_pixel = (area_y * width + area_x) * RGBA_BYTES_PER_PIXEL;
                        area_accum_r += image_iter_snapshot[area_pixel as usize] as u32;
                        area_accum_g += image_iter_snapshot[area_pixel as usize + G_OFFSET] as u32;
                        area_accum_b += image_iter_snapshot[area_pixel as usize + B_OFFSET] as u32;
                        area_accum_a += image_iter_snapshot[area_pixel as usize + A_OFFSET] as u32;
                        area_pixel_cnt += 1;
                    }
                }

                // Средние значения с округлением
                let avg_r = ((area_accum_r as f64) / (area_pixel_cnt as f64)).round() as u8;
                let avg_g = ((area_accum_g as f64) / (area_pixel_cnt as f64)).round() as u8;
                let avg_b = ((area_accum_b as f64) / (area_pixel_cnt as f64)).round() as u8;
                let avg_a = ((area_accum_a as f64) / (area_pixel_cnt as f64)).round() as u8;

                // Меняем исходный пиксель 
                rgba_data[cur_pixel_pos as usize] = avg_r;
                rgba_data[cur_pixel_pos as usize + G_OFFSET] = avg_g;
                rgba_data[cur_pixel_pos as usize + B_OFFSET] = avg_b;
                rgba_data[cur_pixel_pos as usize + A_OFFSET] = avg_a;
            }
        }

        // memcpy в послеитерационный буфер всего буфера
        image_iter_snapshot.copy_from_slice(rgba_data);
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
    // Нужно указать тип как в parse_blur_params для catch_unwind (сделал String)
    let result = catch_unwind(|| -> Result<(), String> {
        
        // SAFETY: вызывающий гарантирует, что rgba_data - валидный указатель на область указанного размера (width * height * 4).
        // И что params - сишная строка с NULL-терминирующим символом.

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
        Ok(Ok(())) => {
            println!("Изображение успешно обработано плагином Blur");
        },
        Ok(Err(parse_error)) => {
            eprintln!("Ошибка плагина Blur: {}.\nИсходное изображение не меняется", parse_error);
        },
        Err(_) => {
            eprintln!("Плагин Blur запаниковал!");
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
    const RGBA_3X3_UNIFORM: [u8; WIDTH * HEIGHT * RGBA_BYTES_PER_PIXEL as usize] = 
        [120, 120, 120, 120,   120, 120, 120, 120,   120, 120, 120, 120,
         120, 120, 120, 120,   120, 120, 120, 120,   120, 120, 120, 120,
         120, 120, 120, 120,   120, 120, 120, 120,   120, 120, 120, 120];

    const RGBA_3X3_BRIGHT_CENTER: [u8; WIDTH * HEIGHT * RGBA_BYTES_PER_PIXEL as usize] = 
        [0, 0, 0, 255,   0, 0, 0, 255,           0, 0, 0, 255,
         0, 0, 0, 255,   255, 255, 255, 255,     0, 0, 0, 255,
         0, 0, 0, 255,   0, 0, 0, 255,           0, 0, 0, 255];
    
    #[test]
    fn test_apply_blur_logic_uniform_no_change() {
        let mut rgba_data: Vec<u8> = RGBA_3X3_UNIFORM.to_vec();
        let blur_params = BlurParams{radius: 2, iterations: 3};

        apply_blur_logic(&mut rgba_data, WIDTH as u32, HEIGHT as u32, blur_params);
        
        assert_eq!(rgba_data, RGBA_3X3_UNIFORM);
    }

    #[test]
    fn test_apply_blur_logic_center_affects_area() {
        let mut rgba_data: Vec<u8> = RGBA_3X3_BRIGHT_CENTER.to_vec();
        let blur_params = BlurParams{radius: 2, iterations: 3};

        apply_blur_logic(&mut rgba_data, WIDTH as u32, HEIGHT as u32, blur_params);
        
        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                let cur_pixel_pos = (row * WIDTH + column) * RGBA_BYTES_PER_PIXEL as usize;
                // Этот свалится, для проверки
                // assert_eq!(rgba_data[cur_pixel_pos], 0);
                
                // Цвета поменялись
                assert_ne!(rgba_data[cur_pixel_pos], 0);
                assert_ne!(rgba_data[cur_pixel_pos + G_OFFSET], 0);
                assert_ne!(rgba_data[cur_pixel_pos + B_OFFSET], 0);
                // Транспарентность 255 осталась
                assert_eq!(rgba_data[cur_pixel_pos + A_OFFSET], 255);
                // assert_eq!(rgba_data[cur_pixel_pos + A_OFFSET], 100);
            }
        }
    }
}