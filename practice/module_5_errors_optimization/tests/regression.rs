use broken_app::{leak_buffer, sum_even, average_positive};

#[test]
fn regression_sum_even_no_ub_on_boundary() {
    // Пустой массив => старый код с off-by-one мог бы попытаться прочитать 0-й элемент
    let empty_vec: &[i64] = &[];
    assert_eq!(sum_even(empty_vec), 0);

    // Один элемент => пограничный случай для 0..=len
    assert_eq!(sum_even(&[2]), 2);
    assert_eq!(sum_even(&[1]), 0);

    // Смешанный массив
    let mixed_vec = [1, 2, 3, 4, 5, 6];
    assert_eq!(sum_even(&mixed_vec), 12);
}


#[test]
fn regression_average_positive_ignores_non_positive() {
    let mixed_vec = [-10, 0, 10, 20];
    let result = average_positive(&mixed_vec);
    // Старый код посчитал бы отрциательные
    assert!((result - 15.0).abs() < f64::EPSILON, 
            "Функция не должна учитывать <= 0. Ожидалось 15.0, получено {}", result);

    // Только неположительные
    let non_positive_vec = [-5, -5, 0, -25000];
    let result_no_pos = average_positive(&non_positive_vec);
    assert!((result_no_pos - 0.0).abs() < f64::EPSILON, 
             "Должно быть 0.0");
}

#[test]
fn regression_leak_buffer_correctness_and_no_crash() {
    // Буфер нулевого нулевого размера не должно приводить к UB (или панике)
    assert_eq!(leak_buffer(&[]), 0);

    // Все нули
    assert_eq!(leak_buffer(&[0, 0, 0]), 0);

    // Все ненулевые => 3
    assert_eq!(leak_buffer(&[1, 2, 3]), 3);

    // Смешанный => 3 ненулевых
    assert_eq!(leak_buffer(&[1, 0, 2, 0, 3]), 3);
    
    // Большой буфер => проверка, аллокатор корректно освобождает большой кусок памяти
    // Десять тыщ двоек!)
    let large_buffer: Vec<u8> = vec![2; 10000];
    assert_eq!(leak_buffer(&large_buffer), 10000);
}