use serde::{Deserialize, Serialize};

/// 업비트 주문 `side`: Rust에서는 `Buy` / `Sell`, API·JSON에서는 `bid` / `ask`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuySellType {
    #[serde(rename = "bid")]
    Buy,
    #[serde(rename = "ask")]
    Sell,
}

impl BuySellType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            BuySellType::Buy => "bid",
            BuySellType::Sell => "ask",
        }
    }
}

/// 업비트 주문 `ord_type`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// 지정가 매수 / 매도
    #[serde(rename = "limit")]
    LIMIT,
    /// 시장가 매수
    #[serde(rename = "price")]
    PRICE,
    /// 시장가 매도
    #[serde(rename = "market")]
    MARKET,
    /// 최유리 지정가 매수 / 매도
    #[serde(rename = "best")]
    BEST,
}

impl OrderType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            OrderType::LIMIT => "limit",
            OrderType::PRICE => "price",
            OrderType::MARKET => "market",
            OrderType::BEST => "best",
        }
    }
}


/// 체결 전략
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContStrategy {
    /// Immediately or Cancel (즉시 체결 가능한 것만 체결하고, 나머지 미체결은 취소하는 방식)
    #[serde(rename = "ioc")]
    IOC,
    /// Fill or Kill (전량 체결 혹은 전량 주문 취소 방식)
    #[serde(rename = "fok")]
    FOK,
    /// 메이커 주문으로 생성될 수 있는 주문에 대해서만 주문이 접수되는 방식, 테이커 조건이 될 때 전량 취소 (부분 or 전량 체결이 될 때)
    #[serde(rename = "post_only")]
    PostOnly,
}

/// 자전거래 체결 방지 `smp_type` — https://docs.upbit.com/kr/reference/new-order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmpType {
    #[serde(rename = "cancel_maker")]
    CancelMaker,
    #[serde(rename = "cancel_taker")]
    CancelTaker,
    #[serde(rename = "reduce")]
    Reduce,
}
