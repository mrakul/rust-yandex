use broken_app::{algo, leak_buffer, normalize, sum_even};

fn main() {
    // - Сделал цикл, чтобы время выполнения было 3-5 секунд, и при этом сохранились пропорции в стеке-snapshot'е, снимаемом perf'ом
    // - Обернул вызовы в black_box, чтобы не соптимизировались
    // - Убрал println'ы. Можно оставить, но я убрал :)
    // - Входные данные те же
    for _ in 0..=100000
    {
        std::hint::black_box(sum_even(&[1, 2, 3, 4]));

        std::hint::black_box(leak_buffer(&[1_u8, 0, 2, 3]));

        std::hint::black_box(normalize(" Hello World "));

        std::hint::black_box(algo::slow_fib(20));

        std::hint::black_box(algo::slow_dedup(&[1, 2, 2, 3, 1, 4, 4]));
    }

    println!("Конец обработки");
}
