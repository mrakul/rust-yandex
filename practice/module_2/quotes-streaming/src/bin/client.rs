// Запуск клиента, пример:
// > cargo run --bin client -- --server-addr-port 127.0.0.1:11000 --udp-client-port 30000 --subscriptions-file aux/client_1_tickers.txt
//
//  --server-addr-port <SERVER_ADDR_PORT>
//  --udp-client-port <UDP_CLIENT_PORT>
//  --subscriptions-file <SUBSCRIPTIONS_FILE>

// Пускануть два клиента со своими подписками:
// cargo run --bin client -- --server-addr-port 127.0.0.1:11000 --udp-client-port 30000 --subscriptions-file aux/client_1_tickers.txt
// cargo run --bin client -- --server-addr-port 127.0.0.1:11000 --udp-client-port 30002 --subscriptions-file aux/client_2_tickers.txt

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpStream, UdpSocket, SocketAddr};
use std::fs::File;
use std::thread::JoinHandle;
use std::time::Duration;
use clap::{Parser};

#[derive(Parser, Debug)]
#[command(name = "Клиент получения котировок с заданием нужных котировок")]
struct Args {
    #[arg(long)]
    server_addr_port: String,

    #[arg(long)]
    udp_client_port: String,

    #[arg(long)]
    subscriptions_file: String,

}

fn main() -> io::Result<()> {
    // Читаем аргументы командной строки, ошибки формата обрабатываются в parse()
    let args = Args::parse();

    let subscriptions_open = File::open(args.subscriptions_file)?;
    let reader_subs = BufReader::new(subscriptions_open);
    let mut subscriptions= Vec::<String>::new();

    // Заполняем подписки
    for line in reader_subs.lines() {
        let line = line?;
        let cur_ticker = line.trim();
        
        subscriptions.push(cur_ticker.to_string());
    }
    
    println!("Запрошенные котировки: {:?}", subscriptions);

    // Парсим адрес
    let server_addr_port= match args.server_addr_port.parse::<SocketAddr>() {
        Ok(parsed_addr) => parsed_addr,
        Err(error) => {
            eprintln!("Адрес не распарсился '{}': {}", args.server_addr_port, error);
            std::process::exit(1);
        }
        // Можно .unwrap_or_else()
    };

    // Парсим UDP-порт, куда будут приходит котировки
    let udp_client_port: u16 = match args.udp_client_port.parse::<u16>() {
        Ok(parsed_port) => parsed_port,
        Err(error) => {
            eprintln!("Порт не распарсился '{}': {}", args.udp_client_port, error);
            std::process::exit(1);
        }
    };
    
    // Подключаемся и клонируем stream для reader'а
    println!("Подключаемся к {}", server_addr_port);

    let to_server_stream = TcpStream::connect(server_addr_port)?;
    let mut to_client_stream = BufReader::new(to_server_stream.try_clone()?);

    // Читаем приветствие
    let mut buffer = [0; 1024];
    let read_count = to_client_stream.read(&mut buffer)?;
    let starting_string = String::from_utf8_lossy(&buffer[..read_count]);
    print!("{}", starting_string);
    io::stdout().flush().unwrap();

    // Посылаем команду STREAM: STREAM udp://127.0.0.1:30000 [тикеры из файла]
    let stream_request_command = format!("STREAM udp://127.0.0.1:{} {}", udp_client_port, subscriptions.join(","));
    println!("Посылаемая команда: {}", stream_request_command);

    // Создание сокета UDP для стриминга
    let client_udp_socket = UdpSocket::bind(format!("127.0.0.1:{}", udp_client_port))
        .unwrap_or_else(|error| {
            eprintln!("Не удалось получить сокет для UDP-приёма котировок на порт {}: {}", format!("127.0.0.1:{}", udp_client_port), error);
            std::process::exit(1);
        });

    // Handle'ы для потоков receiver'а, ping'ера
    let mut udp_receiver_join_handle_opt: Option<JoinHandle<()>> = None;
    let mut udp_ping_join_handle_opt: Option<JoinHandle<()>> = None;

    match send_command(&to_server_stream, &mut to_client_stream, &stream_request_command) {
        Ok(response) => {
            // Парсинг ответа
            print!("{}", response);
            // Разбираем команду
            let reponse_tokens: Vec<&str> = response.split_whitespace().collect();

            if reponse_tokens.first() == Some(&"OK:") {
                // Поток-приёмник котировок
                udp_receiver_join_handle_opt = Some(launch_udp_receiver(client_udp_socket.try_clone().unwrap()));
                
                // Поток-пингователь
                let mut udp_ping_server_addr = server_addr_port.clone();
                udp_ping_server_addr.set_port(udp_client_port + 1);
                udp_ping_join_handle_opt = Some(launch_udp_ping(client_udp_socket, udp_ping_server_addr));
            }
            else if reponse_tokens.first() == Some(&"ERROR:") {
                println!("Произошла ошибка при подключении к серверу: {}", response);
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("ERROR: Сервер закрыл соединение ({})", e);
        }
    }

    // Ждём завершение, если был запущен
    if let Some(udp_receiver_join_handle) = udp_receiver_join_handle_opt  {
        // TODO: да, обработка ошибок
        udp_receiver_join_handle.join();
    }

    if let Some(udp_ping_join_handle) = udp_ping_join_handle_opt  {
        // TODO: да, обработка ошибок
        udp_ping_join_handle.join();
    }

    Ok(())

}

// Отправка команды по TCP-стриму и получение response в BufReader
fn send_command(
    mut stream: &TcpStream,
    reader: &mut BufReader<TcpStream>,
    command: &String,
) -> io::Result<String> 
{
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut read_buffer = String::new();
    let bytes_read_cnt = reader.read_line(&mut read_buffer)?;

    if bytes_read_cnt == 0 {
        // Cервер закрыл соединение
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                                 "Cервер закрыл соединение",
        ));
    }

    Ok(read_buffer)
}

