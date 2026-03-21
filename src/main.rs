use parser::report::Report;
use parser::csv_format::CsvFormatIO;
use std::fs::File;
use std::path::Path;

fn main() {
    let file_path = Path::new("aux/records_example.csv");

    // Открываем файл
    let mut file = File::open(file_path)
        .unwrap_or_else(|e| {
            eprintln!("Не удалось открыть файл: {}", e);
            std::process::exit(1);
        });

    let report = Report::new_from_csv_file(&mut file)
        .unwrap_or_else(|e| {
            eprintln!("СSV не прочитан: {}", e);
            std::process::exit(1);
        });

    println!("Отчёт: {:?} размер {} ", report, report.get_transactions().len());

    // let report = Report::new_from_csv_file();
}