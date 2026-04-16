 # Стриминг котировок

Константы можно поменять в lib.rs

> #### TCP-сервер работает на 11000 порту 
> pub const SERVER_ADDR: &str = "127.0.0.1:11000";

> ####  PING раз в 2 секунды
> pub const PING_INTERVAL_SECS: u64 = 2;

> #### Таймаут по PING'у - 5 секунд
> pub const PING_TIMEOUT_MILLISECS: u64 = 5000;

 ## Здесь три приложения:
 
 ### server
 > cargo run --bin server
   - Запускается на порту 11000 по умолчанию
   - Запускает генератор котировок в отдельном потоке, инициализируется подписками из файла
   - При подключении ждёт команду вида STREAM udp://127.0.0.1:30000 AAPL,MSFT,GOOGL
   - При верной команде регистрирует клиента в генераторе с нужными котировками, сооздаёт поток UDP-стриминга и PING-обработчика,
   который обменивается с QuoteGenerator'ом через каналы 
   - Логгер по умолчанию включён в DEBUG-логировании

 ### client
 Запуск клиента, пример:
> cargo run --bin client -- --server-addr-port 127.0.0.1:11000 --udp-client-port 30000 --subscriptions-file aux/client_1_tickers.txt
'''
--server-addr-port <SERVER_ADDR_PORT>
--udp-client-port <UDP_CLIENT_PORT>
--subscriptions-file <SUBSCRIPTIONS_FILE>
'''

Пускануть два клиента со своими подписками:
> cargo run --bin client -- --server-addr-port 127.0.0.1:11000 --udp-client-port 30000 --subscriptions-file aux/client_1_tickers.txt\
> cargo run --bin client -- --server-addr-port 127.0.0.1:11000 --udp-client-port 30002 --subscriptions-file aux/client_2_tickers.txt

  ### client_cli
  Клиент для отладки. 
  Отправляет команды на сервер (подключается тоже на адрес сервера из lib.rs).
  После обработки команды продолжает работу, можно ввести другие команды, запустить несколько серверов.

  Но здесь не охвачена логика PING - он, скорее, для отладки работы сервера.