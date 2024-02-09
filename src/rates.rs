use anyhow::{anyhow, Result};
use serde::Deserialize;
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
            tracing::warn!(
                "Error while trying to get data from {}: {}\nAttempt {}",
                url,
                e,
                attempt + 1
            );
            if e.is_connect() {
                attempt += 1;
                continue;
            }
            return Err(anyhow!(e));
        } else {
            response.unwrap()
        };

        let json = response.json::<serde_json::Value>().await?;
        let json_object = match json.as_object() {
            Some(json_object) => json_object,
            None => return Err(anyhow!("Unexpected format of response: {}", json)) 
        };

        let price = match json_object.get("price") {
            Some(price) => price,
            None => return Err(anyhow!("There is no `price` field in response object: {}", json)) 
        };
        
        let price = match price.as_str() {
            Some(price) => price,
            None => return Err(anyhow!("Failed to get `price` value as string in response object: {}", json))
        };
        
        let rate = price.parse::<f64>()
            .map_err(|_| anyhow!("Failed to parse `price` value as a f64: {}", price))?;
        return Ok(rate);
    }

    return Err(anyhow!("Too many attempts"));
}

#[derive(Debug, Deserialize)]
struct CoingeckErrorStatus {
    pub error_code: usize,
    pub error_message: String,
}

#[derive(Debug, Deserialize)]
struct CoingeckError {
    pub status: CoingeckErrorStatus
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
    let data: serde_json::Value = match json {
        Ok(data) => data,
        Err(err) => {
            tracing::error!("Received response: {}", response_debug);
            return Err(anyhow!(err));
        }
    };

    if let Ok(error) = serde_json::from_value::<CoingeckError>(data.clone()) {
        let CoingeckErrorStatus { error_code, error_message } = error.status;
        return Err(anyhow!("Coin `{}` request finished with error code: {}. Cause: {}", coin, error_code, error_message));
    } 

    let coin_value = match data.get(coin) {
        Some(value) => value,
        None => {
            return Err(anyhow!("JSON object doesn't contain `{}` data field: {}", coin, data));
        }
    };

    let price_value = match coin_value.get("usd") {
        Some(value) => value,
        None => {
            return Err(anyhow!("Coin object doesn't contain `usd` data field: {}", data));
        }
    };

    let price = price_value
        .as_f64()
        .ok_or(anyhow!("Could not convert JSON to f64: {}", price_value))?;

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
        .ok_or(anyhow!("Could not convert JSON `{coin}.usd` to f64: {}", data))?;
    let change = data[coin]["usd_24h_change"]
        .as_f64()
        .ok_or(anyhow!("Could not convert JSON `{coin}.usd_24h_change` to f64: {}", data))?;
    Ok((price, change))
}

#[instrument]
async fn request_non_coin_rate(from: &str, to: &str) -> Result<f64> {
    let url = format!(
        "https://cdn.jsdelivr.net/gh/fawazahmed0/currency-api@1/latest/currencies/{}/{}.json",
        from, to
    );
    let data: serde_json::Value = reqwest::get(url).await?.json().await?;
    let price = data[to]
        .as_f64()
        .ok_or(anyhow!("Could not convert JSON `to` to f64: {}", data))?;
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
pub async fn get_sol_rate() -> Result<f64> {
    request_rate_from_binance("SOL").await
}

#[instrument]
pub async fn get_etc_rate() -> Result<f64> {
    request_rate_from_binance("ETC").await
}

#[instrument]
pub async fn get_bnb_rate() -> Result<f64> {
    request_rate_from_binance("BNB").await
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
