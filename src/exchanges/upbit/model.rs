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

/// WebSocket ticker `ask_bid` — `ASK` / `BID`
/// <https://docs.upbit.com/kr/reference/websocket-ticker>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UpbitWsAskBid {
    Ask,
    Bid,
}

/// WebSocket ticker `market_state`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UpbitWsMarketState {
    Preview,
    Active,
    Delisted,
}

/// WebSocket ticker `market_warning` — Deprecated 필드이나 명세상 유지
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UpbitWsMarketWarning {
    None,
    Caution,
}

/// WebSocket ticker `stream_type`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UpbitWsStreamType {
    Snapshot,
    Realtime,
}

/// 업비트 공개 WebSocket에서 파싱한 페이로드(티커·호가 등)를 한 채널로 보낼 때 공통 타입.
///
/// `Box<dyn UpbitWsEvent>`는 `Clone`이 없어 `broadcast`로 넘길 때는 `Arc::from(box)`로 바꿉니다.
pub trait UpbitWsEvent: Send + Sync + std::fmt::Debug{
    fn to_json_string(&self) -> Result<String, serde_json::Error>;
}

/// WebSocket `ticker` 구독 — `format: "DEFAULT"` 페이로드
/// <https://docs.upbit.com/kr/reference/websocket-ticker>
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpbitWsTicker {
    #[serde(rename = "type")]
    pub kind: String,
    pub code: String,
    pub opening_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub trade_price: Decimal,
    pub prev_closing_price: Decimal,
    pub acc_trade_price: Decimal,
    pub change: Change,
    pub change_price: Decimal,
    pub signed_change_price: Decimal,
    pub change_rate: Decimal,
    pub signed_change_rate: Decimal,
    pub ask_bid: UpbitWsAskBid,
    pub trade_volume: Decimal,
    pub acc_trade_volume: Decimal,
    pub trade_date: String,
    pub trade_time: String,
    pub trade_timestamp: u64,
    pub acc_ask_volume: Decimal,
    pub acc_bid_volume: Decimal,
    pub highest_52_week_price: Decimal,
    pub highest_52_week_date: String,
    pub lowest_52_week_price: Decimal,
    pub lowest_52_week_date: String,
    pub market_state: UpbitWsMarketState,
    #[serde(default)]
    pub is_trading_suspended: bool,
    #[serde(default)]
    pub delisting_date: Option<String>,
    pub market_warning: UpbitWsMarketWarning,
    pub timestamp: u64,
    pub acc_trade_price_24h: Decimal,
    pub acc_trade_volume_24h: Decimal,
    pub stream_type: UpbitWsStreamType,
}

impl UpbitWsEvent for UpbitWsTicker {
    fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderBookUnit {
    pub ask_price: Decimal,
    pub bid_price: Decimal,
    pub ask_size: Decimal,
    pub bid_size: Decimal,
}

/// WebSocket `orderbook` 구독 — `format: "DEFAULT"` 페이로드
/// <https://docs.upbit.com/kr/reference/websocket-orderbook>
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpbitWsOrderbook {
    #[serde(rename = "type")]
    pub kind: String,
    pub code: String,
    pub total_ask_size: Decimal,
    pub total_bid_size: Decimal,
    #[serde(rename = "orderbook_units")]
    pub orderbook_units: Vec<OrderBookUnit>,
    pub timestamp: u64,
    pub stream_type: UpbitWsStreamType,
    pub level: Decimal,
}

impl UpbitWsEvent for UpbitWsOrderbook {
    fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
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
    /// 최근 체결 1건의 거래량(기준 통화, 예: BTC). 24h 거래대금과 무관.
    pub trade_volume: Decimal,
    /// 최근 24시간 누적 거래대금(KRW 마켓이면 원화). 시세 「거래대금」은 이 값.
    pub acc_trade_price_24h: Decimal,
    pub timestamp: u64,
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

    /// 문서 예시(JSON) — `format: "DEFAULT"`
    /// <https://docs.upbit.com/kr/reference/websocket-ticker>
    #[test]
    fn ws_ticker_default_deserializes_doc_example() {
        let j = r#"{
            "type": "ticker",
            "code": "KRW-BTC",
            "opening_price": 31883000,
            "high_price": 32310000,
            "low_price": 31855000,
            "trade_price": 32287000,
            "prev_closing_price": 31883000.00000000,
            "acc_trade_price": 78039261076.51241000,
            "change": "RISE",
            "change_price": 404000.00000000,
            "signed_change_price": 404000.00000000,
            "change_rate": 0.0126713295,
            "signed_change_rate": 0.0126713295,
            "ask_bid": "ASK",
            "trade_volume": 0.03103806,
            "acc_trade_volume": 2429.58834336,
            "trade_date": "20230221",
            "trade_time": "074102",
            "trade_timestamp": 1676965262139,
            "acc_ask_volume": 1146.25573608,
            "acc_bid_volume": 1283.33260728,
            "highest_52_week_price": 57678000.00000000,
            "highest_52_week_date": "2022-03-28",
            "lowest_52_week_price": 20700000.00000000,
            "lowest_52_week_date": "2022-12-30",
            "market_state": "ACTIVE",
            "is_trading_suspended": false,
            "delisting_date": null,
            "market_warning": "NONE",
            "timestamp": 1676965262177,
            "acc_trade_price_24h": 228827082483.70729000,
            "acc_trade_volume_24h": 7158.80283560,
            "stream_type": "REALTIME"
        }"#;

        let t: UpbitWsTicker = serde_json::from_str(j).unwrap();
        assert_eq!(t.kind, "ticker");
        assert_eq!(t.code, "KRW-BTC");
        assert_eq!(t.change, Change::Rise);
        assert_eq!(t.ask_bid, UpbitWsAskBid::Ask);
        assert_eq!(t.market_state, UpbitWsMarketState::Active);
        assert_eq!(t.market_warning, UpbitWsMarketWarning::None);
        assert_eq!(t.stream_type, UpbitWsStreamType::Realtime);
    }

    /// 문서 예시(JSON) — `format: "DEFAULT"`
    /// <https://docs.upbit.com/kr/reference/websocket-orderbook>
    #[test]
    fn ws_orderbook_default_deserializes_doc_example() {
        let j = r#"{
            "type": "orderbook",
            "code": "KRW-BTC",
            "timestamp": 1746601573804,
            "total_ask_size": 4.79158413,
            "total_bid_size": 2.65609625,
            "orderbook_units": [
                {
                    "ask_price": 137002000,
                    "bid_price": 137001000,
                    "ask_size": 0.10623869,
                    "bid_size": 0.03656812
                }
            ],
            "stream_type": "SNAPSHOT",
            "level": 0
        }"#;

        let o: UpbitWsOrderbook = serde_json::from_str(j).unwrap();
        assert_eq!(o.kind, "orderbook");
        assert_eq!(o.code, "KRW-BTC");
        assert_eq!(o.orderbook_units.len(), 1);
        assert_eq!(o.orderbook_units[0].ask_price, Decimal::from(137002000));
        assert_eq!(o.stream_type, UpbitWsStreamType::Snapshot);
    }
}
