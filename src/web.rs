use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json,
    Router,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::constants::{BuySellType, ContStrategy, OrderType};
use crate::exchanges::upbit::{
    UpbitError, UpbitMinuteCandle, UpbitMyBalance, UpbitOrderBook, UpbitOrderRequest,
    UpbitPairQuote, UpbitPublicClient, UpbitTradePair, UpbitWsTicker,
};

/// 업비트 분봉 API에서 허용하는 minute 단위
pub const MINUTE_CANDLE_UNITS: &[i32] = &[1, 3, 5, 15, 30, 60, 240];

#[derive(Clone)]
pub struct AppState {
    pub upbit: Arc<UpbitPublicClient>,
    /// 업비트 WS 티커를 브라우저 등 여러 구독자에게 fan-out
    pub ticker_broadcast: broadcast::Sender<UpbitWsTicker>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/ws/upbit-ticker", get(ws_upbit_ticker))
        .route("/api/exchanges/upbit/ticker", get(upbit_tickers))
        .route("/api/exchanges/upbit/orderbook", get(upbit_orderbook))
        .route("/api/exchanges/upbit/candles/minutes", get(upbit_minute_candle))
        .route("/api/exchanges/upbit/balance", get(upbit_my_balance))
        .route("/api/exchanges/upbit/markets", get(upbit_trade_pair))
        .route("/api/exchanges/upbit/orders", post(upbit_order))
        .with_state(state)
}

/// https://docs.upbit.com/kr/reference/new-order
fn validate_new_order_request(body: &UpbitOrderRequest) -> Result<(), &'static str> {
    if body.market.trim().is_empty() {
        return Err("market은 비어 있을 수 없습니다.");
    }
    if let Some(id) = &body.identifier {
        if id.trim().is_empty() {
            return Err("identifier를 보낼 때는 비어 있지 않은 문자열이어야 합니다.");
        }
    }
    if matches!(body.time_in_force, Some(ContStrategy::PostOnly)) && body.smp_type.is_some() {
        return Err("post_only는 smp_type과 함께 사용할 수 없습니다.");
    }
    if matches!(body.time_in_force, Some(ContStrategy::PostOnly)) && body.ord_type != OrderType::LIMIT {
        return Err("post_only는 지정가(limit)에서만 사용할 수 있습니다.");
    }

    let pos = |d: &Option<Decimal>| matches!(d, Some(x) if *x > Decimal::ZERO);

    match body.ord_type {
        OrderType::LIMIT => {
            if !pos(&body.volume) {
                return Err("지정가(limit)는 volume이 필요합니다.");
            }
            if !pos(&body.price) {
                return Err("지정가(limit)는 price가 필요합니다.");
            }
        }
        OrderType::PRICE => {
            if body.time_in_force.is_some() {
                return Err("시장가 매수(ord_type=price)에는 time_in_force를 보내면 안 됩니다.");
            }
            if body.side != BuySellType::Buy {
                return Err("시장가 매수(ord_type=price)는 side가 bid여야 합니다.");
            }
            if !pos(&body.price) {
                return Err("시장가 매수는 주문 총액 price가 필요합니다.");
            }
            if body.volume.is_some() {
                return Err("시장가 매수에는 volume을 보내면 안 됩니다.");
            }
        }
        OrderType::MARKET => {
            if body.time_in_force.is_some() {
                return Err("시장가 매도(ord_type=market)에는 time_in_force를 보내면 안 됩니다.");
            }
            if body.side != BuySellType::Sell {
                return Err("시장가 매도(ord_type=market)는 side가 ask여야 합니다.");
            }
            if !pos(&body.volume) {
                return Err("시장가 매도는 volume이 필요합니다.");
            }
            if body.price.is_some() {
                return Err("시장가 매도에는 price를 보내면 안 됩니다.");
            }
        }
        OrderType::BEST => {
            match body.side {
                BuySellType::Buy => {
                    if !pos(&body.price) {
                        return Err("최유리 매수(best·bid)는 주문 총액 price가 필요합니다.");
                    }
                    if body.volume.is_some() {
                        return Err("최유리 매수에는 volume을 보내면 안 됩니다.");
                    }
                }
                BuySellType::Sell => {
                    if !pos(&body.volume) {
                        return Err("최유리 매도(best·ask)는 volume이 필요합니다.");
                    }
                    if body.price.is_some() {
                        return Err("최유리 매도에는 price를 보내면 안 됩니다.");
                    }
                }
            }
            match body.time_in_force {
                Some(ContStrategy::IOC) | Some(ContStrategy::FOK) => {}
                _ => return Err("최유리(best)는 time_in_force에 ioc 또는 fok가 필요합니다."),
            }
        }
    }

    Ok(())
}

/// `markets`: 쉼표로 구분 (예: `KRW-BTC` 또는 `KRW-BTC,KRW-ETH`). 없으면 `KRW-BTC`.
#[derive(Deserialize)]
pub struct UpbitOrderbookQuery {
    pub markets: Option<String>,
}

