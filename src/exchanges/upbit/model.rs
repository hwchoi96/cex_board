use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::constants::{BuySellType, ContStrategy, OrderType, SmpType};

/// `GET /v1/market/all` 한 행 — [문서](https://docs.upbit.com/kr/reference/list-market-all)
///
/// 실제 JSON에는 `market`·`korean_name`·`english_name`만 있고 `market_event`는 없다.
/// 다른 엔드포인트나 향후 확장을 위해 `market_event`는 기본값으로 채운다.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct UpbitTradePair {
    pub market: String,
    pub korean_name: String,
    pub english_name: String,
    #[serde(default)]
    pub market_event: MarketEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct MarketEvent {
    #[serde(default)]
    pub warning: bool,
    #[serde(rename = "PRICE_FLUCTUATIONS", default)]
    pub price_fluctuation: bool,
    #[serde(rename = "TRADING_VOLUME_SOARING", default)]
    pub trading_volume_soaring: bool,
    #[serde(rename = "DEPOSIT_AMOUNT_SOARING", default)]
    pub deposit_amount_soaring: bool,
    #[serde(rename = "GLOBAL_PRICE_DIFFERENCES", default)]
    pub global_price_differences: bool,
    #[serde(rename = "CONCENTRATION_OF_SMALL_ACCOUNTS", default)]
    pub concentration_of_small_accounts: bool,
}

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
    pub order_book_unit: Vec<OrderBookUnit>,
    pub level: Decimal,
}

/// https://docs.upbit.com/kr/reference/list-candles-minutes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpbitMinuteCandle {
    pub market: String,
    #[serde(rename = "candle_date_time_utc")]
    pub candle_date_time_utc: String,
    pub candle_date_time_kst: String,
    pub opening_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub trade_price: Decimal,
    pub timestamp: u64,
    pub candle_acc_trade_price: Decimal,
    pub candle_acc_trade_volume: Decimal,
    pub unit: i32,
}

/// https://docs.upbit.com/kr/reference/list-accounts
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpbitMyBalance {
    pub currency: String,
    pub balance: Decimal,
    pub locked: Decimal,
    pub avg_buy_price: Decimal,
    pub avg_buy_price_modified: bool,
    pub unit_currency: String,
}

/// https://docs.upbit.com/kr/reference/new-order
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpbitOrderRequest {
    pub market: String,
    pub side: BuySellType,
    #[serde(rename = "ord_type")]
    pub ord_type: OrderType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<Decimal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    #[serde(default, rename = "time_in_force", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<ContStrategy>,
    #[serde(default, rename = "smp_type", skip_serializing_if = "Option::is_none")]
    pub smp_type: Option<SmpType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_all_json_has_no_market_event_field() {
        let j = r#"{"market":"KRW-BTC","korean_name":"비트코인","english_name":"Bitcoin"}"#;
        let r: UpbitTradePair = serde_json::from_str(j).unwrap();
        assert_eq!(r.market, "KRW-BTC");
        assert_eq!(r.market_event, MarketEvent::default());
    }
}
