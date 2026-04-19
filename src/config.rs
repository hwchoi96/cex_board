//! `config.toml` 로딩 — 루트에 `[Upbit]` 테이블.

use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default, rename = "Upbit")]
    pub upbit: Option<UpbitSection>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpbitSection {
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub secret_key: Option<String>,
}

/// `path`에 파일이 없으면 기본 설정(키 없음)을 반환합니다.
pub fn load_config(path: &Path) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(AppConfig::default());
        }
        Err(e) => return Err(e.into()),
    };
    if raw.trim().is_empty() {
        return Ok(AppConfig::default());
    }
    let cfg: AppConfig = toml::from_str(&raw)?;
    Ok(cfg)
}
