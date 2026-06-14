use rand;
use std::{thread::sleep, time::Duration};

use plugin_interface::{Plugin, TradeDecision, TRADE_BUY, TRADE_NONE, TRADE_SELL};

fn main() {
    let plugin_path = std::env::args().nth(1);
    let plugin_path = match plugin_path {
        Some(filename) => filename,
        None => {
            let runner = std::env::args().next();
            let runner = runner.as_ref().map_or("exchange", |s| s);
            println!("Usage: {runner} <plugin path>");
            std::process::exit(1);
        }
    };
    let plugin = Plugin::new(&plugin_path).unwrap();
    let iface = plugin.interface().unwrap();
    let mut prices = Vec::new();
    loop {
        price_updates(&mut prices);

        let mut decision = TradeDecision {
            action: TRADE_NONE,
            amount: 0,
        };
        let rc = unsafe { (iface.trade)(prices.as_ptr(), prices.len(), &mut decision) };
        if rc != 0 {
            eprintln!("plugin returned error code {rc}");
            break;
        }

        match decision.action {
            TRADE_SELL => println!("Selling {} of currency", decision.amount),
            TRADE_BUY => println!("Buying {} of currency", decision.amount),
            TRADE_NONE => println!("Doing nothing"),
            other => println!("Unknown action code: {other}"),
        }

        sleep(Duration::from_secs(1));
    }
}

fn price_updates(prices: &mut Vec<u32>) {
    prices.push(rand::random_range(50000..120780));
} 