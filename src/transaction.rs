use std::fmt::Formatter;
use std::fmt;
use std::mem;

use crate::error::ParserError;
use crate::transaction::fmt::Display;

use crate::csv_format::CsvFormatIO;
use crate::text_format::TextFormatIO;
use crate::bin_format::BinFormatIO;

use std::io::ErrorKind;
use byteorder::{ByteOrder, BigEndian};
use clap::Parser;

use std::collections::HashMap;
use std::io::{BufReader, BufRead};

// Задаём тип (аналог using C++)
pub type ID = u64;

// Структура для чтения/записи транзакции
#[derive(Debug, PartialEq)]
pub struct Transaction {
    tx_id:          u64,
    tx_type:        TransactionType,
    from_user_id:   u64,
    to_user_id:     u64,
    amount:         u64,
    timestamp:      u64,
    status:         TransactionStatus,
    description:    String,
}

// Конструктор из значений
impl Transaction {
    pub fn new(tx_id:        u64,
               tx_type:      TransactionType,
               from_user_id: u64,
               to_user_id:   u64,
               amount:       u64,
               timestamp:    u64,
               status:       TransactionStatus,
               description:  String) -> Self 
            {
                Transaction {tx_id, tx_type, from_user_id, to_user_id, amount, timestamp, status, description}
            }
}

// Отдельная структура для представления транзакции в бинарном виде без alignment'ов
// (пока нужна только для size_of(), если будут добавляться поля)

// Header отдельно
#[repr(packed)]
#[derive(Debug)]
pub struct BinTransactionHeader {
    // 'YPBN' = [0x59, 0x50, 0x42, 0x4E]
    magic: [u8; 4],    
    // Прямой порядок байт?
    record_size: u32,
}
#[repr(packed)]
pub struct BinTransactionBodyFixed {
    tx_id: u64, 
    tx_type: u8,  
    from_user_id: u64,
    to_user_id: u64,
    // | `AMOUNT` | 8 байт | знаковое 64-битное | Сумма в наименьшей денежной единице (центах). 
    // Положительное значение для зачислений, отрицательное для списаний. |
    amount: i64,
    timestamp: u64,
    status: u8,
    desc_len: u32,
    // TODO: спросить про возможность иметь указатель (?) переменой aka Flexible array member
}


// В таком случае packed может не сработать, насколько понял, пока лучше не использовать
#[repr(packed)]  
pub struct BinaryTransaction {
    pub header: BinTransactionHeader,
    pub body:   BinTransactionBodyFixed,
}

impl BinaryTransaction {
    pub fn size_without_description() -> usize {
        mem::size_of::<BinTransactionHeader>() +
        mem::size_of::<BinTransactionBodyFixed>()
    }
}

// Тип транзакции
#[derive(Debug, PartialEq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer,
    Unknown
}

// Для вывода в виде строки
impl Display for TransactionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Deposit => write!(f, "DEPOSIT"),
            TransactionType::Withdrawal=> write!(f, "WITHDRAWAL"),
            TransactionType::Transfer=> write!(f, "TRANSFER"),
            TransactionType::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

// Статус транзакции
#[derive(Debug, PartialEq)]
pub enum TransactionStatus {
    Success,
    Failure,
    Pending,
    Unknown,
}

// Для вывода в виде строки
impl Display for TransactionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Success => write!(f, "SUCCESS"),
            TransactionStatus::Failure => write!(f, "FAILURE"),
            TransactionStatus::Pending => write!(f, "PENDING"),
            TransactionStatus::Unknown => write!(f, "UNKONWN"),
        }
    }
}

/*** Преобразования типа и статуса для бинарного формата ***/
impl TransactionType {
    pub fn from_u8(bin_value: u8) -> Self {
        match bin_value {
            0 => TransactionType::Deposit,
            1 => TransactionType::Transfer,
            2 => TransactionType::Withdrawal,
            _ => TransactionType::Unknown,
        }
    }
    
    pub fn to_u8(&self) -> u8 {
        match self {
            TransactionType::Deposit => 0,
            TransactionType::Transfer => 1,
            TransactionType::Withdrawal => 2,
            // Подумать над ошибкой
            TransactionType::Unknown => 255,
        }
    }
}

impl TransactionStatus {
    pub fn from_u8(bin_value: u8) -> Self {
        match bin_value {
            0 => TransactionStatus::Success,
            1 => TransactionStatus::Failure,
            2 => TransactionStatus::Pending,
            _ => TransactionStatus::Unknown,
        }
    }
    
