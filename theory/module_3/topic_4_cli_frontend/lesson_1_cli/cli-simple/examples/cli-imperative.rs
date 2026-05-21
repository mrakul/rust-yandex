use clap::{Arg, ArgMatches, Command};

// Императивный (через Command API)
// Работает через Command API. Это builder-стиль для сложных случаев, требующих runtime-конфигурации.
// Он позволяет динамически описывать команды, поэтому полезен, когда конфигурация CLI формируется во время выполнения. 
// Такое бывает, например, с плагинами или расширениями.

fn main() {
    // Определяем структуру CLI
    let matches = Command::new("bank")
        .version("1.0")
        .about("Банковская CLI-утилита для перевода денег и просмотра баланса")
        .subcommand_required(true)
        .arg_required_else_help(true)
        // cargo run --example cli-imperative -- balance --user Ivan
        .subcommand(
            Command::new("balance")
                .about("Показать баланс клиента")
                .arg(
                    Arg::new("user")
                        .short('u')
                        .long("user")
                        .help("Имя пользователя")
                        .required(true)
                        .value_name("USER"),
                ),
        )
        // cargo run --example cli-imperative -- deposit --user Ivan --amount 100
        .subcommand(
            Command::new("deposit")
                .about("Перевести на баланс")
                .arg(
                    Arg::new("user")
                        .short('u')
                        .long("user")
                        .help("Имя пользователя")
                        .required(true)
                        .value_name("USER"),
                )
                .arg(
                    Arg::new("amount")
                        .short('a')
                        .long("amount")
                        .help("Сумма зачисления")
                        .required(true)
                        .value_name("AMOUNT"),
                ),
        )
        .subcommand(
            Command::new("transfer")
                .about("Перевести средства между пользователями")
                .arg(
                    Arg::new("from")
                        .short('f')
                        .long("from")
                        .help("Отправитель")
                        .required(true)
                        .value_name("FROM"),
                )
                .arg(
                    Arg::new("to")
                        .short('t')
                        .long("to")
                        .help("Получатель")
                        .required(true)
                        .value_name("TO"),
                )
                .arg(
                    Arg::new("amount")
                        .short('a')
                        .long("amount")
                        .help("Сумма перевода")
                        .required(true)
                        .value_name("AMOUNT"),
                ),
        )
        .subcommand(
            Command::new("history")
                .about("Показать историю транзакций")
                .arg(
                    Arg::new("user")
                        .short('u')
                        .long("user")
                        .help("Имя пользователя (опционально)")
                        .required(false)
                        .value_name("USER"),
                ),
        )
        .get_matches();

    // Обработка подкоманд
    match matches.subcommand() {
        Some(("balance", sub_m)) => balance(sub_m),
        Some(("deposit", sub_m)) => deposit(sub_m),
        Some(("transfer", sub_m)) => transfer(sub_m),
        Some(("history", sub_m)) => history(sub_m),
        _ => unreachable!("Неизвестная команда"),
    }
}

// ==== Реализация подкоманд ====

fn balance(matches: &ArgMatches) {
    let user = matches.get_one::<String>("user").unwrap();
    println!("Баланс пользователя {user}: 1000₽");
}

fn deposit(matches: &ArgMatches) {
    let user = matches.get_one::<String>("user").unwrap();

        // Получаем как строку и парсим в f64
    let amount_str = matches.get_one::<String>("amount").unwrap();
    let amount: f64 = amount_str.parse().expect("Некорректная сумма перевода");

    println!("Пополнение {amount}₽ пользователя {user}");
}

fn transfer(matches: &ArgMatches) {
    let from = matches.get_one::<String>("from").unwrap();
    let to = matches.get_one::<String>("to").unwrap();
    
    // Получаем как строку и парсим в f64
    let amount_str = matches.get_one::<String>("amount").unwrap();
    let amount: f64 = amount_str.parse().expect("Некорректная сумма перевода");

    println!("Переводим {amount}₽ от {from} к {to}");
}

fn history(matches: &ArgMatches) {
    match matches.get_one::<String>("user") {
        Some(user) => println!("История операций пользователя {user}: ..."),
        None => println!("История всех операций: ..."),
    }
} 