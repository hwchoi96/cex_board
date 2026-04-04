//! CEX Board HTTP 서버 엔트리.
//! `http://127.0.0.1:9090` — 대시보드 HTML 및 거래소 API.

use std::net::SocketAddr;
use std::sync::Arc;

use crate::upbit_client::UpbitPublicClient;
use crate::web::{create_router, AppState};

mod markets;
mod upbit_client;
mod upbit_model;
mod web;

/// 웹 대시보드·API 바인드 포트
const HTTP_PORT: u16 = 9090;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let upbit = Arc::new(UpbitPublicClient::new()?);
    let app = create_router(AppState { upbit });

    let addr = SocketAddr::from(([127, 0, 0, 1], HTTP_PORT));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("CEX Board — http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
