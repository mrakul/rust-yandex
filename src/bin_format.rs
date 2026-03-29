use crate::error::ParserError;
pub trait BinFormatIO<InternalType> {
    // Читает отчёт из любого приёмника, реализующего трейт Read (статическая, для создания экземпляра "класса")
    fn new_from_bin_reader<R: std::io::Read>(reader: R) -> Result<InternalType, ParserError>;

    // Записывает отчёт в любой приёмник, реализующий трейт Write
    fn write_as_bin_to_writer<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), ParserError>;
}