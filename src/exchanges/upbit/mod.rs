mod auth;
mod client;
mod model;
mod websocket;

pub use client::{UpbitError, UpbitPublicClient};
#[allow(unused_imports)]
pub use websocket::run_quote_websocket_with_reconnect;
// 웹소켓 타입 등 외부/추후 사용을 위해 re-export (바이너리 단일 크레이트에서는 unused 경고)
#[allow(unused_imports)]
pub use model::{
    UpbitMinuteCandle, UpbitMyBalance, UpbitOrderBook, UpbitOrderRequest, UpbitPairQuote,
    UpbitTradePair, UpbitWsAskBid, UpbitWsMarketState, UpbitWsMarketWarning, UpbitWsStreamType,
    UpbitWsTicker,
};
