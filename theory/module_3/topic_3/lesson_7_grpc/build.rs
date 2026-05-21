fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        // Здесь не объяснено, что здесь билдятся клиент и сервер с учётом схемы protobuf
        // (не файлы clien.rs/server.rs, а подкапотная часть по обмену, как я понял)
        .build_server(true)   // ← Generate server trait + stub
        .build_client(true)   // ← Generate client struct + methods
        .compile(
            &["proto/exchange.proto"],
            &["proto"],
        )?;
    Ok(())
}

