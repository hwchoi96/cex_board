use crate::upbit_client::UpbitPublicClient;

mod upbit_client;
mod upbit_model;

const COIN_PAIR: [&str; 6] = [
    "KRW-BTC",
    "KRW-ETH",
    "KRW-USDT",
    "KRW-DOGE",
    "KRW-XRP",
    "KRW-ONG",
];

// 나중에는 각 거래소마다 처리하는 스레드를 다르게 할거임.
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Hello, world!");

    let upbit_client = UpbitPublicClient::new()?;

    let quote = upbit_client.get_quote_all(&COIN_PAIR).await?;

    for q in quote {
        println!("{:?}", q);
    }

    Ok(())
}
