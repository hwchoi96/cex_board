//! CEX Board HTTP 서버 엔트리.
//! `http://127.0.0.1:9090` — 대시보드 HTML 및 거래소 API.

use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use crate::exchanges::upbit::UpbitPublicClient;
use crate::web::{create_router, AppState};

mod config;
mod constants;
mod exchanges;
mod web;

/// 웹 대시보드·API 바인드 포트
const HTTP_PORT: u16 = 9090;

#[derive(Parser)]
#[command(name = "cex_board")]
struct Args {
    /// 설정 파일 경로 (기본: `config.toml`, 워킹 디렉터리 기준).
    #[arg(long = "config", default_value = "config.toml", value_name = "PATH")]
    config_path: std::path::PathBuf,
    /// `config.toml`의 Upbit access_key를 덮어씁니다.
    #[arg(long = "access_key", value_name = "ACCESS_KEY")]
    access_key: Option<String>,
    /// `config.toml`의 Upbit secret_key를 덮어씁니다. `=` 형식 권장: `--secret_key=...`
    #[arg(long = "secret_key", value_name = "SECRET_KEY")]
    secret_key: Option<String>,
}

fn merge_key(cli: Option<String>, file: Option<String>) -> Option<String> {
    cli.or(file).filter(|s| !s.trim().is_empty())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let cfg = crate::config::load_config(&args.config_path)?;
    if !args.config_path.exists() {
        eprintln!(
            "[cex_board] 설정 파일 없음: {:?} — [Upbit] 키 없이 기동합니다.",
            args.config_path
        );
    }
    let upbit_cfg = cfg.upbit.unwrap_or_default();
    let access_key = merge_key(args.access_key, upbit_cfg.access_key);
    let secret_key = merge_key(args.secret_key, upbit_cfg.secret_key);

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
            "비활성 — config.toml [Upbit] access_key/secret_key 없음. 자산 조회는 401."
        }
    );

    axum::serve(listener, app).await?;
    Ok(())
}
