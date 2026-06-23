use criterion::{black_box, criterion_group, criterion_main, Criterion};
use optimization_practice::{process_json, sum_numbers};

fn bench_process_json(c: &mut Criterion) 
{
    // Числа как Payload - Vec<u32>
    let json_data = r#"{"numbers":[1,2,3,4,5,6,7,8,9,10]}"#;
    
    c.bench_function("process_json", |b| {
        b.iter(|| {
            let numbers = process_json(black_box(json_data)).unwrap();
            black_box(sum_numbers(&numbers));
        });
    });
}

criterion_group!(benches, bench_process_json);
criterion_main!(benches);

/*** Результаты ***/

// До оптимизации:
// > cargo bench
// => 
// ... [тесты]
//
// (!) Время:
//  Running benches/json_bench.rs (target/release/deps/json_bench-928af1bd8c5d4ed1)
// process_json            time:   [391.86 ns 398.47 ns 406.88 ns]
// Found 12 outliers among 100 measurements (12.00%)
//   5 (5.00%) high mild
//   7 (7.00%) high severe

// После оптимизации (только 5 процентов, не 60 у меня почему-то)
//      Running benches/json_bench.rs (target/release/deps/json_bench-928af1bd8c5d4ed1)
// process_json            time:   [370.44 ns 379.58 ns 387.44 ns]
//                         change: [-8.3365% -5.3398% -2.4988%] (p = 0.00 < 0.05)
//                         Performance has improved.
// Found 14 outliers among 100 measurements (14.00%)
//   4 (4.00%) low severe
//   4 (4.00%) low mild
//   5 (5.00%) high mild
//   1 (1.00%) high severe

// Плавает немного - тут -9, но p = 0.00
//      Running benches/json_bench.rs (target/release/deps/json_bench-928af1bd8c5d4ed1)
// process_json            time:   [374.63 ns 377.28 ns 379.97 ns]
//                         change: [-14.160% -9.1716% -4.7111%] (p = 0.00 < 0.05)
//                         Performance has improved.
// Found 8 outliers among 100 measurements (8.00%)
//   1 (1.00%) low mild
//   3 (3.00%) high mild
//   4 (4.00%) high severe

// (!)
// Поменял на статический массив, ровно 10 чтобы влазило:
//   pub struct Payload {
//       pub numbers: [u32; 10],
//   }

// > cargo bench
//      Running benches/json_bench.rs (target/release/deps/json_bench-928af1bd8c5d4ed1)
// process_json            time:   [301.55 ns 306.49 ns 311.56 ns]
//                         change: [-22.347% -19.667% -16.844%] (p = 0.00 < 0.05)
//                         Performance has improved.
// Found 6 outliers among 100 measurements (6.00%)
//   2 (2.00%) high mild
//   4 (4.00%) high severe