#[derive(Deserialize)]
pub struct UpbitMinuteCandleQuery {
    pub market: String,
    pub unit: i32,
    pub count: i32,
}

async fn index() -> Html<&'static str> {
    Html(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/static/index.html"
    )))
}

/// 브라우저 클라이언트 → 업비트에서 받은 `UpbitWsTicker` JSON(텍스트 프레임) 스트림
async fn ws_upbit_ticker(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_upbit_ticker_ws(socket, state))
}

async fn handle_upbit_ticker_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.ticker_broadcast.subscribe();

    loop {
        tokio::select! {
            recv = rx.recv() => {
                match recv {
                    Ok(t) => {
                        let Ok(json) = serde_json::to_string(&t) else {
                            continue;
                        };
                        if socket
                            .send(Message::Text(json.into()))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        /* 느린 클라이언트는 스킵 */
                    }
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
            inc = socket.recv() => {
                match inc {
                    None => return,
                    Some(Ok(Message::Close(_))) => return,
                    Some(Ok(Message::Ping(p))) => {
                        let _ = socket.send(Message::Pong(p)).await;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => return,
                }
            }
        }
    }
}

async fn upbit_tickers(State(state): State<AppState>) -> impl IntoResponse {
    let codes = match state.upbit.krw_market_codes().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    };
    if codes.is_empty() {
        return Json(Vec::<UpbitPairQuote>::new()).into_response();
    }
    let refs: Vec<&str> = codes.iter().map(|s| s.as_str()).collect();
    match state.upbit.get_quote_all(&refs).await {
        Ok(rows) => Json(rows).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    }
}

async fn upbit_orderbook(
    State(state): State<AppState>,
    Query(q): Query<UpbitOrderbookQuery>,
) -> impl IntoResponse {
    let raw = q.markets.unwrap_or_else(|| "KRW-BTC".to_string());
    let markets: Vec<&str> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if markets.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "markets 쿼리가 비었습니다. 예: ?markets=KRW-BTC",
        )
            .into_response();
    }

    match state.upbit.get_order_book(&markets).await {
        Ok(rows) => Json::<Vec<UpbitOrderBook>>(rows).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    }
}

async fn upbit_minute_candle(
    State(state): State<AppState>,
    Query(q): Query<UpbitMinuteCandleQuery>,
) -> impl IntoResponse {
    let krw = match state.upbit.krw_market_codes().await {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    };
    if !krw.iter().any(|m| m == &q.market) {
        return (
            StatusCode::BAD_REQUEST,
            "market은 업비트 KRW 마켓 목록에 있는 코드여야 합니다.",
        )
            .into_response();
    }

    if !MINUTE_CANDLE_UNITS.contains(&q.unit) {
        return (
            StatusCode::BAD_REQUEST,
            "unit은 1, 3, 5, 15, 30, 60, 240 중 하나여야 합니다.",
        )
            .into_response();
    }

    if !(1..=200).contains(&q.count) {
        return (
            StatusCode::BAD_REQUEST,
            "count는 1 이상 200 이하여야 합니다.",
        )
            .into_response();
    }

    match state
        .upbit
        .get_minute_candle(q.unit, q.market, q.count)
        .await
    {
        Ok(rows) => Json::<Vec<UpbitMinuteCandle>>(rows).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    }
}

async fn upbit_order(
    State(state): State<AppState>,
    Json(body): Json<UpbitOrderRequest>,
) -> impl IntoResponse {
    if let Err(msg) = validate_new_order_request(&body) {
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }
    let krw = match state.upbit.krw_market_codes().await {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    };
    if !krw.iter().any(|m| m == &body.market) {
        return (
            StatusCode::BAD_REQUEST,
            "market은 업비트 KRW 마켓 목록에 있는 코드여야 합니다.",
        )
            .into_response();
    }

    match state.upbit.order(body).await {
        Ok(rows) => Json(rows).into_response(),
        Err(e) => {
            let status = if matches!(e, UpbitError::AuthorizationError) {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::BAD_GATEWAY
            };
            (status, e.to_string()).into_response()
        }
    }
}

async fn upbit_my_balance(State(state): State<AppState>) -> impl IntoResponse {
    match state.upbit.get_my_balance().await {
        Ok(rows) => Json::<Vec<UpbitMyBalance>>(rows).into_response(),
        Err(e) => {
            let status = if matches!(e, UpbitError::AuthorizationError) {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::BAD_GATEWAY
            };
            (status, e.to_string()).into_response()
        }
    }
}

async fn upbit_trade_pair(State(state): State<AppState>) -> impl IntoResponse {

    match state.upbit.get_trading_pair().await {
        Ok(rows) => Json::<Vec<UpbitTradePair>>(rows).into_response(),
        Err(e) => {
            let status = if matches!(e, UpbitError::AuthorizationError) {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::BAD_GATEWAY
            };
            (status, e.to_string()).into_response()
        }
    }
}
