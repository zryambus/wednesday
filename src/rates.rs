use anyhow::{anyhow, Result};
use reqwest;
use std::collections::HashMap;
use tracing::instrument;

#[instrument]
async fn request_rate_from_binance(coin: &str) -> Result<f64> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}USDT",
        coin
    );

    let mut attempt = 0;

    while attempt < 3 {
        let response = reqwest::get(&url).await;
        let response = if let Err(e) = response {
            tracing::debug!("Error while trying to get data from {}: {}\nAttempt {}", url, e, attempt + 1);
            if e.is_connect() {
                attempt += 1;
                continue;
            }
            return Err(anyhow!(e));
        } else {
            response.unwrap()
        };

        let data = response
            .json::<HashMap<String, String>>()
            .await?;

        let rate = data["price"].parse::<f64>()?;
        return Ok(rate);
    }

    return Err(anyhow!("Too many attempts"));
}

#[instrument]
async fn request_rate_from_coingecko(coin: &str) -> Result<f64> {
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        coin
    );
    let response = reqwest::get(url).await?;
    let response_debug = format!("{:?}", response);

    let json = response.json().await;
    let data: serde_json::Value = if let Ok(data) = json {
        data
    } else {
        tracing::error!("Received response: {}", response_debug);
        return Err(anyhow!(json.unwrap_err()));
    };

    let price = data[coin]["usd"]
        .as_f64()
        .ok_or(anyhow!("Could not convert JSON to f64"))?;
    Ok(price)
}

#[instrument]
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

#[instrument]
async fn request_non_coin_rate(from: &str, to: &str) -> Result<f64> {
    let url = format!("https://cdn.jsdelivr.net/gh/fawazahmed0/currency-api@1/latest/currencies/{}/{}.json", from, to);
    let data: serde_json::Value = reqwest::get(url).await?.json().await?;
    let price = data[to]
        .as_f64()
        .ok_or(anyhow!("Could not convert JSON to f64"))?;
    Ok(price)
}

#[instrument]
pub async fn get_btc_rate() -> Result<f64> {
    request_rate_from_binance("BTC").await
}

#[instrument]
pub async fn get_btc_rate_with_24hr_change() -> Result<(f64, f64)> {
    request_rate_from_coingecko_with_24hr_change("bitcoin").await
}

#[instrument]
pub async fn get_eth_rate() -> Result<f64> {
    request_rate_from_binance("ETH").await
}

#[instrument]
pub async fn get_eth_rate_with_24hr_change() -> Result<(f64, f64)> {
    request_rate_from_coingecko_with_24hr_change("ethereum").await
}

#[instrument]
pub async fn get_ltc_rate() -> Result<f64> {
    request_rate_from_binance("LTC").await
}

#[instrument]
pub async fn get_etc_rate() -> Result<f64> {
    request_rate_from_binance("ETC").await
}

#[instrument]
pub async fn get_ada_rate() -> Result<f64> {
    request_rate_from_coingecko("cardano").await
}

#[instrument]
pub async fn get_zee_rate() -> Result<f64> {
    request_rate_from_coingecko("zeroswap").await
}

#[instrument]
pub async fn get_zee_rate_with_24hr_change() -> Result<(f64, f64)> {
    request_rate_from_coingecko_with_24hr_change("zeroswap").await
}

#[instrument]
pub async fn get_sol_rate_with_24hr_change() -> Result<(f64, f64)> {
    request_rate_from_coingecko_with_24hr_change("solana").await
}

#[instrument]
pub async fn get_bnb_rate_with_24hr_change() -> Result<(f64, f64)> {
    request_rate_from_coingecko_with_24hr_change("binancecoin").await
}

#[instrument]
pub async fn get_luna_rate_with_24hr_change() -> Result<(f64, f64)> {
    request_rate_from_coingecko_with_24hr_change("terra-luna").await
}

#[instrument]
pub async fn get_usd_rate() -> Result<f64> {
    request_non_coin_rate("usd", "rub").await
}