fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Нужно добавить в build-dependencies tonic_build
    tonic_build::configure()
        .build_server(false)    // Клиенту не нужны серверные trait'ы
        .build_client(true)     // Только клиентский код
        .compile(
            &["proto/blog.proto"],
            &["proto"],
        )?;
    
    println!("cargo:rerun-if-changed=proto/blog.proto");
    Ok(())
}