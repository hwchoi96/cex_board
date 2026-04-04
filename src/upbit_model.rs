use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// API 문자열 `EVEN` / `RISE` / `FALL` 과 매칭
/// https://docs.upbit.com/kr/reference/list-tickers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Change {
    Even, // 보합
    Rise, // 상승
    Fall, // 하락
}

/// https://docs.upbit.com/kr/reference/list-tickers
/// 업비트 페어 단위 현재가
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpbitPairQuote {
    pub market: String,
    pub trade_date: String,
    pub trade_time: String,
    pub trade_date_kst: String,
    pub trade_time_kst: String,
    pub trade_timestamp: u64,
    pub opening_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub trade_price: Decimal,
    pub prev_closing_price: Decimal,
    pub change: Change,
    pub trade_volume: Decimal,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderBookUnit {
    pub ask_price: Decimal,
    pub bid_price: Decimal,
    pub ask_size: Decimal,
    pub bid_size: Decimal,
}

/// https://docs.upbit.com/kr/reference/list-orderbooks
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpbitOrderBook {
    pub market: String,
    pub timestamp: u64,
    pub total_ask_size: Decimal,
    pub total_bid_size: Decimal,
    #[serde(rename = "orderbook_units")]
    pub order_book_units: Vec<OrderBookUnit>,
    pub level: Decimal,
}