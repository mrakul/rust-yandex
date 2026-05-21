fn main() {
    // Этот код выполняется перед сборкой

    // Команды для Cargo:
    //  cargo:rerun-if-changed=<file> — пересобрать проект, если файл изменился.
    //  cargo:rerun-if-env-changed=<var> — пересборка при изменении env.
    //  cargo:warning=<msg> — вывести предупреждение при сборке.
    //  cargo:rustc-link-lib=<lib> — связать бинарь с системной библиотекой.
    //  cargo:rustc-env=<k=v> — установить env-переменную для компиляции.
    //  cargo:rustc-cfg=<flag> — включить условную компиляцию.

    // Показать предупреждение
    println!("cargo:warning=This is a build script warning");

    // Пересобрать при изменении файла
    println!("cargo:rerun-if-changed=src/config.toml");

    // Пересобрать при изменении переменной окружения
    println!("cargo:rerun-if-env-changed=DATABASE_URL");
} 