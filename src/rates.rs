use anyhow::{anyhow, Result};
use reqwest;
use std::collections::HashMap;

#[tracing::instrument]
async fn request_rate_from_binance(coin: &str) -> Result<f64> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}USDT",
        coin
    );
    let data = reqwest::get(url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    let rate = data["price"].parse::<f64>()?;
    Ok(rate)
}

#[tracing::instrument]
async fn request_rate_from_coingecko(coin: &str) -> Result<f64> {
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        coin
    );
    let data: serde_json::Value = reqwest::get(url).await?.json().await?;
    let price = data[coin]["usd"]
        .as_f64()
        .ok_or(anyhow!("Could not convert JSON to f64"))?;
    Ok(price)
}

#[tracing::instrument]
async fn request_rate_from_coingecko_with_24hr_change(coin: &str) -> Result<(f64, f64)> {
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true",
        coin
    );
    let data: serde_json::Value = reqwest::get(url).await?.json().await?;
    let price = data[coin]["usd"]
        .as_f64()
        .ok_or(anyhow!("Could not convert JSON to f64"))?;
    let change = data[coin]["usd_24h_change"]
        .as_f64()
        .ok_or(anyhow!("Could not convert JSON to f64"))?;
    Ok((price, change))
}

#[tracing::instrument]
pub async fn get_btc_rate() -> Result<f64> {
    request_rate_from_binance("BTC").await
}

#[tracing::instrument]
pub async fn get_btc_rate_with_24hr_change() -> Result<(f64, f64)> {
    request_rate_from_coingecko_with_24hr_change("bitcoin").await
}

#[tracing::instrument]
pub async fn get_eth_rate() -> Result<f64> {
    request_rate_from_binance("ETH").await
}

#[tracing::instrument]
pub async fn get_eth_rate_with_24hr_change() -> Result<(f64, f64)> {
    request_rate_from_coingecko_with_24hr_change("ethereum").await
}

#[tracing::instrument]
pub async fn get_ltc_rate() -> Result<f64> {
    request_rate_from_binance("LTC").await
}

#[tracing::instrument]
pub async fn get_etc_rate() -> Result<f64> {
    request_rate_from_binance("ETC").await
}

#[tracing::instrument]
pub async fn get_bch_rate() -> Result<f64> {
    request_rate_from_coingecko("bitcoin-cash").await
}

#[tracing::instrument]
pub async fn get_ada_rate() -> Result<f64> {
    request_rate_from_coingecko("cardano").await
}

#[tracing::instrument]
pub async fn get_zee_rate() -> Result<f64> {
    request_rate_from_coingecko("zeroswap").await
}

#[tracing::instrument]
pub async fn get_zee_rate_with_24hr_change() -> Result<(f64, f64)> {
    request_rate_from_coingecko_with_24hr_change("zeroswap").await
}
