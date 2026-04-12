use std::io::BufRead;
use std::io::BufReader;
use std::time::Duration;
use std::io::Write;
use std::net::{TcpStream, TcpListener};
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use std::net::SocketAddr;
use rand::Rng;
use std::net::UdpSocket;
use std::path::Path;

use quotes_streaming::quotes::{StockQuote, QuoteGenerator};

fn main() -> std::io::Result<()> {
    
    /* 1. Запускаем и инициализируем генератор котировок в отдельном потоке */
    
    // Новый генератор через Arc для передачи в потоки
    let quotes_generator_arc = Arc::new(QuoteGenerator::new());
    let quotes_generator_clone = quotes_generator_arc.clone();

    // Запускаем поток генератора цен до парсинга команд
    std::thread::spawn(move || -> std::io::Result<()> {
        let loaded_count = quotes_generator_clone.load_tickers_from_file(Path::new("aux/tickers.txt"))?;
        println!("Загружено {} компаний: \n", loaded_count);

        loop {
            quotes_generator_clone.update_prices();
            quotes_generator_clone.broadcast_quotes_to_subscribers();

            std::thread::sleep(Duration::from_secs(2));
        }

        // Ok(())
    });

    /* 2. Слушаем и обрабатываем входящие соединения */

    let server_addr_port = "127.0.0.1:11000";
    let listener = TcpListener::bind(server_addr_port)?;
    println!("Сервер слушает на порту 11000");

    // Фактически бесконечный цикл, при возникновении соединения создаёт новый сервер
    // (блокирующий вызов .incoming(), аналог accept() в цикле)
    for stream_result in listener.incoming() {
        match stream_result {
            Ok(to_client_stream) => {
                // Клонируем на каждой итерации
                let quotes_generator_clone = quotes_generator_arc.clone();

                // Здесь: main создаёт соединения с обработкой клиента, то есть каждый клиент обрабатывается
                // в своём потоке, фактически каждое соединение
                // Но при этом vault - разделяемое между потоками через Arc::clone(&vault)
                thread::spawn(move || {
                    server_process_request(to_client_stream, quotes_generator_clone);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    // Подключение классическое:
    // > nc 127.0.0.1 7878

    Ok(())
}

pub fn server_process_request(stream: TcpStream, quotes_generator: Arc<QuoteGenerator>) {

    // let client_addr = match stream.peer_addr() {
    //     Ok(addr) => addr,
    //     Err(e) => {
    //         eprintln!("Failed to get peer address: {}", e);
    //         return;
    //     }
    // };
    
    // let server_addr = match stream.local_addr() {
    //     Ok(addr) => addr,
    //     Err(e) => {
    //         eprintln!("Failed to get local address: {}", e);
    //         return;
    //     }
    // };

    let client_addr = stream.peer_addr().expect("failed to get client address");
    let server_addr = stream.local_addr().expect("failed to get server address");

    // клонируем stream: один экземпляр для чтения (обёрнут в BufReader), другой — для записи
    let mut to_client_stream = stream.try_clone().expect("failed to clone stream");
    let mut to_server_stream = BufReader::new(stream);

    let welcome_string = format!("Вы подключились к серверу: {} => {} \nВведите команду: STREAM, PING \n> ",
                                                                     client_addr, server_addr
    );
    
    // Отправляем Welcome клиенту
    if let Err(e) = to_client_stream.write_all(welcome_string.as_bytes()) {
        eprintln!("Не удалось отправить Welcome-сообщение: {}", e);
        return;
    }
    let _ = to_client_stream.flush();

    let mut command_from_client = String::new();
    let mut response = String::new();

    loop {
        // Очищаем входную строку и главное - response        
        command_from_client.clear();
        response.clear();

        // read_line ждёт '\n' — nc отправляет строку по нажатию Enter
        match to_server_stream.read_line(&mut command_from_client) {
            Ok(0) => {
                // EOF — клиент закрыл соединение
                return;
            }
            // Успешно прочитали line
            Ok(_) => {

                // Пустой ввод
                let command_from_client = command_from_client.trim();
                if command_from_client.is_empty() {
                    response = "Вы ничего не ввели. Введите команду в формате: STREAM udp://host:port TICKER1,TICKER2\n>".to_string();
                    let _ = to_client_stream.write_all(response.as_bytes());
                    let _ = to_client_stream.flush();

                    let _ = to_client_stream.flush();
                    continue;
                }

                match StreamCommand::parse(&command_from_client) {
                    Ok(stream_command_ok ) => {
                        // Создание нового соединения UDP
                        // 👇 Handle errors here, before spawning thread
                        let server_udp_socket_result = UdpSocket::bind("127.0.0.1:0");
                        
                        let server_udp_socket = match server_udp_socket_result {
                            Ok(socket) => socket,
                            Err(e) => {
                                response = format!("ERROR: Failed to bind UDP socket: {}", e);
                                return; // or handle error appropriately
                            }
                        };
                        
                        let server_addr_port = match server_udp_socket.local_addr() {
                            Ok(addr) => addr,
                            Err(e) => {
                                response = format!("ERROR: Failed to get local address: {}", e);
                                return;
                            }
                        };
                        
                        response = format!("Начало стриминга: {} → {}\n>", server_addr_port, stream_command_ok.client_addr_port); 
                        println!("Начало стриминга: {} → {}", server_addr_port, stream_command_ok.client_addr_port);
                        
                        // 👇 Clone shared resources for the thread
                        // let client_addr_port = stream_command_ok.client_addr_port; // Clone the addr
                        // let tickers = stream_command_ok.tickers.clone(); // Clone tickers
                        
                        /// Handle a STREAM command: spawn thread + start streaming
                        // pub fn handle_stream_command(&self, cmd: StreamCommand) -> Result<(), String> {
                        // 👇 Create UDP socket with OS-assigned port (0 = ephemeral port)
                        
                        // // TODO: обработка ошибки
                        // let server_udp_socket = UdpSocket::bind("127.0.0.1:0");
                        //     // .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;
                            
                        // // TODO: обработка ошибки
                        // let server_addr_port = server_udp_socket.unwrap().local_addr();
                        //     // .map_err(|e| format!("Failed to get local address: {}", e))?;
                            
                        println!("Начало стриминга: {} → {}", server_addr_port, stream_command_ok.client_addr_port);
                            
                        // 👇 Clone shared resources for the thread
                        let quotes_generator_clone = quotes_generator.clone();
                        // let tickers = cmd.tickers.clone();
                        // let target_addr = cmd.target_addr;


                        // Регистрация клиента и 
                        let read_from_gen_channel = match quotes_generator_clone.register_udp_streaming(
                                                                    stream_command_ok.client_addr_port, stream_command_ok.tickers) {
                            Ok(receiver) => receiver,
                            Err(e) => {
                                response = format!("ERROR: Внутренняя ошибка для регистрации подисок на тикеры : {}", e);
                                break; // Use break instead of return
                            }
                        };

                        // Поток UDP-стриминга под нового клиента 
                        thread::spawn(move || {
                            // Читаем из каналов
                            loop {
                                match read_from_gen_channel.recv() {
                                    Ok(read_quote) => {
                                        let data = read_quote.to_string() + "\n";
                                        if let Err(e) = server_udp_socket.send_to(data.as_bytes(), stream_command_ok.client_addr_port) {
                                            eprintln!("❌ Ошибка отправки {}: {}", stream_command_ok.client_addr_port, e);
                                            break;
                                        }
                                        println!("📤 Отправлено: {} → {}", read_quote.ticker, stream_command_ok.client_addr_port);
                                    }
                                    Err(_) => {
                                        // Ошибка чтения из канала
                                        println!("Ошибка чтения из канала {}", stream_command_ok.client_addr_port);
                                        continue;
                                    }
                                }
                            
                                // Пусть пока читают сразу по несколько тиков
                                thread::sleep(Duration::from_secs(5));
                            }



                            // Предыдущая реализация напрямую через generator с локами
                            // loop {
                            //     for ticker in &stream_command_ok.tickers {
                            //         // Генерируем нужную котировку
                            //         if let Some(quote) = quotes_generator_clone.generate_quote(ticker) {
                            //             let data = quote.to_string() + "\n";
                            //             // Посылаем на адрес из команды
                            //             if let Err(e) = server_udp_socket.send_to(data.as_bytes(), stream_command_ok.client_addr_port) {
                            //                 eprintln!("❌ Ошибка отправки {}: {}", stream_command_ok.client_addr_port, e);
                            //                 // Ошибка при посылке
                            //                 break;
                            //             }
                            //             println!("📤 Отправлено: {} → {}", quote.ticker, stream_command_ok.client_addr_port);
                            //         }
                            //     }

                            //      thread::sleep(Duration::from_secs(10));
                            // }
                        });
                    },
                    Err(e) => {
                        response = e;
                    }
                }
                
                // Отправляем ответ и снова показываем промпт
                let _ = to_client_stream.write_all(response.as_bytes());
                let _ = to_client_stream.flush();
            }
            Err(_) => {
                // ошибка чтения — закрываем
                return;
            }
        }
    }
}


/*** Секция команды STREAM ***/

pub struct StreamCommand {
    pub client_addr_port: SocketAddr,
    pub tickers: Vec<String>,
}

impl StreamCommand {
    pub fn parse(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        // Проверяю тут жёстко на конкретную команду (можно отдельно, если буду добавлять команды)
        if parts.len() != 3 || parts[0].to_uppercase() != "STREAM" {
            return Err("ERROR: Неверный формат команды: STREAM udp://host:port TICKER1,TICKER2\n>".into());
        }
        
        // Отрезаем префикс
        let addr_str_no_prefix = parts[1].strip_prefix("udp://")
            .ok_or("ERROR: Принимаем только UDP: в адресе нет префикса udp://\n>")?;

        // Парсим адрес с портом через parse<SocketAddr>
        let target_addr = addr_str_no_prefix.parse::<SocketAddr>()
            .map_err(|e| format!("Адрес не распарсился {}: {}\n>", addr_str_no_prefix, e))?;
        
        // Разделяем компании, засовываем в вектор (совсем путые тикеры тоже убираем)
        let tickers: Vec<String> = parts[2].split(',')
            .map(|t| t.to_uppercase())
            .filter(|t| !t.is_empty())
            .collect();
        
        if tickers.is_empty() {
            return Err("ERROR: Не указано ни одной компании\n>".into());
        }
        
        Ok(Self{client_addr_port: target_addr, tickers})
    }
}