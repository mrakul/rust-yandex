
/*** DWARF-символы и профилирование ***/

fn accumulating() {
    let mut acc = 0u64;
    for i in 0..500_000_000 {
        acc = acc.wrapping_add(i ^ (i >> 3));
    }
    println!("result: {acc}");
}

fn main() {
    accumulating();
}

// 1. Выведите содержимое секций .debug_*:
// > readelf -S target/debug/dwarf-practice | grep debug 
//   [12] .debug_gdb_s[...] PROGBITS         000000000000a1a0  0000a1a0
//   [33] .debug_abbrev     PROGBITS         0000000000000000  00055138
//   [34] .debug_info       PROGBITS         0000000000000000  00056909
//   [35] .debug_aranges    PROGBITS         0000000000000000  0015b462
//   [36] .debug_str        PROGBITS         0000000000000000  00162f52
//   [37] .debug_line       PROGBITS         0000000000000000  002c8920
//   [38] .debug_ranges     PROGBITS         0000000000000000  00336bb3

// Вы увидите секции .debug_info, .debug_abbrev, .debug_line и т. д. 

// 2. Чтобы извлечь номера строк, напишите:
// TODO (!): надо устанавливать
// > llvm-dwarfdump --debug-line target/debug/instrumentation-demo | head -n 40 

// Или через objdump:
// > objdump --dwarf=info target/debug/dwarf-practice | less 

// Найти accumulating:
// > objdump --dwarf=info target/debug/dwarf-practice | grep accumulating
//     <c46>   DW_AT_linkage_name: (indirect string, offset: 0x2beeb): _ZN14dwarf_practice12accumulating17ha19085ed415484b6E
//     <c4a>   DW_AT_name        : (indirect string, offset: 0x15a5ee): accumulating

// Там будет видно, что адреса функций сопоставлены со строками main.rs. GDB или lldb используют ровно эти данные, чтобы показать исходный код. Если запустить gdb target/debug/instrumentation-demo, можно поставить breakpoint по имени main и посмотреть, как отладчик мгновенно находит нужную строку благодаря DWARF.

// 3. nm тоже показывает менглированные символы:
// nm target/debug/dwarf-practice | grep accumulating
// 0000000000014510 t _ZN14dwarf_practice12accumulating17ha19085ed415484b6E

// 4. Дополнительно можно проверить символы без DWARF:
// > strip target/debug/instrumentation-demo -o stripped 
// А затем запустить readelf -S stripped — разделов .debug_* больше нет, и отладчик покажет только адреса. 
// В реальной жизни символы хранят отдельно (например, target/debug/.debug/instrumentation-demo.debug),
// а в прод деплоится stripped-бинарь плюс отладочная информация на сервере символов.

/*** Профилирование ***/

// Собираем с релизной сборкой:
// > cargo build --release 

// Записать данные:
// /usr/local/bin/perf record -g -- target/release/dwarf-practice
// result: 1251976370825152
// [ perf record: Woken up 1 times to write data ]
// [ perf record: Captured and wrote 0.010 MB perf.data (101 samples) ]

// (!) Опция -g говорит perf собирать стеки вызовов вместе с сэмплами. После запуска появится файл perf.data с записями о том, какие адреса чаще всего встречались в собранных сэмплах. Чтобы расшифровать эти адреса в имена функций и строки, perf будет использовать DWARF и таблицу символов из release-бинаря. Если вы отключите генерацию отладочной информации и полностью «зачистите» бинарь, имена функций исчезнут и анализ станет почти бессмысленным.
// Дальше можно построить flamegraph с помощью стандартного скрипта:

// Можно проверить через отчёт:
// > /usr/local/bin/perf report

// Через flamegraph:
// /usr/local/bin/perf script | flamegraph.pl > flame.svg

// Но надо установить: или из официального источника, или через cargo: cargo install flamegraph
// Способ А (Прямая замена вашей команды):

// Пакет предоставляет утилиту flamegraph, которая умеет парсить вывод perf:
// /usr/local/bin/perf script | flamegraph --from-perf-script > flame.svg

// /usr/local/bin/perf script | flamegraph --from-perf-script > flame.svg

// Используйте код с осторожностью.
// Способ Б (Самый простой — без ручного вызова perf):
// Вы можете вообще не вызывать perf record и perf script вручную. Утилита сделает всё сама, автоматически подтянув отладочные символы Rust:
// (+) cargo flamegraph --bin dwarf-practice


// Открыв flame.svg в браузере, вы увидите полосы, каждая из которых соответствует функции в стеке. Чем шире полоса, тем больше времени суммарно проводит код в этой функции и ниже по стеку. Для нашего искусственного примера основной вклад будет в цикл и связанные с ним операции. В реальном проекте вы увидите свои функции, вызовы библиотек, системные вызовы и сможете понять, где именно тратится CPU.
// Для корректного восстановления стеков в сильно оптимизированном Rust-коде иногда полезно включать сохранение указателей на кадры стека. Это можно сделать через настройку профиля в Cargo.toml, передав нужные флаги компилятору, если стандартного поведения недостаточно. В противном случае профилировщик всё равно будет работать, но некоторые стеки могут получаться усечёнными или «рваными» из-за агрессивных оптимизаций.