    pub fn to_u8(&self) -> u8 {
        match self {
            TransactionStatus::Success => 0,
            TransactionStatus::Failure => 1,
            TransactionStatus::Pending => 2,
            // Подумать над ошибкой
            TransactionStatus::Unknown => 255,
        }
    }
}


impl BinFormatIO<Transaction> for Transaction {
    // Пребразование в бинарные данные
    // pub fn to_binary_bytes(&self) -> Vec<u8> {
    //     ...
    // }
    
    // Используем буферизированный вывод
    fn write_as_bin_to_writer<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), ParserError> {
        let description_bytes = self.description.as_bytes();
        let desc_len = description_bytes.len();
        
        // Размер записи
        let record_size = mem::size_of::<BinTransactionBodyFixed>()
                                 + desc_len;
        
        // Буфер на всю запись с предвыделенной памятью в куче
        let mut record_buffer = Vec::with_capacity(record_size);
        
        // Заполняем часть Header'а
        record_buffer.extend_from_slice(&[0x59, 0x50, 0x42, 0x4E]); // 'YPBN'
        record_buffer.extend_from_slice(&(record_size as u32).to_be_bytes());
        
        // Заполняем часть Body
        record_buffer.extend_from_slice(&self.tx_id.to_be_bytes());
        record_buffer.push(self.tx_type.to_u8());
        record_buffer.extend_from_slice(&self.from_user_id.to_be_bytes());
        record_buffer.extend_from_slice(&self.to_user_id.to_be_bytes());
        record_buffer.extend_from_slice(&(self.amount as i64).to_be_bytes());
        record_buffer.extend_from_slice(&self.timestamp.to_be_bytes());
        record_buffer.push(self.status.to_u8());
        record_buffer.extend_from_slice(&(desc_len as u32).to_be_bytes());
        record_buffer.extend_from_slice(description_bytes);
        
        // Пишем, можно через .map_err()
        if let Err(_) = writer.write_all(&record_buffer) {
            return Err(ParserError::BinTxWriteError);
        }
        
        Ok(())
    }

    // Возвращаем Result<Option<Transaction>, ..., поскольку Transaction может быть не получена в случае EOF
    fn new_from_bin_reader<R: std::io::Read>(mut reader: R) -> Result<Transaction, ParserError> {
        // Выделяем буфер для header'а
        let mut header_bytes = [0u8; mem::size_of::<BinTransactionHeader>()];

        const BODY_SIZE_NO_DESCR: usize = mem::size_of::<BinTransactionBodyFixed>();

        // Читаем строго количество байт 
        match reader.read_exact(&mut header_bytes) {
            Ok(()) => {},
            // Для обработки EOF (грубовато, лучше на Option переделать)
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return Err(ParserError::EOFEncountered),
            
            // Err(e) => return Err(format!("Ошибка чтения header'а: {}", e)),
            Err(_) => return Err(ParserError::BinTxReadError),
        }

        // Используем внешний crate byteorder для переводов из сетевого порядка байт и обратно
        let magic = &header_bytes[0..4];
        // Для чтения используем слайсы - ключевой момент
        let record_size = BigEndian::read_u32(&header_bytes[4..8]) as usize;
        
        // Проверка на 'YPBN'
        const EXPECTED_MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E]; 
        
        // Позволяет сравнивать таким образом
        if magic != EXPECTED_MAGIC {
            // return Err(format!("Неверное magic: {:?}, должно быть {:?}", magic, EXPECTED_MAGIC));
            return Err(ParserError::BinWrongMagicEncountered);
        }
        
        // Читаем body
        let mut body_bytes = vec![0u8; record_size];
        reader.read_exact(&mut body_bytes)
            // .map_err(|| format!("Не смогли прочитать {}", e))?;
            .map_err(|_| ParserError::BinTxReadError)?;
        
        // Прочитали меньше чем тело записи
        if body_bytes.len() < BODY_SIZE_NO_DESCR {
            // return Err("Слишком короткая запись".to_string());
            return Err(ParserError::BinReadLessThanBody);
        }
        
        // Извлекаем остальные записи
        let mut offset = 0;
        let mut field_len = mem::size_of::<u64>();

        let tx_id = BigEndian::read_u64(&body_bytes[offset..offset + field_len]); 
        offset += field_len;

        field_len = mem::size_of::<u8>();
        let tx_type = body_bytes[offset]; 
        offset += field_len;

        field_len = mem::size_of::<u64>();
        let from_user_id = BigEndian::read_u64(&body_bytes[offset .. offset + field_len]); 
        offset += field_len;

        field_len = mem::size_of::<u64>();
        let to_user_id = BigEndian::read_u64(&body_bytes[offset .. offset + field_len]); 
        offset += field_len;

        field_len = mem::size_of::<i64>();
        let amount = BigEndian::read_i64(&body_bytes[offset .. offset + field_len]); 
        offset += field_len;

        field_len = mem::size_of::<u64>();
        let timestamp = BigEndian::read_u64(&body_bytes[offset .. offset + field_len]); 
        offset += field_len;

        field_len = mem::size_of::<u8>();
        let status = body_bytes[offset]; 
        offset += field_len;

        field_len = mem::size_of::<u32>();
        let desc_len = BigEndian::read_u32(&body_bytes[offset .. offset + field_len]) as usize; 
        offset += field_len;
                
        
        // Проверка на длину Description
        if offset + desc_len > body_bytes.len() {
            // return Err("Указана слишком большая длина Description".to_string());
            return Err(ParserError::BinReadDescLenIsExcessive);
        }
        
        // Забираем description
        let description_bytes = &body_bytes[offset .. offset + desc_len];
        
        // Копируем - плохо
        // let description = String::from_utf8(description_bytes.to_vec())
        //     .map_err(|e| format!("Только UTF-8 символы: {}", e))?;

        // Через слайс
        let description = std::str::from_utf8(description_bytes)
            // .map_err(|e| format!("Только UTF-8 символы: {}", e))?
            .map_err(|_| ParserError::BinReadNonUtf8Symbols)?
            .to_string();

        let new_transaction = Transaction::new(tx_id,
                                                            TransactionType::from_u8(tx_type),
                                                            from_user_id,
                                                            to_user_id,
                                                            amount as u64,
                                                            timestamp,
                                                            TransactionStatus::from_u8(status),
                                                            description); 

        println!("Прочитанная запись: {:?}", new_transaction);

        Ok(new_transaction)

    }
}

