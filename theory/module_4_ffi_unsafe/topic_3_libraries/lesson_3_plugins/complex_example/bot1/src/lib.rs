use plugin_interface::{TradeDecision, TRADE_BUY, TRADE_NONE, TRADE_SELL};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn trade(
    prices: *const u32,
    prices_len: usize,
    out: *mut TradeDecision,
) -> i32 {
    if out.is_null() {
        return -1;
    }
    if prices.is_null() && prices_len != 0 {
        return -2;
    }

    let prices = if prices_len == 0 {
        &[]
    } else {
        // SAFETY: хост гарантирует валидный срез на время вызова.
        unsafe { core::slice::from_raw_parts(prices, prices_len) }
    };

    const N: usize = 3;
    let last = match prices.len() {
        0..=N => prices,
        len => &prices[len - N..],
    };

    let decision = if last.windows(2).all(|w| w[0] <= w[1]) && !last.is_empty() {
        TradeDecision { action: TRADE_BUY, amount: 100 }
    } else if last.windows(2).all(|w| w[0] >= w[1]) && !last.is_empty() {
        TradeDecision { action: TRADE_SELL, amount: 100 }
    } else {
        TradeDecision { action: TRADE_NONE, amount: 0 }
    };

    // SAFETY: out не null; записываем один repr(C) объект.
    unsafe { *out = decision };
    0
}