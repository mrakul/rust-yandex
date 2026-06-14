use clap::Parser;
use std::path::PathBuf;

// cargo run -p image_processor
// cargo run -p image_processor -- --input dummy_input.png --output dummy_output.png --plugin mirror --params dummy_params.txt

#[derive(Parser, Debug)]
#[command(name = "image_processor", about = "CLI для проверки применения плагинов к изображению")]
struct Args {
    /// Исходное изображение (путь)
    #[arg(short, long)]
    input: PathBuf,

    /// Путь сохранения обработанного изображения
    #[arg(short, long)]
    output: PathBuf,

    /// Имя плагина. Предполагаются: "mirror", "blur"
    #[arg(long)]
    plugin: String,

    /// Путь к файлу с параметрами
    #[arg(long)]
    params: PathBuf,

    /// Путь к директории с плагинами
    #[arg(long, default_value = "target/debug")]
    plugin_path: PathBuf,
}

fn main() {
    // С CLAP'ом всё стандартно
    let args = Args::parse();

    // Проверка наличия файла изображения
    if !args.input.exists() {
        eprintln!("Файл изображения '{}' не существует.", args.input.display());
        std::process::exit(1);
    }

// Проверка файла парааметров
    if !args.params.exists() {
        eprintln!("Файл параметров '{}' не существует", args.params.display());
        std::process::exit(1);
    }

    println!("Изображение:\t\t\t{}", args.input.display());
    println!("Обработанное изображение:\t{}", args.output.display());
    println!("Плагин обработал:\t\t{}", args.plugin);
    println!("Файл параметров:\t\t{}", args.params.display());
    println!("Путь к плагинам:\t\t{}", args.plugin_path.display());

    // TODO: основная логика
}