impl CsvFormatIO<Transaction> for Transaction {
    fn new_from_csv_reader<R: std::io::BufRead>(mut reader: R) -> Result<Transaction, ParserError> {
        // Можно весь прочитать: match reader.read_to_string(&mut buffer) ...

        // let mut buf_reader = BufReader::new(reader);
        // Создаём итератор для пропуска header'а - первой строки 
        // let mut buf_reader = BufReader::new(reader);

        // TODO: HashMap для однократности полей (не нужен, считаем колонки)
        // let mut _cur_tx_hashmap = HashMap::<String, String>::new();
        let mut line_from_buffer = String::new();
            
        // Читаем одну строку
        let bytes_read = reader.read_line(&mut line_from_buffer)
            .map_err(|_| ParserError::CsvLineReadError)?;
            
        // Обработка EOF - пока подумать
        if bytes_read == 0 {
            return Err(ParserError::EOFEncountered);
        }
        
        print!("Прочитанная строка: {}", line_from_buffer);

        // Разделяем строку по запятым
        let columns: Vec<&str> = line_from_buffer.trim().split(',').collect();

        // Если нужное количество столбцов
        if columns.len() == 8 {

            // Получем поля из вектора
            // Примечание:  сделал с дефолтными значениями грубовато, можно под ошибки поправить

            // 1. Transaction ID
            let tx_id = Transaction::parse_u64_with_warning(columns[0], 0);

            // 2. Transaction Type: сравниваем с &str
            let tx_type = match columns[1] {
                "DEPOSIT" => TransactionType::Deposit,
                "WITHDRAWAL" => TransactionType::Withdrawal,
                "TRANSFER" => TransactionType::Transfer,
                _ => TransactionType::Unknown
            };
            
            // 3. From User
            let from_user_id = Transaction::parse_u64_with_warning(columns[2], 0);
            // 4. To User
            let to_user_id = Transaction::parse_u64_with_warning(columns[3], 0);
            // 5. Amount
            let amount = Transaction::parse_u64_with_warning(columns[4], 0);
            // 6. Timestamp
            let timestamp = Transaction::parse_u64_with_warning(columns[5], 0);

            // 7. Status
            let status = match columns[6] {
                "SUCCESS" => TransactionStatus::Success,
                "FAILURE" => TransactionStatus::Failure,
                "PENDING" => TransactionStatus::Pending,
                _ => TransactionStatus::Unknown,
            };

            // 8. Description
            let description = columns[7].to_string();

            // Всё ок, возвращаем транзакцию
            return Ok(Transaction{tx_id, tx_type, from_user_id, to_user_id, amount, timestamp, status, description});


        }
        else {
            // eprintln!("Неверный формат транзакции: {}", ok_line);
            return Err(ParserError::CsvWrongTransactionFormat(line_from_buffer));
        }

    }

