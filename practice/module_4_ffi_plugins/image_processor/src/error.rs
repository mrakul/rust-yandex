
#[derive(Debug)]
pub enum ProcessingError {
    LoadImage(image::ImageError),
    SaveImage(image::ImageError),
    ProcessImage(image::ImageError),
    FileNotFound(String),
    // Общая ошибка для плагина
    PluginError(String)
}

// Дисплей для вывода
impl std::fmt::Display for ProcessingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::LoadImage(image_error) => write!(formatter, "Ошибка загрузки изображения, image крат: {}", image_error),
            ProcessingError::SaveImage(image_error) => write!(formatter, "Ошибка сохранения изображения, image крат: {}", image_error),
            ProcessingError::ProcessImage(image_error) => write!(formatter, "Ошибка обработки изображения, image крат: {}", image_error),
            ProcessingError::FileNotFound(path) => write!(formatter, "Файл не найден: {}", path),
            ProcessingError::PluginError(plugin_error) => write!(formatter, "Ошибка плагина: {}", plugin_error)
        }
    }
}