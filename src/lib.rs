//! # Библиотека parser
//! 
//! Библиотека для парсинга отчёта по финансовым операциям из форматов CSV, текстовый, бинарный
//! и перевода в любой их указанных 
//! 
//! ### Структуры данных
//! - [`struct Report`] - структура внутреннего представления коллекции транзакций, 
//!                       используемая для перевода из/в указанные форматы
//! 
//! ### Использование
//! 
//! ```rust
//! use parser::{Report, CsvFormatIO};
//! use std::fs::File;
//! 
//! // Чтение из CSV-файла (или другого источника, релизующего трейт Read)
//! let mut file = File::open("transactions.csv")?;
//! let report = Report::new_from_csv_file(file)?;
//! //! 
//! // Запись обратно в CSV (трейт Write)
//! let mut output_file = File::create("output.csv")?;
//! report.write_to_csv_file(&mut output_file)?;
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
// Пока намеренно скрыто
mod transaction;