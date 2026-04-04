use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::markets::DEFAULT_UPBIT_MARKETS;
use crate::upbit_client::UpbitPublicClient;
use crate::upbit_model::{UpbitMinuteCandle, UpbitOrderBook};

/// 업비트 분봉 API에서 허용하는 minute 단위
pub const MINUTE_CANDLE_UNITS: &[i32] = &[1, 3, 5, 15, 30, 60, 240];

#[derive(Clone)]
pub struct AppState {
    pub upbit: Arc<UpbitPublicClient>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/exchanges/upbit/ticker", get(upbit_tickers))
        .route("/api/exchanges/upbit/orderbook", get(upbit_orderbook))
        .route("/api/exchanges/upbit/candles/minutes", get(upbit_minute_candle))
        .with_state(state)
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

async fn upbit_tickers(State(state): State<AppState>) -> impl IntoResponse {
    match state.upbit.get_quote_all(DEFAULT_UPBIT_MARKETS).await {
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
    if !DEFAULT_UPBIT_MARKETS
        .iter()
        .any(|&m| m == q.market.as_str())
    {
        return (
            StatusCode::BAD_REQUEST,
            format!("market은 {DEFAULT_UPBIT_MARKETS:?} 중 하나여야 합니다."),
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
