fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)  // Генерируем trait для сервера
        .build_client(true)  // Генерируем клиент (опционально, но полезно для тестов, вроде бы)
        .compile(
            &["proto/blog.proto"],
            &["proto"],
        )?;
    
    // Автоматическая пересборка при изменении proto-файла
    println!("cargo:rerun-if-changed=proto/blog.proto");
    Ok(())
}