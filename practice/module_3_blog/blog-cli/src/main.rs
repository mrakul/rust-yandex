use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};
use std::fs;

// Адреса серверов по умолчанию (больше для gRPC, чтоб каждый раз не писать)
const DEFAULT_HTTP_SERVER_ADDR: &str = "http://localhost:3000";
const DEFAULT_GRPC_SERVER_ADDR: &str = "http://127.0.0.1:50051";

#[derive(Parser)]
#[command(name = "blog-cli")]
#[command(about = "CLI-утилита для блога", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Адрес сервера (по умолчанию: http://127.0.0.1:3000, где должен быть HTTP)
    // #[arg(long, default_value = "http://127.0.0.1:3000")]

    // Убрал умолчание, дефолтные значения берутся в зависимости от типа транспорта
    #[arg(long)]
    server: Option<String>,
    
    /// Использовать gRPC транспорт вместо HTTP
    #[arg(long)]
    grpc: bool
}

#[derive(Subcommand)]
enum Commands {
    /// Регистрация нового пользователя
    Register {
        #[arg(long)]
        username: String,
        
        #[arg(long)]
        email: String,
        
        #[arg(long)]
        password: String,
    },
    
    /// Логин с получением JWT-токена в файлик
    Login {
        #[arg(long)]
        username: String,
        
        #[arg(long)]
        password: String,
    },
    
    /// Создание поста
    Create {
        #[arg(long)]
        title: String,
        
        #[arg(long)]
        content: String,
    },
    
    /// Пост по ID
    Get {
        #[arg(long)]
        id: i64,
    },
    
    /// Обновление поста
    Update {
        #[arg(long)]
        id: i64,
        
        #[arg(long)]            // TODO: проверить, должны быть Option 
        title: Option<String>,
        
        #[arg(long)]            // TODO: проверить, должны быть Option
        content: Option<String>,
    },
    
    /// Удалить пост по ID
    Delete {
        #[arg(long)]
        id: i64,
    },
    
