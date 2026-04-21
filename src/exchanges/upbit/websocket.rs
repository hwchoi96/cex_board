use crate::exchanges::upbit::{UpbitWsEvent, UpbitWsOrderbook, UpbitWsTicker};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

/// 업비트 웹소켓 정의

/// 공개 WS — 티커·호가(`orderbook`)·체결 등. 비공개 `.../v1/private` 는 JWT 필요.
const UPBIT_PUBLIC_WEBSOCKET_URL: &str = "wss://api.upbit.com/websocket/v1";

const PING_INTERVAL: u64 = 30;

/// 끊긴 뒤 재연결까지 대기: 최초 1초, 최대 60초까지 지수 백오프
const RECONNECT_BASE: Duration = Duration::from_secs(1);
const RECONNECT_MAX: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpbitWebsocketAction {
    Quote,
    /// 오더북 전용 연결 (구독 JSON·파싱·채널 타입은 추후 분리)
    Orderbook,
}

/// 한 번 연결해 끊길 때까지 읽기 (재연결 없음). 상위에서 [`run_websocket_with_reconnect`] 권장.
async fn connect_quote_websocket(
    pairs: Vec<String>,
    sender: Sender<Box<dyn UpbitWsEvent>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 웹소켓 연결.
    let (ws_stream, _) = connect_async(UPBIT_PUBLIC_WEBSOCKET_URL).await?;

    let (mut write, mut read) = ws_stream.split();

    // 시세 정보 조회를 위한, 업비트 명세 준수
    // https://docs.upbit.com/kr/reference/websocket-guide
    let subscribe_info = serde_json::json!([
        { "ticket": Uuid::new_v4().to_string() },
        { "type": "ticker", "codes": pairs} ,
        { "format": "DEFAULT"}
    ]);
    let subscribe_msg = subscribe_info.to_string();

    write
        .send(Message::Text(subscribe_msg))
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    // 핑퐁을 위한, 핑 객체 선언
    let mut ping_interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL));
    ping_interval.tick().await;

    loop {
        tokio::select! {
            next = read.next() => {
                match next {
                    None => break,
                    // 업비트는 티커 JSON을 Text가 아니라 Binary(UTF-8)로 보내는 경우가 많음.
                    Some(Ok(Message::Text(text))) => {
                        try_send_ticker(&sender, &text).await;
                    }
                    Some(Ok(Message::Binary(bin))) => {
                        match std::str::from_utf8(&bin) {
                            Ok(s) => try_send_ticker(&sender, s).await,
                            Err(_) => { /* 잘못된 UTF-8 */ }
                        }
                    }
                    Some(Ok(Message::Ping(p))) => {
                        if let Err(e) = write.send(Message::Pong(p)).await {
                            return Err(e.into());
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        println!("시세 > 퐁 수신");
                    }
                    Some(Ok(Message::Close(_frame))) => {
                        println!("시세 > 업비트 웹소켓 커넥션 종료 신호");
                        break;
                    }
                    Some(Err(e)) => {
                        println!("시세 > 업비트 웹소켓 에러. {}", e);
                        break;
                    }
                    Some(Ok(_)) => {}
                }
            }
            _ = ping_interval.tick() => {
                print!("시세 > 핑 전송");
                write.send(Message::Ping(vec![])).await?;
            }
        }
    }

    Ok(())
}

