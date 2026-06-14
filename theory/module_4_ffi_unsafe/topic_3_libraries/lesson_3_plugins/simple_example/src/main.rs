use libloading::{Library, Symbol};

pub enum TradeAction {
    Sell(u32),
    Buy(u32),
    None,
}

// Плагин как Library
pub struct Plugin {
    plugin: Library,
}

// Интерфейс плагина
pub struct PluginInterface<'a> {
    pub trade: Symbol<'a, extern "C" fn(prices: *const u32, prices_len: usize) -> TradeAction>,
} 

impl Plugin {
    pub fn new(filename: &str) -> Result<Self, libloading::Error> {
        Ok(Plugin {
            plugin: unsafe { Library::new(filename) }?,
        })
    }
    pub fn interface(&self) -> Result<PluginInterface<'_>, libloading::Error> {
        Ok(PluginInterface {
            // подгрузка функции по символу `trade`
            trade: unsafe { self.plugin.get("trade") }?,
        })
    }
} 

fn main() {

}