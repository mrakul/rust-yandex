use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser)]
#[command(name = "blog-cli")]
#[command(about = "CLI-утилита для блога", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Адрес сервера (по умолчанию: http://localhost:3000, где должен быть HTTP)
    #[arg(long, default_value = "http://localhost:3000")]
    server: String,
    
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
        /// Количество постов на странице (дефолтное: 10)
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
    
    let cli = Cli::parse();
    let transport = if cli.grpc {
        Transport::Grpc(cli.server)
    } else {
        Transport::Http(cli.server)
    };
    
    let mut client = BlogClient::new(transport)?;
    
    // Try to load saved token
    if let Ok(token) = load_token() {
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

fn load_token() -> Result<String, Box<dyn std::error::Error>> {
    let token = fs::read_to_string(TOKEN_FILE_NAME)?;
    Ok(token.trim().to_string())
}

fn save_token(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(TOKEN_FILE_NAME, token)?;
    Ok(())
}

// Регистрация
// cargo run -p blog-cli -- register --username "bob" --email "bob@example.com" --password "secret123"

// Логин
// cargo run -p blog-cli -- login --username "bob" --password "secret123"

// Создать пост
// cargo run -p blog-cli -- create --title "Мой первый пост" --content "Содержание"

// Пост через gRPC
// cargo run -p blog-cli -- create --title "Мой первый пост" --content "Содержание" --grpc

// Получить пост
// cargo run -p blog-cli -- get --id 1

// Обновить пост, только заголовок 
// cargo run -p blog-cli -- update --id 1 --title "Обновлённый заголовок"

// Удалить пост
// cargo run -p blog-cli -- delete --id 1

// Список с limit и offset
// cargo run -p blog-cli -- list --limit 20 --offset 0

// Дефлотные значения
// cargo run -p blog-cli -- list

// Обновить только контент
// cargo run -p blog-cli -- update --id 1 --content "Обновлённый контент"