    /// Список постов с limit и ofsset
    List {
        /// Количество постов на странице (дефолтное: 10 в коде)
        #[arg(long)]
        limit: Option<i64>,
        
        /// Оффсет по постам (дефолтное: 0)
        #[arg(long)]
        offset: Option<i64>,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Пока, вроде, не нужно
    // dotenvy::dotenv().ok();
    
    // Парсим командную строку
    let cli = Cli::parse();

    // Адрес в зависимости от указания в команде server'а
    let server_addr = match cli.server {
        Some(cli_addr) => {
            println!("Адрес сервера из команды: {}", cli_addr);
            cli_addr
        },
        None => {
            if !cli.grpc {
                DEFAULT_HTTP_SERVER_ADDR.to_string()
            } else {
                DEFAULT_GRPC_SERVER_ADDR.to_string()
            }
        }
    };

    // Только тип транспорта, адрес определён
    let transport = if !cli.grpc {
        Transport::Http(server_addr)
    } else {
        Transport::Grpc(server_addr)
    };
    
    // Переделал под асинхронный вариант с await из-за gRPC'шного connect
    let mut client = BlogClient::new(transport).await?;

    // Прочитать токен из файла
    if let Ok(token) = read_token_from_file() {
        client.set_token(token);
    }
    
    match cli.command {
        Commands::Register { username, email, password } => {
            // Регистрация с сохранением токена
            match client.register(username, email, password).await {
                Ok(response) => {
                    println!("Пользователь успешно зарегистрирован");
                    println!("ID пользователя: {}", response.user.id);
                    println!("Имя: {}", response.user.username);
                    
                    // Сохраняем токен при регистрации
                    save_token(&response.token)?;
                    println!("Токен сохранён в .blog_token");
                }
                Err(e) => {
                    eprintln!("Ошибка регистрации: {}", e);
                    // Тут и ниже делаю через возврат ошибки, если потом переделаю под interac
                    return Err(e.into());
                }
            }
        }
        
        Commands::Login { username, password } => {
            // Логин
            match client.login(username, password).await {
                Ok(response) => {
                    println!("Логин: ");
                    println!("ID пользователя: {}", response.user.id);
                    println!("Имя пользователя: {}", response.user.username);
                    
                    // Сохраняем токен
                    save_token(&response.token)?;
                    println!("Токен сохранён в .blog_token");
                }
                Err(e) => {
                    eprintln!("Ошибка логина: {}", e);

                    return Err(e.into());
                }
            }
        }
        
        Commands::Create { title, content } => {
            // Создание поста
            match client.create_post(title, content).await {
                Ok(post) => {
                    println!("Пост создан");
                    println!("ID поста: {}", post.id);
                    println!("Заголовок: {}", post.title);
                    println!("ID автора: {}", post.author_id);
                    println!("Создан: {}", post.created_at);
                }
                Err(e) => {
                    eprintln!("Ошибка создания поста: {}", e);

                    return Err(e.into());
                }
            }
        }
        
        Commands::Get { id } => {
            // Получение поста
            match client.get_post(id).await {
                Ok(post) => {
                    println!("Пост #{}", post.id);
                    println!("Заголовок: {}", post.title);
                    println!("Содержимое: {}", post.content);
                    println!("ID автора: {}", post.author_id);
                    println!("Создан: {}", post.created_at);
                    println!("Обновлён: {}", post.updated_at);
                }
                Err(e) => {
                    eprintln!("Ошибка получение поста: {}", e);
                    return Err(e.into());
                }
            }
        }
        
        Commands::Update { id, title, content } => {
            // Обновление поста
            match client.update_post(id, title, content).await {
                Ok(post) => {
                    println!("Пост успешно обновлён!");
                    println!("ID поста: {}", post.id);
                    println!("Заголовок: {}", post.title);
                    println!("Содержимое: {}", post.content);
                    println!("Время: {}", post.updated_at);
                }
                Err(e) => {
                    eprintln!("Ошибка обновления поста: {}", e);

                    return Err(e.into());
                }
            }
        }
        
        Commands::Delete { id } => {
            // Удаление поста
            match client.delete_post(id).await {
                Ok(()) => {
                    println!("Пост #{} удалён!", id);
                }
                Err(e) => {
                    eprintln!("Ошибка удаления поста: {}", e);

                    return Err(e.into());
                }
            }
        }
        
        Commands::List { limit, offset } => {
            // Вывести список постов
            match client.list_posts(limit, offset).await {
                Ok(response) => {
                    println!("Постов: {}", response.total);
                    println!("Выведены: {}-{}, Limit: {}", 
                        response.offset + 1, 
                        response.offset + response.posts.len() as i64, 
                        response.limit
                    );
                    
                    if response.posts.is_empty() {
                        println!("Список по запросу пуст");
                    } else {
                        for post in response.posts {
                            println!("- #{}: {} (пользователь #{})", post.id, post.title, post.author_id);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка получения списка постов: {}", e);

                    return Err(e.into());
                }
            }
        }
    }
    
    Ok(())
}

// Пишем в файлик в рабочей директории
const TOKEN_FILE_NAME: &str = ".blog_token";

fn read_token_from_file() -> Result<String, Box<dyn std::error::Error>> {
    let token = fs::read_to_string(TOKEN_FILE_NAME)?;
    Ok(token.trim().to_string())
}

fn save_token(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(TOKEN_FILE_NAME, token)?;
    Ok(())
}

/*** Команды для проверки: HTTP и gRPC попарно ***/

// Регистрация
// cargo run -p blog-cli -- register --username "misha" --email "misha@misha.com" --password "12345678"
// cargo run -p blog-cli -- --grpc register --username "misha2" --email "misha2@misha.com" --password "12345678"

// Логин
// cargo run -p blog-cli -- login --username "misha" --password "12345678"
// cargo run -p blog-cli -- --grpc login --username "misha2" --password "12345678"

// Создать пост
// cargo run -p blog-cli -- create --title "Мой первый пост" --content "Содержание"
// cargo run -p blog-cli -- --grpc create --title "Мой второй пост" --content "Содержание"

// Получить пост
// cargo run -p blog-cli -- get --id 1
// cargo run -p blog-cli -- --grpc get --id 2

// Обновить пост, только заголовок 
// cargo run -p blog-cli -- update --id 1 --title "Обновлённый заголовок HTTP"
// cargo run -p blog-cli -- --grpc update --id 1 --title "Обновлённый заголовок gRPC"

// Обновить пост, только содержимое 
// cargo run -p blog-cli -- update --id 1 --content "Обновлённое содержание HTTP"
// cargo run -p blog-cli -- --grpc update --id 1 --content "Обновлённое содержание gRPC"

// Удалить пост
// cargo run -p blog-cli -- delete --id 1
// cargo run -p blog-cli -- --grpc delete --id 1

// Список с limit и offset
// cargo run -p blog-cli -- list --limit 2 --offset 1
// cargo run -p blog-cli -- --grpc list --limit 2 --offset 1

// Дефлотные значения
// cargo run -p blog-cli -- list
// cargo run -p blog-cli -- --grpc list