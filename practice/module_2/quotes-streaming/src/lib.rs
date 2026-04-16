pub mod quotes;

// TCP-сервер работает на 11000 порту
pub const SERVER_ADDR: &str = "127.0.0.1:11000";

// PING раз в 2 секунды
pub const PING_INTERVAL_SECS: u64 = 3;

// Таймаут по PING'у - 5 секунд
pub const PING_TIMEOUT_MILLISECS: u64 = 5000;