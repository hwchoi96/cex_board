use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};

use crate::upbit_client::UpbitPublicClient;
use crate::markets::DEFAULT_UPBIT_MARKETS;

#[derive(Clone)]
pub struct AppState {
    pub upbit: Arc<UpbitPublicClient>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/exchanges/upbit/ticker", get(upbit_tickers))
        .with_state(state)
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
