//! This is a bunch of AI generated code just to stress the compiler
//! There is no meaning behind it.

#![allow(dead_code, unused)]

use anyhow::Context;
use clap::Parser;
use diesel::Connection as _;
use diesel_migrations;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "cargo-cache-testing")]
struct Args {
    #[arg(long, default_value = "https://example.com")]
    url: String,
    #[arg(long, default_value_t = 4)]
    jobs: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    id: uuid::Uuid,
    stamp: chrono::DateTime<chrono::Utc>,
    digest: String,
    data: Vec<u64>,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("bad url: {0}")]
    BadUrl(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait::async_trait]
trait Hasher {
    async fn hash(&self, input: &[u8]) -> String;
}

struct B64Hasher;

#[async_trait::async_trait]
impl Hasher for B64Hasher {
    async fn hash(&self, input: &[u8]) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, input)
    }
}

static WORD_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\w+").unwrap());

fn compress(input: &[u8]) -> anyhow::Result<Vec<u8>> {
    use std::io::Write as _;
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    enc.write_all(input)?;
    Ok(enc.finish()?)
}

fn parallel_sum(data: &[u64]) -> u64 {
    use rayon::prelude::*;
    data.par_iter().copied().sum()
}

async fn fetch_head(client: &reqwest::Client, url: &url::Url) -> anyhow::Result<http::StatusCode> {
    let resp = client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("request to {url} failed"))?;
    Ok(http::StatusCode::from_u16(resp.status().as_u16())?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let args = Args::parse();
    let parsed = url::Url::parse(&args.url).map_err(|_| AppError::BadUrl(args.url.clone()))?;
    tracing::info!(url = %parsed, jobs = args.jobs, "starting");

    let data: Vec<u64> = (0..10_000).collect();
    let sum = tokio::task::spawn_blocking(move || parallel_sum(&data)).await?;
    tracing::info!(sum, "parallel sum done");

    let record = Record {
        id: uuid::Uuid::new_v4(),
        stamp: chrono::Utc::now(),
        digest: B64Hasher.hash(b"hello").await,
        data: vec![rand::random::<u64>(), sum],
    };
    let json = serde_json::to_string_pretty(&record)?;
    let words = WORD_RE.find_iter(&json).count();
    let packed = compress(json.as_bytes())?;
    tracing::info!(words, bytes = packed.len(), "record packed");

    let buf = bytes::Bytes::from(packed);
    let client = reqwest::Client::new();
    match fetch_head(&client, &parsed).await {
        Ok(status) => tracing::info!(%status, "fetched"),
        Err(e) => tracing::warn!(error = %e, "fetch skipped"),
    }

    let _ = axum::http::StatusCode::OK;
    let _ = sqlx::sqlite::SqliteConnectOptions::new();
    let _ = diesel::sqlite::SqliteConnection::establish(":memory:").is_ok();
    let _ =
        rsa::pkcs8::DecodePrivateKey::from_pkcs8_der as fn(&[u8]) -> Result<rsa::RsaPrivateKey, _>;
    let _ = jsonwebtoken::Algorithm::HS256;
    let _ = argon2::Argon2::default;
    let values: Vec<u64> = futures::future::join_all((0..8).map(|i| async move { i * i }))
        .await
        .into_iter()
        .collect();
    println!("ok bytes={} values={values:?}", buf.len());
    Ok(())
}
