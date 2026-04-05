use std::time::Duration;

use reqwest::Client;

use serde_json::Value;

use crate::upbit_auth::{issue_upbit_jwt, post_body_to_query_string_for_jwt};
use crate::upbit_model::{
    UpbitMinuteCandle, UpbitMyBalance, UpbitOrderBook, UpbitOrderRequest, UpbitPairQuote,
};

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

fn minute_candle_url(base: &str, unit: i32, market: String, count: i32) -> String {

    let base = base.trim_end_matches('/');
    format!("{}/v1/candles/minutes/{}?market={}&count={}", base, unit, market, count)
}

fn my_balance_url(base: &str) -> String {
    let base = base.trim_end_matches('/');
    format!("{}/v1/accounts", base)
}

fn order_url(base: &str) -> String {
    let base = base.trim_end_matches('/');
    format!("{}/v1/orders", base)
}

/// 업비트 public client (시세 정보 조회 등)
#[derive(Clone)]
pub struct UpbitPublicClient {
    client: Client,
    url: String,
    /// Exchange API용 Access Key. `secret_key`와 쌍으로 JWT 생성.
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

/// API 에러 타입
#[derive(Debug, thiserror::Error)]
pub enum UpbitError {
    #[error("HTTP 요청 실패: {0}")]
    Request(#[from] reqwest::Error),
    #[error("마켓 목록이 비어있습니다")]
    EmptyMarkets,
    #[error(
        "Access Key / Secret Key가 이 서버 프로세스에 없습니다. \
         `cargo run -- --access_key ... --secret_key ...` 또는 UPBIT_ACCESS_KEY/UPBIT_SECRET_KEY로 기동했는지 확인하세요. \
         (IDE에서 Run만 누르면 인자가 빠지는 경우가 많습니다.)"
    )]
    AuthorizationError,
    #[error("JWT 발급 실패: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("JSON 직렬화/역직렬화 실패: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("API 에러 (status: {status}): {}", .body.as_deref().unwrap_or(""))]
    ApiError {
        status: reqwest::StatusCode,
        body: Option<String>,
    },
}

impl UpbitPublicClient {
    /// IDE/쉘에서 넘어온 따옴표·BOM·공백 제거. `--access_key '....'` 처럼 **따옴표가 값 안에 포함**되면 업비트가 `invalid_access_key` 반환.
    fn normalize_api_key_value(s: String) -> Option<String> {
        let mut t = s.trim().trim_start_matches('\u{FEFF}').trim().to_string();
        loop {
            if t.len() < 2 {
                break;
            }
            let b = t.as_bytes();
            let strip_quotes = matches!(
                (b[0], b[b.len() - 1]),
                (b'\'', b'\'') | (b'"', b'"')
            );
            if !strip_quotes {
                break;
            }
            t = t[1..t.len() - 1].trim().to_string();
        }
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    }

    /// Upbit API 클라이언트. `access_key`·`secret_key`는 선택; 둘 다 있으면 Exchange API용 JWT를 만듭니다.
    pub fn new(
        access_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<UpbitPublicClient, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        let access_key = access_key.and_then(Self::normalize_api_key_value);
        let secret_key = secret_key.and_then(Self::normalize_api_key_value);

        Ok(Self {
            client,
            url: BASE_URL.to_string(),
            access_key,
            secret_key,
        })
    }

    fn exchange_keys(&self) -> Result<(&str, &str), UpbitError> {
        let access = self
            .access_key
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or(UpbitError::AuthorizationError)?;
        let secret = self
            .secret_key
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or(UpbitError::AuthorizationError)?;
        Ok((access, secret))
    }

    /// Exchange API 요청용 `Authorization` 값. `query_string`은 [업비트 인증 문서](https://docs.upbit.com/kr/reference/auth)의 query_hash 규칙에 맞는 문자열; 없으면 `""`.
    pub fn authorization_header(&self, query_string: &str) -> Result<String, UpbitError> {
        let (access_key, secret_key) = self.exchange_keys()?;
        let jwt = issue_upbit_jwt(access_key, secret_key, query_string)?;
        Ok(format!("Bearer {}", jwt))
    }

    /// 커스텀 베이스 URL(프록시·테스트 등). `base` 끝의 `/`는 무시됨.
    #[cfg(test)]
    pub fn with_base_url(
        base_url: impl Into<String>,
        access_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        Ok(Self {
            client,
            url: base_url.into(),
            access_key,
            secret_key,
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

    /// 업비트 분 캔들 조회
    ///
    /// Parameters
    /// unit: 단위 (1, 3, 5, 10, 15, 30, 60, 240)
    /// market: 캔들 조회 대상 코인 (e.g. KRW-BTC)
    /// count: 조회 대상 캔들 개수
    pub async fn get_minute_candle(&self, unit: i32, market: String, count: i32) -> Result<Vec<UpbitMinuteCandle>, UpbitError> {

        let url = minute_candle_url(&self.url, unit, market, count);

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if !status.is_success() {
            return Err(UpbitError::ApiError {
                status,
                body: Some(response.text().await?),
            })
        }

        let minute_candles: Vec<UpbitMinuteCandle> = response.json().await?;
        Ok(minute_candles)
    }

    /// 내 자산현황 조회.
    pub async fn get_my_balance(&self) -> Result<Vec<UpbitMyBalance>, UpbitError> {
        let url = my_balance_url(&self.url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.authorization_header("")?)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?;
        let status = response.status();

        if !status.is_success() {
            return Err(UpbitError::ApiError {
                status,
                body: Some(response.text().await?),
            })
        }

        let my_balance: Vec<UpbitMyBalance> = response.json().await?;
        Ok(my_balance)
    }

    /// 주문 생성 — https://docs.upbit.com/kr/reference/new-order
    pub async fn order(&self, request: UpbitOrderRequest) -> Result<Value, UpbitError> {
        let body_value = serde_json::to_value(&request)?;
        let qs = post_body_to_query_string_for_jwt(&body_value);
        let body_str = serde_json::to_string(&body_value)?;
        let url = order_url(&self.url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.authorization_header(&qs)?)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "application/json")
            .body(body_str)
            .send()
            .await?;
        let status = response.status();

        if !status.is_success() {
            return Err(UpbitError::ApiError {
                status,
                body: Some(response.text().await?),
            });
        }

        Ok(response.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upbit_model::Change;

    #[test]
    fn normalize_strips_wrapping_quotes_via_new() {
        let c = UpbitPublicClient::new(
            Some("  'my_access'  ".to_string()),
            Some("\"sec_x\"".to_string()),
        )
        .unwrap();
        assert_eq!(c.access_key.as_deref(), Some("my_access"));
        assert_eq!(c.secret_key.as_deref(), Some("sec_x"));
    }
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

        let client = UpbitPublicClient::with_base_url(server.uri(), None, None).unwrap();
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

        let client = UpbitPublicClient::with_base_url(server.uri(), None, None).unwrap();
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
