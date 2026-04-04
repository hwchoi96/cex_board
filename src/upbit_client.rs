use std::time::Duration;
use reqwest::Client;
use crate::upbit_model::{UpbitOrderBook, UpbitPairQuote};

/// 업비트 open api base url
const BASE_URL: &str = "https://api.upbit.com";

fn ticker_url(base: &str, markets: &[&str]) -> String {
    let base = base.trim_end_matches('/');
    format!("{}/v1/ticker?markets={}", base, markets.join(","))
}

fn order_book_url(base: &str, markets: &[&str]) -> String {
    let base = base.trim_end_matches('/');
    format!("{}/v1/orderbook?markets={}&count=10", base, markets.join(","))
    // 10호가만 모아보기
}

/// 업비트 public client (시세 정보 조회 등)
#[derive(Clone)]
pub struct UpbitPublicClient {
    client: Client,
    url: String,
}

/// API 에러 타입
#[derive(Debug, thiserror::Error)]
pub enum UpbitError {
    #[error("HTTP 요청 실패: {0}")]
    Request(#[from] reqwest::Error),
    #[error("마켓 목록이 비어있습니다")]
    EmptyMarkets,
    #[error("API 에러 (status: {status}): {}", .body.as_deref().unwrap_or(""))]
    ApiError {
        status: reqwest::StatusCode,
        body: Option<String>,
    },
}

impl UpbitPublicClient {
    /// Upbit public api client constructor.
    pub fn new() -> Result<UpbitPublicClient, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        Ok(Self {
            client,
            url: BASE_URL.to_string(),
        })
    }

    /// 커스텀 베이스 URL(프록시·테스트 등). `base` 끝의 `/`는 무시됨.
    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        Ok(Self {
            client,
            url: base_url.into(),
        })
    }

    /// 업비트 마켓-페어 시세 정보 조회
    pub async fn get_quote_all(&self, pair: &[&str]) -> Result<Vec<UpbitPairQuote>, UpbitError> {
        let url = ticker_url(&self.url, pair);

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await?;

            return Err(UpbitError::ApiError {
                status,
                body: Some(body),
            });
        }

        let markets: Vec<UpbitPairQuote> = response.json().await?;

        Ok(markets)
    }

    /// 업베트 마켓-페어 오더북 정보 조회
    pub async fn get_order_book(&self, pair: &[&str]) -> Result<Vec<UpbitOrderBook>, UpbitError> {

        if pair.is_empty() {
            return Err(UpbitError::EmptyMarkets);
        }

        let url = order_book_url(&self.url, pair);

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await?;

            return Err(UpbitError::ApiError {
                status,
                body: Some(body),
            });
        }

        let order_books: Vec<UpbitOrderBook> = response.json().await?;
        Ok(order_books)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upbit_model::Change;
    use rust_decimal::Decimal;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn ticker_url_joins_base_and_markets() {
        assert_eq!(
            ticker_url("https://api.upbit.com", &["KRW-BTC", "KRW-ETH"]),
            "https://api.upbit.com/v1/ticker?markets=KRW-BTC,KRW-ETH"
        );
    }

    #[test]
    fn ticker_url_strips_trailing_slash_on_base() {
        assert_eq!(
            ticker_url("https://example.com/", &["KRW-XRP"]),
            "https://example.com/v1/ticker?markets=KRW-XRP"
        );
    }

    fn sample_ticker_json() -> serde_json::Value {
        serde_json::json!([{
            "market": "KRW-BTC",
            "trade_date": "20230330",
            "trade_time": "120000",
            "trade_date_kst": "20230330",
            "trade_time_kst": "120000",
            "trade_timestamp": 1680172800000_i64,
            "opening_price": 50_000_000.0,
            "high_price": 51_000_000.0,
            "low_price": 49_000_000.0,
            "trade_price": 50_500_000.0,
            "prev_closing_price": 49_500_000.0,
            "change": "RISE",
            "trade_volume": 123.45,
            "timestamp": 1680172800123_i64,
        }])
    }

    #[tokio::test]
    async fn get_quote_all_parses_success_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/ticker"))
            .and(query_param("markets", "KRW-BTC"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_ticker_json()))
            .mount(&server)
            .await;

        let client = UpbitPublicClient::with_base_url(server.uri()).unwrap();
        let quotes = client.get_quote_all(&["KRW-BTC"]).await.unwrap();

        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].market, "KRW-BTC");
        assert_eq!(quotes[0].change, Change::Rise);
        assert_eq!(quotes[0].trade_price, Decimal::from(50_500_000));
    }

    #[tokio::test]
    async fn get_quote_all_maps_http_error_to_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/ticker"))
            .respond_with(
                ResponseTemplate::new(400).set_body_string(r#"{"error":{"message":"bad"}}"#),
            )
            .mount(&server)
            .await;

        let client = UpbitPublicClient::with_base_url(server.uri()).unwrap();
        let err = client
            .get_quote_all(&["KRW-BTC"])
            .await
            .expect_err("expected API error");

        match err {
            UpbitError::ApiError { status, body } => {
                assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
                assert!(body.unwrap().contains("bad"));
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }
}