    fn write_as_csv_to_writer<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), ParserError> {
       // Собираем все данные в текстовом виде в одну строку с newline'ами
       let mut out_data = String::new();

        out_data.push_str(&format!("{},{},{},{},{},{},{},{}\n", self.tx_id,
                                                                        self.tx_type,
                                                                        self.from_user_id,
                                                                        self.to_user_id,
                                                                        self.amount,
                                                                        self.timestamp,
                                                                        self.status,
                                                                        self.description));

        // (?) Не используем BufWriter, потому что сразу пишем всю строку целиком 

        writer.write_all(out_data.as_bytes())
            .map_err(|_| ParserError::CsvTxWriteError)?;

        Ok(())
    }
}

impl TextFormatIO<Transaction> for Transaction {
    fn new_from_text_reader<R: std::io::Read>(reader: R) -> Result<Transaction, ParserError> {
        // Чтение всего: match reader.read_to_string(&mut buffer) {
        // Все строки - не очень в данном случае
        // let lines = buf_reader.lines();

        let mut buf_reader = BufReader::new(reader);
        // HashMap для однократности полей
        let mut cur_tx_hashmap = HashMap::<String, String>::new();
        let mut line_from_buffer = String::new();

        loop {
            line_from_buffer.clear();
            
            // Читаем очередную строку
            let bytes_read = buf_reader.read_line(&mut line_from_buffer)
                .map_err(|_| ParserError::TextLineReadError)?;
            
            // Обработка EOF - пока подумать
            if bytes_read == 0 {
                if !cur_tx_hashmap.is_empty() {
                    // Неполная транзакция
                    return Err(ParserError::TextTxReadError);
                }
                return Err(ParserError::EOFEncountered);
            }

            // Обрабатывам комментарии и записи 
            if !line_from_buffer.is_empty() {

                // Пропускаем комменты
                if line_from_buffer.starts_with('#') {
                    continue;
                }

                // Разбиваем на токены и грубовато (для Description) проверяем, что формат "ключ: значение"
                let line_tokens: Vec<&str> = line_from_buffer.trim().split(": ").collect();        
                if line_tokens.len() != 2 {
                    return Err(ParserError::TextWrongLineFormat(line_from_buffer))
                }
                
                // TODO: проверить, что уже было поле
                cur_tx_hashmap.insert(line_tokens[0].to_string(), line_tokens[1].to_string());
            }
            // Пустая строка 
            else {

                if !cur_tx_hashmap.is_empty() {
                    // Если все поля, распарсиваем значения, сохраняем в транзакцию и возвращаем её
                    if Transaction::tx_hashmap_has_all_fields(&cur_tx_hashmap) {
                        // Не забывать, что ? возвращает unwrapped-значение, удобно
                        let transaction = Transaction::tx_from_tx_hashmap(&cur_tx_hashmap)?;
                        return Ok(transaction);
                    }
                    else {
                        println!("Не все поля присутствуют в транзакции {:?}", cur_tx_hashmap);
                        return Err(ParserError::TextMissingRequiredFields)
                    }
                }
                // Пропускаем все пустые строчки (?) (в документе строго одна)
            }
        }

        // Err("Искусственная ошибка для проверки вызова".to_string())
        // Ok(екфт)
    }

    fn write_as_text_to_writer<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), ParserError> {    
        // Пишем запись: перевод в строку => в байты
        if let Err(_) = writer.write_all(self.as_str().as_bytes()) {
            return Err(ParserError::TextTxWriteError);
        }

        Ok(())
    }
}