/// 오더북 WebSocket (공개 엔드포인트 — private URL은 JWT 없으면 401)
async fn connect_orderbook_websocket(
    pairs: Vec<String>,
    sender: Sender<Box<dyn UpbitWsEvent>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (ws_stream, _) = connect_async(UPBIT_PUBLIC_WEBSOCKET_URL).await?;

    let (mut write, mut read) = ws_stream.split();

    // 오더북 정보 조회를 위한, 업비트 명세 준수
    // https://docs.upbit.com/kr/reference/websocket-guide
    let subscribe_info = serde_json::json!([
        { "ticket": Uuid::new_v4().to_string() },
        { "type": "orderbook", "codes": pairs} ,
        { "format": "DEFAULT"}
    ]);
    let subscribe_msg = subscribe_info.to_string();

    write
        .send(Message::Text(subscribe_msg))
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    // 핑퐁을 위한, 핑 객체 선언
    let mut ping_interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL));
    ping_interval.tick().await;

    loop {
        tokio::select! {
            next = read.next() => {
                match next {
                    None => break,
                    // 업비트는 티커 JSON을 Text가 아니라 Binary(UTF-8)로 보내는 경우가 많음.
                    Some(Ok(Message::Text(text))) => {
                        try_send_orderbook(&sender, &text).await;
                    }
                    Some(Ok(Message::Binary(bin))) => {
                        match std::str::from_utf8(&bin) {
                            Ok(s) => try_send_orderbook(&sender, s).await,
                            Err(_) => { /* 잘못된 UTF-8 */ }
                        }
                    }
                    Some(Ok(Message::Ping(p))) => {
                        if let Err(e) = write.send(Message::Pong(p)).await {
                            return Err(e.into());
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        println!("오더북 > 퐁 수신");
                    }
                    Some(Ok(Message::Close(_frame))) => {
                        println!("오더북 > 업비트 웹소켓 커넥션 종료 신호");
                        break;
                    }
                    Some(Err(e)) => {
                        println!("오더북 > 업비트 웹소켓 에러. {}", e);
                        break;
                    }
                    Some(Ok(_)) => {}
                }
            }
            _ = ping_interval.tick() => {
                print!("오더북 > 핑 전송");
                write.send(Message::Ping(vec![])).await?;
            }
        }
    }

    Ok(())
}

/// 네트워크/서버 종료 등으로 세션이 끊기면 **자동으로 다시 연결**합니다.
///
/// - `sender`를 구독하는 `Receiver`가 모두 드롭되면 (`is_closed`) 루프를 빠져 나옵니다.
/// - `connect_quote_websocket`이 `Err`로 끝나면(최초 연결 실패 등) 지수 백오프 후 재시도합니다.
/// - 정상적으로 세션이 끝난 뒤(`Ok`)에는 백오프를 초기화하고 짧은 간격으로 다시 붙습니다.
pub async fn run_websocket_with_reconnect(
    pairs: Vec<String>,
    sender: Sender<Box<dyn UpbitWsEvent>>,
    action: UpbitWebsocketAction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut backoff = RECONNECT_BASE;

    loop {
        if sender.is_closed() {
            return Ok(());
        }

        // 각 arm은 `Result`를 **마지막 표현식으로** 반환 (`;`로 끝나면 `()` 타입이 됨)
        let result = match action {
            UpbitWebsocketAction::Quote => {
                connect_quote_websocket(pairs.clone(), sender.clone()).await
            }
            UpbitWebsocketAction::Orderbook => {
                connect_orderbook_websocket(pairs.clone(), sender.clone()).await
            }
        };

        match &result {
            Ok(()) => {
                // 세션이 끊긴 뒤에는 짧은 간격으로 다시 붙기
                backoff = RECONNECT_BASE;
            }
            Err(e) => {
                eprintln!("업비트 웹소켓 연결/세션 오류: {e}");
            }
        }

        if sender.is_closed() {
            return Ok(());
        }

        tokio::time::sleep(backoff).await;

        if result.is_err() {
            backoff = (backoff * 2).min(RECONNECT_MAX);
        }
    }
}

async fn try_send_ticker(tx: &mpsc::Sender<Box<dyn UpbitWsEvent>>, json: &str) {
    match serde_json::from_str::<UpbitWsTicker>(json) {
        Ok(t) => {
            if tx.send(Box::new(t)).await.is_err() {
                // 수신 쪽이 모두 드롭됨
            }
        }
        Err(_) => {
            // 구독 응답·에러 JSON 등 ticker가 아닌 메시지는 여기로 옴 → 무시/로그
        }
    }
}

async fn try_send_orderbook(tx: &mpsc::Sender<Box<dyn UpbitWsEvent>>, json: &str) {
    match serde_json::from_str::<UpbitWsOrderbook>(json) {
        Ok(t) => {
            if tx.send(Box::new(t)).await.is_err() {
                // 수신 쪽이 모두 드롭됨
            }
        }
        Err(_) => {
            // 구독 응답·에러 JSON 등 ticker가 아닌 메시지는 여기로 옴 → 무시/로그
        }
    }
}
