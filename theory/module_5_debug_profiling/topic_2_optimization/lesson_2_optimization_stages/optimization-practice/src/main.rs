use optimization_practice::{process_json, sum_numbers};

// Как и в прошлых уроках, сразу делает и perf, и строит flamegraph:
// > cargo flamegraph --bin optimization-practice

// Бенчи:
// Запустите бенчмарк:
// > cargo bench

// Изучите результаты:

//     Откройте HTML-отчёт в target/criterion/process_json/index.html.
//     Запишите медианное время выполнения.


fn main() {
    let json_data = r#"{"numbers":[1,2,3,4,5,6,7,8,9,10]}"#;
    // Разбираем один раз и переиспользуем — ключевая оптимизация
    let payload = process_json(json_data).expect("parse failed");
    
    for _ in 0..1_000 {
        let _sum = sum_numbers(&payload);
    }
} 

// До оптимизации

// fn main() {
//     // Числа как Payload - Vec<u32>
//     let json_data = r#"{"numbers":[1,2,3,4,5,6,7,8,9,10]}"#;

//     // 100000 итераций на каждую функцию
//     for _ in 0..100000 {
//         let numbers = process_json(json_data).unwrap();
//         let _sum = sum_numbers(&numbers);
//     }
// }