impl Transaction {
    pub fn as_str(&self) -> String {
        let mut tx_as_str = String::new();

        tx_as_str.push_str(&format!("# Запись о транзакции\n"));
        tx_as_str.push_str(&format!("TX_ID: {}\n", self.tx_id));
        tx_as_str.push_str(&format!("TX_TYPE: {}\n", self.tx_type));
        tx_as_str.push_str(&format!("FROM_USER_ID: {}\n", self.from_user_id));
        tx_as_str.push_str(&format!("TO_USER_ID: {}\n", self.to_user_id));
        tx_as_str.push_str(&format!("AMOUNT: {}\n", self.amount));
        tx_as_str.push_str(&format!("TIMESTAMP: {}\n", self.timestamp));
        tx_as_str.push_str(&format!("STATUS: {}\n", self.status));
        // Два newline в конце для разделения записей
        tx_as_str.push_str(&format!("DESCRIPTION: {}\n\n", self.description));

        return tx_as_str;
    }

    
    pub fn tx_from_tx_hashmap(tx_hashmap: &HashMap<String, String>) -> Result<Transaction, ParserError> {
        // Здесь чуть лишние проверки на наличие, tx_has_all_fields должна покрывать
        let tx_id = tx_hashmap.get("TX_ID")
            .ok_or(ParserError::TextMissingField)
            .and_then(|s| {
                s.parse::<u64>()
                    .map_err(|_| ParserError::TextWrongFieldValue)
            })?;
        
        let tx_type = tx_hashmap.get("TX_TYPE")
            .ok_or(ParserError::TextMissingField)
            .and_then(|s: &String| Self::parse_transaction_type(s))
            .map_err(|_| ParserError::TextWrongFieldValue)?;
        
        let from_user_id = tx_hashmap.get("FROM_USER_ID")
            .ok_or(ParserError::TextMissingField)
            .and_then(|s| {
                s.parse::<u64>()
                    .map_err(|_| ParserError::TextWrongFieldValue)
            })?;
        
        let to_user_id = tx_hashmap.get("TO_USER_ID")
            .ok_or(ParserError::TextMissingField)
            .and_then(|s| {
                s.parse::<u64>()
                    .map_err(|_| ParserError::TextWrongFieldValue)
            })?;
        
        let amount = tx_hashmap.get("AMOUNT")
            .ok_or(ParserError::TextMissingField)
            .and_then(|s| {
                s.parse::<u64>()
                    .map_err(|_| ParserError::TextWrongFieldValue)
            })?;
        
        let timestamp = tx_hashmap.get("TIMESTAMP")
            .ok_or(ParserError::TextMissingField)
            .and_then(|s| {
                s.parse::<u64>()
                    .map_err(|_| ParserError::TextWrongFieldValue)
            })?;
        
        let status = tx_hashmap.get("STATUS")
            .ok_or(ParserError::TextMissingField)
            .and_then(|s| Self::parse_transaction_status(s))?;
        
        let description = tx_hashmap.get("DESCRIPTION")
            .ok_or(ParserError::TextMissingField)
            .map(|s| s.to_string())?;

        Ok(Transaction::new(
            tx_id, tx_type, from_user_id, to_user_id, amount, timestamp, status, description
        ))
    }

    fn parse_transaction_type(type_str: &str) -> Result<TransactionType, ParserError> {
        match type_str {
            "DEPOSIT" => Ok(TransactionType::Deposit),
            "WITHDRAWAL" => Ok(TransactionType::Withdrawal),
            "TRANSFER" => Ok(TransactionType::Transfer),
            _ => Err(ParserError::TextWrongFieldValue),
        }
    }

    fn parse_transaction_status(status_str: &str) -> Result<TransactionStatus, ParserError> {
        match status_str {
            "SUCCESS" => Ok(TransactionStatus::Success),
            "FAILURE" => Ok(TransactionStatus::Failure),
            "PENDING" => Ok(TransactionStatus::Pending),
            _ => Err(ParserError::TextWrongFieldValue),
        }
    }

    // Проверка, что все значения есть в транзакции
    fn tx_hashmap_has_all_fields(tx_hash_map: &HashMap<String, String>) -> bool {
        const REQUIRED_FIELDS: [&str; 8] = ["TX_ID", "TX_TYPE", "FROM_USER_ID", "TO_USER_ID", "AMOUNT", "TIMESTAMP", "STATUS", "DESCRIPTION"];        

        for &required_field in &REQUIRED_FIELDS {
            if !tx_hash_map.contains_key(required_field) {
                return false;
            }
        }

        true
    }

    fn parse_u64_with_warning(in_str: &str, default_value: u64) -> u64 {
        match in_str.parse::<u64>() {
            Ok(parsed) => parsed,
            Err(_) => {
                eprintln!("Значение не распарсилось {}, устанавливается дефолтное {}", in_str, default_value);
                default_value
            }
        }
    }
}