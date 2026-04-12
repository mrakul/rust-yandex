// Простой клиент, без наворотов :)

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    // Подключаемся и клонируем stream для reader'а
    let to_server_stream = TcpStream::connect("127.0.0.1:11000")?;
    let mut to_client_stream = BufReader::new(to_server_stream.try_clone()?);

    // Читаем приветствие
    let mut buffer = [0; 1024];
    // Читаем доступные данные один раз
    let read_count = to_client_stream.read(&mut buffer)?;
    let starting_string = String::from_utf8_lossy(&buffer[..read_count]);
    print!("{}", starting_string);

    let stdin = io::stdin();
    
    // Когда подключились, тут бесконечный цикл
    loop {
        print!("> ");

        io::stdout().flush().unwrap();

        // Читаем команду из консоли в строку
        let mut input_line = String::new();
        stdin.read_line(&mut input_line)?; 
        
        //     return ConnectionResult::Lost;
        // }

        let trimmed_input = input_line.trim();
        // if trimmed_input.is_empty() {
        //     continue;
        // }

        // // EXIT — выходим из клиента
        // if trimmed.eq_ignore_ascii_case("EXIT") {
        //     println!("Bye!");
        //     return ConnectionResult::Exit;
        // }

        // // PING — измеряем задержку
        // if trimmed.eq_ignore_ascii_case("PING") {
        //     match send_ping(&stream, &mut reader) {
        //         Ok(latency) => println!("PONG (latency: {}ms)", latency),
        //         Err(e) => {
        //             println!("ERROR: server unreachable ({})", e);
        //             return ConnectionResult::Lost;
        //         }
        //     }
        //     continue;
        // }

        // любые другие команды
        match send_command(&to_server_stream, &mut to_client_stream, trimmed_input) {
            Ok(response) => print!("{}", response),
            Err(e) => {
                // TODO: 
                println!("ERROR: Сервер закрыл соединение ({})", e);
            }
        }
    }

    Ok(())
}


fn send_command(
    mut stream: &TcpStream,
    reader: &mut BufReader<TcpStream>,
    command: &str,
) -> io::Result<String> 
{
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut read_buffer = String::new();
    let bytes_read_cnt = reader.read_line(&mut read_buffer)?;

    if bytes_read_cnt == 0 {
        // сервер закрыл соединение
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                                 "Cервер закрыл соединение",
        ));
    }

    Ok(read_buffer)
}
