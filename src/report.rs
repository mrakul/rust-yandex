use std::io::{BufRead, BufReader};

use crate::csv_format::CsvFormatIO;
use crate::text_format::TextFormatIO;
use crate::bin_format::BinFormatIO;

use crate::transaction::{Transaction, TransactionStatus, TransactionType};
use crate::error::ParserError;

// Хранение отчёта в виде вектора транзакций
#[derive(Debug)]
pub struct Report {
    // Примечание: начал с HashMap, но, судя по всему, для поставленных задач проекта
    // чтения/записи/сравнения подходит больше вектор
    // transactions: HashMap<ID, Transaction>

    transactions: Vec<Transaction>
}

impl Report {
    // Конструктор, возвращем структуру Report (by value, Move-семантика)
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
        }
    }
    
    fn add_transaction(&mut self, tx_to_add: Transaction) -> () {
        self.transactions.push(tx_to_add)
    }


    /*** Реализации для HashMap, если понадобятся ***/
    // Добавить транзакцию: важно, что передаём с передачей владения для .insert()
    // С одним проходом => .entry()
    // pub fn add_transaction(&mut self, tx_to_add: Transaction) -> Option<&Transaction> {
    //     match self.transactions.entry(tx_to_add.tx_id) {
    //         // TODO: маловероятно, можно обработать
    //         std::collections::hash_map::Entry::Occupied(occupied_entry) => {
    //             Some(occupied_entry.get_mut())
    //         },
    //         std::collections::hash_map::Entry::Vacant(entry) => {
    //             // let added_transaction = Transaction::new();     
    //             let added_transaction = entry.insert(tx_to_add);
    //             Some(added_transaction)
    //             // Не очень, что копируем: можно пересмотреть API: возвращать bool или Option<&Transaction>
    //             // Entry::Vacant(entry) => Some(entry.insert(Balance::new()))
    //         }
    //     }
    // }

    // Удалить транзакцию (ДЛЯ HASHMAP): Option<i64> => "забираем" данные (Copy для примитивов)
    // pub fn remove_transaction(&mut self, tx_id_to_remove: &ID) -> Option<Transaction> {
    //     self.transactions.remove(tx_id_to_remove)
    // }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn get_transactions_mut(&mut self) -> &mut Vec<Transaction> {
        &mut self.transactions
    }

    pub fn compare_full(&self, report_to_compare: &Report) -> Result<(), ParserError> {
        if self.transactions.len() != report_to_compare.transactions.len() {
            return Err(ParserError::ReportLengthsAreNotEqual(self.transactions.len(), report_to_compare.transactions.len()))
        }

        let mut compared_iter = report_to_compare.transactions.iter();

        for source_tx in self.transactions.iter() {
            if let Some(compared_tx) = compared_iter.next() {
                if source_tx != compared_tx {
                    // Переводим в текстовое представление для ошибок
                    let cur_tx_str = source_tx.as_str();
                    let compared_tx_str = compared_tx.as_str();

                    return Err(ParserError::NonEqualTransactionFound(cur_tx_str, compared_tx_str));
                }
            }
            // В теории не должно быть, длины проверены
            else {
                // Пока так
                unreachable!()
            }
        }

        Ok(())
    }

    pub fn compare_streaming(&self, _report_to_compare: &Report) -> Result<(), ParserError> {
        todo!()
    }

}


