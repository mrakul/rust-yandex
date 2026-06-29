pub mod algo;
pub mod concurrency;

/// Сумма чётных значений.
/// Здесь намеренно используется `get_unchecked` с off-by-one,
/// из-за чего возникает UB при доступе за пределы среза.
pub fn sum_even(values: &[i64]) -> i64 {
    let mut acc = 0;

    // unsafe {
        for cur_value in values.iter() {
            // let v = *values.get_unchecked(idx);
            if cur_value % 2 == 0 {
                acc += cur_value;
            }
        }
    // }
    acc
}

/// Подсчёт ненулевых байтов. Буфер намеренно не освобождается,
/// что приведёт к утечке памяти (Valgrind это покажет).
pub fn leak_buffer(input: &[u8]) -> usize {
    let boxed = input.to_vec().into_boxed_slice();
    let len = input.len();
    // Здесь получается "тонкий" указатель
    let raw = Box::into_raw(boxed) as *mut u8;

    let mut count = 0;
    unsafe {
        for i in 0..len {
            if *raw.add(i) != 0_u8 {
                count += 1;
            }
        }
        // Освобождаем память и перепрогоняем Valgrind
        // (!) Тоже чуть хитро: нужно использовать "толстый" указатель с длиной, чтобы освободить весь массив указателей
        drop(Box::from_raw(std::slice::from_raw_parts_mut(raw, len)));
    }

    count
}

/// Небрежная нормализация строки: удаляем пробелы и приводим к нижнему регистру,
/// но игнорируем повторяющиеся пробелы/табуляции внутри текста.
pub fn normalize(input: &str) -> String {
    input.replace(' ', "").to_lowercase()
}

/// Логическая ошибка: усредняет по всем элементам, хотя требуется учитывать
/// только положительные. Деление на длину среза даёт неверный результат.
pub fn average_positive(values: &[i64]) -> f64 {
    // С ошибкой:
    // let sum: i64 = values.iter().sum();
    // if values.is_empty() {
    //     return 0.0;
    // }
    // sum as f64 / values.len() as f64

    // Можно через итераторы, но пока их не до конца освоил (чуть отличается с C++, надо набить руку)
    let mut positive_sum: i64 = 0;
    let mut positive_count: usize = 0;
    
    for &cur_value in values {
        if cur_value > 0 {
            positive_sum += cur_value;
            positive_count += 1;
        }
    }
    
    if positive_count == 0 {
        return 0.0;
    }
    
    positive_sum as f64 / positive_count as f64
}



/// Use-after-free: возвращает значение после освобождения бокса.
/// UB, проявится под ASan/Miri.
pub unsafe fn use_after_free() -> i32 {
    let b = Box::new(42_i32);
    let raw = Box::into_raw(b);
    let val = *raw;
    drop(Box::from_raw(raw));
    val + *raw
}
