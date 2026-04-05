//! CEX Board HTTP 서버 엔트리.
//! `http://127.0.0.1:9090` — 대시보드 HTML 및 거래소 API.

use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use crate::upbit_client::UpbitPublicClient;
use crate::web::{create_router, AppState};

mod constants;
mod upbit_auth;
mod upbit_client;
mod upbit_model;
mod web;

/// 웹 대시보드·API 바인드 포트
const HTTP_PORT: u16 = 9090;

#[derive(Parser)]
#[command(name = "cex_board")]
struct Args {
    /// Upbit Open API Access Key. `secret_key`와 함께 있어야 Exchange API(JWT) 사용.
    #[arg(long = "access_key", value_name = "ACCESS_KEY")]
    access_key: Option<String>,
    /// Upbit Open API Secret Key. `=` 형식 권장: `--secret_key=...` (값이 `-`로 시작할 때).
    #[arg(long = "secret_key", value_name = "SECRET_KEY")]
    secret_key: Option<String>,
}

fn key_from_cli_or_env(cli: Option<String>, env_name: &str) -> Option<String> {
    cli.or_else(|| std::env::var(env_name).ok())
        .filter(|s| !s.trim().is_empty())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    // IDE에 빈 `UPBIT_*`만 있고 clap이 env를 먹는 경우 등을 피하기 위해, CLI·환경변수는 여기서만 병합합니다.
    let access_key = key_from_cli_or_env(args.access_key, "UPBIT_ACCESS_KEY");
    let secret_key = key_from_cli_or_env(args.secret_key, "UPBIT_SECRET_KEY");

    let upbit = Arc::new(UpbitPublicClient::new(access_key, secret_key)?);

    eprintln!(
        "[cex_board] 키 길이(정규화 후, 값 미출력): access={:?}자, secret={:?}자",
        upbit.access_key.as_ref().map(String::len),
        upbit.secret_key.as_ref().map(String::len),
    );
    let app = create_router(AppState {
        upbit: upbit.clone(),
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], HTTP_PORT));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("CEX Board — http://{}", addr);
    eprintln!(
        "업비트 Exchange API(잔고 등): {}",
        if upbit.access_key.is_some() && upbit.secret_key.is_some() {
            "키 로드됨 (JWT 사용). invalid_access_key면 대시보드의 Access/Secret이 뒤바뀌지 않았는지 확인."
        } else {
            "비활성 — CLI·환경변수 UPBIT_ACCESS_KEY/UPBIT_SECRET_KEY 없음. 자산 조회는 401."
        }
    );

    axum::serve(listener, app).await?;
    Ok(())
}