/// Реализация трейта для парсинга из CSV-формата в Report и обратно
impl CsvFormatIO<Report> for Report {
    /// Получение структуры Report из ввода СSV-формата
    /// Аргументы: <R: std::io::Read>(reader: R)
    /// Результат: Result<Report, String>
    /// Использование:
    ///     let mut report = Report::new_from_text_file(&mut file_to_read)
    ///         {...}
    fn new_from_csv_reader<R: std::io::BufRead>(mut reader: R) -> Result<Report, ParserError> {
        // Можно весь прочитать: match reader.read_to_string(&mut buffer) ...
        // let mut buf_reader = BufReader::new(reader);

        // Первую строку надо пропустить (и можно обработать)
        let mut header_line = String::new();
        let _bytes_read = reader.read_line(&mut header_line)
            .map_err(|_| ParserError::CsvLineReadError)?;

        // // Создаём итератор для пропуска header'а - первой строки 
        // // let mut lines = buf_reader.lines();
        // // let _header = lines.next();

        // Создаём новый Report и читаем файл построчно
        let mut report = Self::new();
        
        // Идём по списку, читая по одному
        loop {
            match Transaction::new_from_csv_reader(&mut reader) {
                Ok(new_transaction) => {
                    report.add_transaction(new_transaction);
                },
                Err(ParserError::EOFEncountered) => {
                    // EOF => заканчиваем чтение
                    // TODO: подумать для Streaming'а
                    break;
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
        
        Ok(report)
    }

    /// Перевод структуры Report в СSV-формат
    /// Аргументы: <W: std::io::Write>(&mut self, writer: &mut W)
    /// Результат: Result<(), ParserError>
    /// Использование:
    ///     let mut report = Report::new_from_text_reader(&mut file_to_read)
    ///         {...}
    fn write_as_csv_to_writer<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), ParserError> {

        // Бежим по вектору (по мутабельным ссылкам)
        for cur_tx in self.transactions.iter_mut() {
            cur_tx.write_as_csv_to_writer(writer)?;
        }

        Ok(())
    }
}


/// Реализация трейта для парсинга из Bin-формата в Report и обратно
impl BinFormatIO<Report> for Report {
    fn new_from_bin_reader<R: std::io::Read>(mut reader: R) -> Result<Report, ParserError> {
        let mut report = Self::new();
        
        // Идём по списку, читая по одному
        loop {
            match Transaction::new_from_bin_reader(&mut reader) {
                Ok(new_transaction) => {
                    report.add_transaction(new_transaction);
                },
                Err(ParserError::EOFEncountered) => {
                    // EOF => заканчиваем чтение
                    // TODO: подумать для Streaming'а
                    break;
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
        
        Ok(report)
    }

    fn write_as_bin_to_writer<W: std::io::Write>(&mut self, mut writer: &mut W) -> Result<(), ParserError> {

        for cur_tx in self.transactions.iter_mut() {
            // Теперь пишется транзакцией 
            cur_tx.write_as_bin_to_writer(&mut writer)?;

            // Запись каждой транзакции в случае буферизированного вывода
            // writer.flush()
            //     .map_err(|e| format!("Не удалось транзакцию: {:?} => {}", transaction, e))?;
        }
        
        // Сразу все транзакции (или пока буфер не заполнится?)
        // writer.flush()
        //     .map_err(|_| ParserError::BinTxWriteError)?;
        
        Ok(())
    }
}

/// Реализация трейта для парсинга из текстового формата в Report и обратно
impl TextFormatIO<Report> for Report {
    fn new_from_text_reader<R: std::io::Read>(mut reader:  R) -> Result<Report, ParserError> {
        // Чтение всего
        // match reader.read_to_string(&mut buffer) {

        let mut report = Self::new();
        
        // Идём по списку, читая по одному
        loop {
            match Transaction::new_from_text_reader(&mut reader) {
                Ok(new_transaction) => {
                    report.add_transaction(new_transaction);
                },
                Err(ParserError::EOFEncountered) => {
                    // EOF => заканчиваем чтение
                    // TODO: подумать для Streaming'а
                    break;
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
        
        Ok(report)
    }

    fn write_as_text_to_writer<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), ParserError> {
        // Бежим по вектору (по ссылке)
        for cur_tx in self.transactions.iter_mut() {
            cur_tx.write_as_text_to_writer(writer)?;
        }

        Ok(())
    }
}

//*** Секция тестов для Report ***/

// Child-модуль
// #[cfg(test)]
// mod tests {
//     // Подключаем всё из родительского модуля (использование методов/полей)
//     use super::*; 

//     #[test]
//     fn test_new_storage_is_empty() {
//         let bank = Report::new();
//         // При создании нет пользователей
//         assert_eq!(bank.transactions.len(), 0);
//     }

//     #[test]
//     fn test_add_user() {
//         let mut storage = Report::new();

//         // Проверка уже существующего пользователя
//         assert_eq!(storage.add_transaction("Alice".to_string()), Some(Balance::from(0))); // новый пользователь
//         assert_eq!(storage.add_transaction("Alice".to_string()), None);    // уже существует
//     }

//     #[test]
//     fn test_remove_user() {
//         let mut storage = Report::new();

//         storage.add_transaction("Bob".to_string());
//         storage.deposit(&"Bob".to_string(), 100).unwrap();

//         // Проверка баланса до и после удаления пользователя
//         assert_eq!(storage.remove_transaction(&"Bob".to_string()), Some(Balance::from(100)));
//         assert_eq!(storage.remove_transaction(&"Bob".to_string()), None); 
//     }

//     #[test]
//     fn test_deposit_and_withdraw() {
//         let mut storage = Report::new();
//         storage.add_transaction("Charlie".to_string());

//         // Пополнение
//         assert!(storage.deposit(&"Charlie".to_string(), 200).is_ok());
//         assert_eq!(storage.get_balance(&"Charlie".to_string()), Some(&Balance::from(200)));

//         // Успешное снятие
//         assert!(storage.withdraw(&"Charlie".to_string(), 150).is_ok());
//         assert_eq!(storage.get_balance(&"Charlie".to_string()), Some(&Balance::from(50)));

//         // Ошибка: недостаточно средств
//         assert!(storage.withdraw(&"Charlie".to_string(), 100).is_err());
//         // Но 50 ещё имеется
//         assert_eq!(storage.get_balance(&"Charlie".to_string()), Some(&Balance::from(50)));
//     }

//     #[test]
//     fn test_nonexistent_user() {
//         let mut storage = Report::new();

//         // Депозит несуществующему пользователю
//         assert!(storage.deposit(&"Dana".to_string(), 100).is_err());

//         // Снятие у несуществующего пользователя
//         assert!(storage.withdraw(&"Dana".to_string(), 50).is_err());

//         // Баланс у несуществующего пользователя
//         assert_eq!(storage.get_balance(&"Dana".to_string()), None);
//     }

//     use std::fs::File;
//     use std::io::Write;


//     #[test]
//     fn test_load_data_existing_file() {
//         let file_path = String::from("unit_test_file.csv");

//         // Важно сделать mut file
//         // Не нужно делать open после create (?)
//         if let Ok(mut file) = File::create(&file_path) {
//             writeln!(file, "John,100").unwrap();
//             writeln!(file, "Alice,200").unwrap();
//             writeln!(file, "Bob,50").unwrap();
//             // Здесь закроется, отрытый на чтение (?)
//             // .unwrap() нужен, судя по всему, только чтобы убрать unused warning'и
//         }
//         else {
//             panic!("Файл не создан!");
//         }

//         let storage = Report::load_file_to_storage(&file_path.as_str());     

//         assert_eq!(storage.get_balance(&"John".to_string()), Some(&Balance::from(100)));
//         assert_eq!(storage.get_balance(&"Alice".to_string()), Some(&Balance::from(200)));
//         assert_eq!(storage.get_balance(&"Bob".to_string()), Some(&Balance::from(50)));
//         // Пользователь Vasya не добавлен в файле, поэтому None
//         assert_eq!(storage.get_balance(&"Vasya".to_string()), None);

//         // Удаляем тестовый файл
//         fs::remove_file(file_path).unwrap();
//     }

//     #[test]
//     fn test_save_creates_file_with_correct_data() {
//        let file_path = "test_save.csv";

//         // Создаём Storage и добавляем пользователей
//         let mut storage = Report::new();
//         storage.add_transaction("John".to_string());
//         storage.add_transaction("Alice".to_string());
//         storage.deposit(&"John".to_string(), 150).unwrap();
//         storage.deposit(&"Alice".to_string(), 300).unwrap();

//         // Сохраняем в файл
//         storage.save_storage_to_file(file_path);

//         // Читаем файл обратно и проверяем содержимое

//         // Это читает полнстью в String, можно как сделано: let reader = BufReader::new(file);

//         // Фактически, открывает open() и читает read() одновременно
//         let contents = fs::read_to_string(file_path).unwrap();

//         let mut lines: Vec<&str> = contents.lines().collect();
//         // Сортируем, так как get_all() может возвращать в любом порядке, для проверки в тесте
//         lines.sort(); 

//         assert_eq!(lines, vec!["Alice,300", "John,150"]);

//         // Удаляем тестовый файл
//         fs::remove_file(file_path).unwrap();
//     }

//         #[test]
//     fn test_bufwriter() {
//         use std::fs::File;
//         use std::io::{BufWriter, Write};

//         let file_path = "./aux/data.csv";

//         let f = File::create(file_path).unwrap();
//         let mut writer = BufWriter::new(f);

//         writeln!(writer, "John,100").unwrap();   // пока в буфере
//         writeln!(writer, "Alice,200").unwrap();  // пока в буфере

//         let mut contents = fs::read_to_string(file_path).unwrap();
        
//         assert_eq!(contents.len(), 0);

//         // Записываем (flush the buffer), читаем ещё раз и проверяем длину (грубовато)
//         writer.flush().unwrap(); 
//         contents = fs::read_to_string(file_path).unwrap();

//         assert_eq!(contents.len(), 19);

//         // Удаляем файл
//         fs::remove_file(file_path).unwrap();
//     }

//     use std::io::{Cursor, BufWriter};

//     #[test]
//     fn test_load_data_existing_cursor() {
        
//         // (!) Создаём данные в памяти, как будто это CSV-файл
//         let data = b"John,100\nAlice,200\nBob,50\n";
//         let mut cursor = Cursor::new(&data[..]);

//         // Читаем данные из Cursor
//         let mut storage = Report::new();
//         let reader = BufReader::new(&mut cursor);

//         // То же самое, но читаем из буфера в памяти с помощью Cursor
//         for line in reader.lines() {
//             let line = line.unwrap();
//             let columns: Vec<&str> = line.trim().split(',').collect();
    
//             if columns.len() == 2 {
//                 let name = columns[0].to_string();
//                 let balance: i64 = columns[1].parse().unwrap_or(0);
//                 storage.add_transaction(name.clone());
//                 storage.deposit(&name, balance).unwrap();
//             }
//         }

//         assert_eq!(storage.get_balance(&"John".to_string()), Some(&Balance::from(100)));
//         assert_eq!(storage.get_balance(&"Alice".to_string()), Some(&Balance::from(200)));
//         assert_eq!(storage.get_balance(&"Bob".to_string()), Some(&Balance::from(50)));
//         assert_eq!(storage.get_balance(&"Vasya".to_string()), None);
//     }

//     #[test]
//     fn test_save_writes_to_cursor_correctly() {
//         // Создаём Storage и добавляем пользователей
//         let mut storage = Report::new();
//         storage.add_transaction("John".to_string());
//         storage.add_transaction("Alice".to_string());
//         storage.deposit(&"John".to_string(), 150).unwrap();
//         storage.deposit(&"Alice".to_string(), 300).unwrap();

//         // Сохраняем в память через BufWriter

//         // (!) и при чтении, и при записи используется Vec<u8>
//         let buffer = Vec::new();

//         // Буфер оборачивается в Cursor
//         let mut cursor = Cursor::new(buffer);
//         {
//             // Cursor оборачивается в BufWriter
//             let mut writer = BufWriter::new(&mut cursor);
//             // Получение всех записей в векторе запись в буфер как в файл
//             for (name, balance) in storage.get_all() {
//                 writeln!(writer, "{},{}", name, balance.get_value()).unwrap();
//             }
            
//             writer.flush().unwrap();
//         }

//         // Читаем обратно из памяти - обязательно нужно сбросить позицию в 0
//         cursor.set_position(0);

//         let mut lines: Vec<String> = BufReader::new(cursor).lines().map(|l| l.unwrap()).collect();
//         lines.sort(); // сортируем для сравнения

//         assert_eq!(lines, vec!["Alice,300", "John,150"]);
//     }

// }