fn _append_tickers(command: &mut String, tickers_vec: Vec<String>) {
    for cur_ticker in &tickers_vec {
        command.push_str(cur_ticker); 
    }
}

// Запуск UDP-ресивера
fn launch_udp_receiver(udp_socket: UdpSocket) -> JoinHandle<()> {
        // Поток UDP-стриминга под нового клиента 
    let udp_receiver_join_handle = std::thread::spawn(move || {
            // Читаем из каналов
            let mut buffer = [0u8; 1024];
    
            loop {
                match udp_socket.recv_from(&mut buffer) {
                    Ok((bytes_read, sender_addr)) => {
                        // Ожидаем символы UTF-8
                        match std::str::from_utf8(&buffer[..bytes_read]) {
                            Ok(message) => {
                                println!("🚀 Получено от {}: {}", sender_addr, message.trim());
                            }
                            Err(error) => eprintln!("Не удалось декодировать UTF-8: {}", error),
                        }
                    }
                    Err(error) => {
                        eprintln!("Ошибка приёма UDP-датаграммы: {}", error);
                        // Обработка в зависимости от ошибки
                        continue;
                    }
                }

                std::thread::sleep(Duration::from_millis(500));
            }
    });

    return udp_receiver_join_handle;
}

// Запуск UDP-пингера
fn launch_udp_ping(udp_socket: UdpSocket, ping_to_addr: SocketAddr) -> JoinHandle<()> {
    // Поток UDP-пинг 
    let udp_ping_join_handle = std::thread::spawn(move || {
        // Сообщение PING, с newline
        let ping_message = b"PING\n";
        
        loop {
            // Отправляем PING на сервер
            match udp_socket.send_to(ping_message, ping_to_addr) {
                Ok(bytes_sent) => {
                println!("Пинг отправлен ({} байт) PING на {}", bytes_sent, ping_to_addr);
                }
                Err(e) => {
                    eprintln!("Ошибка отправки PING: {}", e);
                }
            }
        
            // Ждём 2 секунды перед следующим пингом (как в задании)
            std::thread::sleep(Duration::from_secs(2));
        }
    });

    return udp_ping_join_handle;
}

