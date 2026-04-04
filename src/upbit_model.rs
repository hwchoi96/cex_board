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
    pub opening_price: Decimal, // 시가 (첫 거래 가격)
    pub high_price: Decimal,    // 고가
    pub low_price: Decimal,     // 저가
    pub trade_price: Decimal,   // 종가
    pub prev_closing_price: Decimal, // 전일 종가 (UTC 0시 기준)
    pub change: Change,
    pub trade_volume: Decimal, // 최근 거래 수량
    pub timestamp: u64,    // 현재가가 반영된 시간
}

pub struct OrderBookUnit {
    pub ask_price: Decimal, // 매도 호가
    pub bid_price: Decimal, // 매수 호가
    pub ask_size: Decimal, // 매도 잔량
    pub bid_size: Decimal, // 매수 잔량
}

/// https://docs.upbit.com/kr/reference/list-orderbooks
/// 업비트 오더북 호가 정보
pub struct UpbitOrderBook {

    pub market: String,
    pub timestamp: u64,
    pub total_ask_size: Decimal, // 현재 호가의 전체 매도 잔량 합계
    pub total_bid_size: Decimal, // 현재 호가의 전체 매수 잔량 합계
    pub order_book_unit: Vec<OrderBookUnit>,
    pub level: Decimal // 해당 호가가 적용된 가격 단위
}