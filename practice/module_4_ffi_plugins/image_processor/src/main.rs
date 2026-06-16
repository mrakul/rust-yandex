mod error;
use error::ProcessingError;
use clap::Parser;
use std::path::PathBuf;
use image::{ImageBuffer, RgbaImage};

// cargo run -p image_processor
// cargo run -p image_processor -- --input ./aux/random_png_1033x775.png --output ./aux/test_output.png --plugin mirror --params ./aux/dummy_params.txt

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
    plugin_path: PathBuf
}

// Обработка плагином
fn process_image_with_plugin( input_path: &PathBuf, output_path: &PathBuf,
                            // TODO:
                            _plugin_name: &str, _plugin_path: &PathBuf,
                            _params_path: &PathBuf) -> Result<(), ProcessingError> {
    
    println!("Загрузка изображения: {}", input_path.display());

    // Загружаем с помощью image::open
    let image_loaded = image::open(input_path)
        .map_err(ProcessingError::LoadImage)?;

    // Конвертер в rgba8 (image крат) - все по байту, a - alpha (прозрачность)
    // (тип: ImageBuffer<image::Rgba<u8>, Vec<u8>>)
    let rgba_image = image_loaded.to_rgba8();

    // Тюпл ширина / высота
    let (width, height) = rgba_image.dimensions();
    println!("Размер изображения: {}x{}", width, height);

    // Перевод в сырой вектор: Vec<u8>, размер должен быть width * height * 4
    let rgba_buffer: Vec<u8> = rgba_image.into_raw();
    println!("Размер буфера: {} байт", rgba_buffer.len());

    // TODO: обработка плагином

    // Перевод обратно в RgbaImage (ImageBuffer::from_vec(...) возвращает RgbaImage)
    // Выдаст ошибку: 
    // let processed_image: RgbaImage = ImageBuffer::from_vec(10000, 10000, rgba_buffer)
    let processed_image: RgbaImage = ImageBuffer::from_vec(width, height, rgba_buffer)
        .ok_or_else(|| {image::ImageError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData,
                            "Размеры не соответствуют размеру буфера"))
    }).map_err(ProcessingError::ProcessImage)?;


    // Сохраняем обработанную картинку
    println!("Обработанное изображение: {}", output_path.display());
    processed_image.save(output_path)
        .map_err(ProcessingError::SaveImage)?;

    Ok(())
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

    // Обработка запрошенным плагином
    match process_image_with_plugin(&args.input, &args.output, &args.plugin, &args.plugin_path, &args.params) {
        Ok(()) => {
            println!("Изображение успешно обработано плагином: {}", args.plugin);
        }
        Err(error) => {
            eprintln!("Ошибка обработки: {}", error);
            // std::process::exit(1);
        }
    }


}