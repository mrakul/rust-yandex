pub mod client;

pub mod exchange {
    // Мой комментарий:
    // Это макро загружает сгенерированный код из OUT_DIR на этапе компиляции
    tonic::include_proto!("exchange");  // ← "exchange" = название 
}

