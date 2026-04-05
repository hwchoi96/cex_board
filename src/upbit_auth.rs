//! 업비트 Exchange API JWT — [인증 가이드](https://docs.upbit.com/kr/reference/auth)
//!
//! `query_string`은 URL 인코딩되지 않은 쿼리 문자열(또는 POST 본문을 쿼리 형태로 가공한 문자열)이어야 합니다.
//! 파라미터가 없는 `GET`(예: `/v1/accounts`)은 빈 문자열을 넘깁니다.

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha512};

#[derive(Serialize)]
struct UpbitJwtClaims {
    access_key: String,
    nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash_alg: Option<&'static str>,
}

/// `HS512`로 서명한 Bearer 토큰(JWT 문자열만, `Bearer ` 접두사 없음).
pub fn issue_upbit_jwt(
    access_key: &str,
    secret_key: &str,
    query_string: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let nonce = uuid::Uuid::new_v4().to_string();

    let (query_hash, query_hash_alg) = if query_string.is_empty() {
        (None, None)
    } else {
        let digest = Sha512::digest(query_string.as_bytes());
        (Some(hex::encode(digest)), Some("SHA512"))
    };

    let claims = UpbitJwtClaims {
        access_key: access_key.to_string(),
        nonce,
        query_hash,
        query_hash_alg,
    };

    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
    header.typ = Some("JWT".to_string());

    jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret_key.as_bytes()),
    )
}

/// POST JSON 본문을 JWT `query_hash`용 쿼리 문자열로 변환합니다. `null` 키는 제외합니다.
/// [new-order](https://docs.upbit.com/kr/reference/new-order) · [인증](https://docs.upbit.com/kr/reference/auth)
pub fn post_body_to_query_string_for_jwt(body: &Value) -> String {
    let Value::Object(map) = body else {
        return String::new();
    };
    map.iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(k, v)| {
            let val = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
                _ => v.to_string(),
            };
            format!("{}={}", k, val)
        })
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    #[test]
    fn jwt_accounts_style_has_no_query_hash() {
        let token = issue_upbit_jwt("test_access", "test_secret_key_bytes", "").unwrap();
        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();
        let data = decode::<serde_json::Value>(
            &token,
            &DecodingKey::from_secret("test_secret_key_bytes".as_bytes()),
            &validation,
        )
        .unwrap();
        let claims = data.claims;
        assert_eq!(claims["access_key"], "test_access");
        assert!(claims.get("query_hash").is_none());
        assert!(claims["nonce"].as_str().unwrap().len() > 0);
    }

    #[test]
    fn jwt_with_query_includes_sha512_hex() {
        let qs = "market=KRW-BTC&limit=10";
        let token = issue_upbit_jwt("ak", "sk_sk_sk_sk_sk_sk", qs).unwrap();
        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();
        let data = decode::<serde_json::Value>(
            &token,
            &DecodingKey::from_secret("sk_sk_sk_sk_sk_sk".as_bytes()),
            &validation,
        )
        .unwrap();
        let claims = data.claims;
        assert_eq!(claims["query_hash_alg"], "SHA512");
        let expected = hex::encode(Sha512::digest(qs.as_bytes()));
        assert_eq!(claims["query_hash"], expected);
    }
}
