//! # Библиотека parser
//! 
//! Библиотека для парсинга отчёта по финансовым операциям из форматов CSV, текстовый, бинарный
//! и перевода в любой их указанных 
//! 
//! ### Структуры данных
//! - [`struct Report`] - структура внутреннего представления коллекции транзакций, 
//!                       используемая для перевода из/в указанные форматы
//! 
//! ### Пример использования
//! 
//! ```rust
//! use parser::report::Report;
//! use parser::csv_format::CsvFormatIO;
//! use parser::bin_format::BinFormatIO;
//! use parser::text_format::TextFormatIO;
//! use std::fs::File;
//! use std::path::Path;
//! 
//! // Чтение из CSV-файла (или другого источника, релизующего трейт Read)
//!    let file_to_read = Path::new("aux/records_example.csv");
//! 
//!    let mut file_to_read = File::open(file_to_read)
//!        .unwrap_or_else(|e| {
//!            eprintln!("Не удалось открыть файл: {}", e);
//!            std::process::exit(1);
//!        });
//!
//!    let mut report = Report::new_from_csv_file(&mut file_to_read)
//!        .unwrap_or_else(|e| {
//!            eprintln!("СSV не прочитан: {}", e);
//!            std::process::exit(1);
//!        });
//! 
//! // Запись обратно в CSV (трейт Write)
//!    let csv_file_to_write_path = Path::new("aux/output.csv");
//!
//!    let mut csv_file_to_write = File::create(csv_file_to_write_path)
//!        .unwrap_or_else(|e| {
//!            eprintln!("Не удалось создать файл: {}", e);
//!            std::process::exit(1);
//!        });
//!
//!    match report.write_to_csv_file(&mut csv_file_to_write) {
//!        Ok(()) => println!("Записано в файл: {:?}", csv_file_to_write_path),
//!        Err(error) => println!("Ошибка записи в файл {:?}: {}", csv_file_to_write_path, error),
//!    }
//! ```
//! Используемые функции:
//! - `Report::new_from_csv_file<R: std::io::Read>(reader: R) -> Result<InternalType, String>` 
//! - `Report::new_from_text_file<R: std::io::Read>(reader: R) -> Result<InternalType, String>`
//! - `Report::new_from_bin_file<R: std::io::Read>(reader: R) -> Result<InternalType, String>`
//! - `report.write_to_csv_file<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), String>`
//! - `report.write_to_text_file<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), String>`
//! - `report.write_to_bin_file<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), String>`
//!
//! Для подробного описания функций см.модуль report.rs
//! 
pub mod report;
pub mod csv_format;
pub mod bin_format;
pub mod text_format;
pub mod error;
// Пока намеренно скрыто
mod transaction;