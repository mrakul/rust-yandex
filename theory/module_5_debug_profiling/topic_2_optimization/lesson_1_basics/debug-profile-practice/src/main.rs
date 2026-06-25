// Вызов отладчика:
// > rust-gdb target/debug/debug-profile-practice

// Полезные команды (в принципе, это gdb):

// break debug_profile_practice::sum_of_squares (или b debug_profile_practice::sum_of_squares) 
// (!) У меня требует обязательно имя крата
// => поставить точку останова (breakpoint) на начало функции.

// break slow_function — остановиться перед вызовом медленной функции
// Или break [номер строки]:
// (gdb) break 4
// Breakpoint 2 at 0x55555556886a: file src/main.rs, line 4.
// (gdb) 

// Или так:
// break src/main.rs:42

// delete (или d) с номером breakpoint - удалить
// disable/enable [номер?] - временно отключить/включить

// Условные breakpoint'ы:
// # Остановка, только когда переменная равна определённому значению
// (gdb) break src/main.rs:20 if x == 42

// # Остановка только после определённого количества итераций
// (gdb) break src/main.rs:15 if i > 100

// # Остановка, только когда указатель не равен null
// (gdb) break process_data if data != 0x0 

// Можно также установить обычную точку останова и затем добавить условие через команду condition. 
// Чтобы посмотреть условие точки останова, используйте info breakpoints. 
// Чтобы удалить условие, используйте condition N без аргументов — это удалит условие для указанного breakpoint.

// run (или r) — запустить программу до первой точки останова
// next (или n) — сделать один шаг, не заходя внутрь вызываемых функций (step over)
// step (или s) — сделать один шаг, заходя внутрь вызываемых функций (step into)

// print n (или p n) — вывести значение переменной n. Можно печатать выражения: p i * i
// info locals — показать все локальные переменные в текущей области видимости
// info registers — показать значения регистров процессора (как вы и просили)
// info breakpoints — информация о breakpoint'ах 
// info args — просмотр аргументов функции
// backtrace (bt) — стек вызовов

// frame (или f) с номером кадра -  перейти к другому кадру стека 
// Команды print и info locals показывают переменные этого кадра
// list (или l) - исходный код вокруг текущей строки. Можно указать функцию или диапазон строк для просмотра конкретного участка кода.

// continue (или c) — продолжить выполнение до следующей точки останова
// finish (или fin) — выполняет оставшуюся часть текущей функции и останавливается на строке, где функция была вызвана.

// set x = 42 — изменение переменных

// quit (или q) — выйти из отладчика.

// Пример вызовов:
// (gdb) n
// 7               sum += i;
// (gdb) n
// 7               sum += i;
// (gdb) n
// 9               slow_function()
// (gdb) n
// 5           for i in 0..n {
// (gdb) p sum
// $19 = 78

/*** Профилирование ***/

// (!) perf для WSL2 и нестанадртного ядра нужно установить отдельно так:
// Мне на WSL потребовались такие доустановит:
// sudo apt install libdwarf-dev libelf-dev flex bison libtraceevent-dev libunwind-dev libdw-dev

// И собрать из исходников:
// git clone https://github.com/microsoft/WSL2-Linux-Kernel --depth 1
// cd WSL2-Linux-Kernel/tools/perf
// make -j8
// sudo cp perf /usr/local/bin


// # -g включает запись стека вызовов (call-graph)
// # --call-graph dwarf использует отладочные символы для красивого стека
// > /usr/local/bin/perf record -g --call-graph dwarf ./target/release/debug-profile-practice

// // Вызов:
// > /usr/local/bin/perf record -g --call-graph dwarf ./target/release/debug-profile-practice
// Сумма квадратов 10 => 55
// Сумма квадратов 100 => 5050
// Сумма квадратов 1000 => 500500
// Slow: 99980001
// [ perf record: Woken up 1 times to write data ]
// [ perf record: Captured and wrote 0.021 MB perf.data (2 samples) ]

// И perf report:
// > /usr/local/bin/perf report

// Тут видно, что slow_function выполняется 92.8% времени, 

    // 92.80%     0.00%  debug-profile-p  debug-profile-practice  [.] debug_profile_practice::sum_of_squares (inlined)
    //         |
    //         ---debug_profile_practice::sum_of_squares (inlined)
    //            debug_profile_practice::slow_function (inlined)

// И успел столько сделать семплов:
// # Samples: 5K of event 'task-clock:uppp'
// # Event count (approx.): 1315500000

// Сумма квадратов до числа
fn sum_of_squares(n: u64) -> u64 {
    let mut sum = 0;

    for i in 0..=n {
        // Специально без квадрата для отладки
        sum += i;

        sum += slow_function();
    }

    sum
}

// Медланная функция
fn slow_function() -> u64 {
    
    let mut num: u64 = 0;
    
    for i in 0..100 { 
        for j in 0..100 {
            for k in 0..100 {
                // Так оптимизатор понимает конечное значение. Хииитренький! :)
                // num = i * j * k;

                // black_box "прячет" значения i и j от оптимизатора
                let i_bb = std::hint::black_box(i);
                let j_bb = std::hint::black_box(j);
                
                // Накапливаем сумму, используя wrapping_add, чтобы избежать паники при переполнении
                num = num.wrapping_add(i_bb * j_bb);
            }
        }
    }

    // Чтобы компилятор ничего не оптимизировал
    std::hint::black_box(num)
}

fn main() {
    // 10
    let mut num = 10;
    println!("Сумма квадратов {} => {}", num, sum_of_squares(num));
    
    // 100
    num = 100;
    println!("Сумма квадратов {} => {}", num, sum_of_squares(num));
    
    // 1000
    num = 1000;
    println!("Сумма квадратов {} => {}", num, sum_of_squares(num));


    println!("Slow: {}", slow_function());
}
