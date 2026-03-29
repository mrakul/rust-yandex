use crate::error::ParserError;

pub trait CsvFormatIO<InternalType> {
    // Читает отчёт из любого приёмника, реализующего трейт Read (статическая, для создания экземпляра "класса")
    fn new_from_csv_reader<R: std::io::BufRead>(reader: R) -> Result<InternalType, ParserError>;

    // Записывает отчёт в любой приёмник, реализующий трейт Write
    fn write_as_csv_to_writer<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), ParserError>;
}