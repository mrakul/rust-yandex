2. Проверка системных зависимостей:

// build.rs
fn main() {
    if cfg!(target_os = "linux") {
        // Линкуемся с системной либой
        println!("cargo:rustc-link-lib=ssl");
    }
} 

3. Условная компиляция:

// build.rs
fn main() {
    if let Ok(ver) = std::env::var("RUSTC_VERSION") {
        println!("cargo:rustc-env=RUSTC_VERSION={}", ver);
    }

    if std::env::var("ENABLE_FEATURE").is_ok() {
        println!("cargo:rustc-cfg=feature=\"enabled\"");
    }
} 

