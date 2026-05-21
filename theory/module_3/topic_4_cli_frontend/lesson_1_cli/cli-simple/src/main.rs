use clap::{Parser, Subcommand};

// Декларативный стиль через макросы: формат команд формируется на этапе компиляции

#[derive(Parser)]
#[command(name = "bank", about = "Банковская CLI-утилита", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Тут прям описание команд, help генерируется автоматически
#[derive(Subcommand)]
enum Commands {
    // Примечание, как везде: для автогенерации название из enum с большими буквами соответствует команде с маленькими

    /// Показать баланс клиента [это описание попадает прям в help - cargo run -- help]
    Balance {
        #[arg(short, long, help = "[Тут описание параметра, выводимое при cargo run -- help, но не особо выводится]")]
        user: String,
    },
    /// Добавление на счёт пользователя
    Deposit {
        #[arg(short, long)]
        user: String,

        #[arg(short, long)]
        amount: f64,
    },
    /// Перевести средства
    Transfer {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: f64,
    },
    /// Показать историю транзакций
    History {
        #[arg(short, long)]
        // Тут интересно, что может быть, а может не быть - Option
        user: Option<String>,
    },

    // #[arg(default_value = "100")] => значение по умолчанию
    // #[arg(required = true)] =>  указывает, что параметр обязателен для передачи.
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // cargo run -- balance --user Ivan
        Commands::Balance { user } => {
            println!("Баланс пользователя {user}: 1000₽");
        }
        
        // cargo run -- depost  Ivan --amount 250
        Commands::Deposit { user, amount } => {
            println!("Добавление {amount}₽ пользователю {user}");
        }

        // cargo run -- transfer --from Ivan --to Maria --amount 250
        Commands::Transfer { from, to, amount } => {
            println!("Переводим {amount}₽ от {from} к {to}");
        }

        // cargo run -- history --user Ivan
        Commands::History { user } => {
            match user {
                Some(u) => println!("История операций пользователя {u}: ..."),
                None => println!("История всех операций: ..."),
            }
        }

        // С ошибкой парсинга - выдаст error: invalid value 'money' for '--amount <AMOUNT>': invalid float literal
        // > cargo run -- transfer --from Ivan --to Maria --amount money
    }
} 