use serde::Deserialize;

/// API 문자열 `EVEN` / `RISE` / `FALL` 과 매칭
/// https://docs.upbit.com/kr/reference/list-tickers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Change {
    Even, // 보합
    Rise, // 상승
    Fall, // 하락
}

/// https://docs.upbit.com/kr/reference/list-tickers
/// 업비트 페어 단위 현재가
#[derive(Debug, Clone, Deserialize)]
pub struct UpbitPairQuote {
    pub market: String,
    pub trade_date: String,
    pub trade_time: String,
    pub trade_date_kst: String,
    pub trade_time_kst: String,
    pub trade_timestamp: f64,
    pub opening_price: f64, // 시가 (첫 거래 가격)
    pub high_price: f64,    // 고가
    pub low_price: f64,     // 저가
    pub trade_price: f64,   // 종가
    pub prev_closing_price: f64, // 전일 종가 (UTC 0시 기준)
    pub change: Change,
    pub trade_volume: f64, // 최근 거래 수량
    pub timestamp: f64,    // 현재가가 반영된 시간
}

