use crate::cache::{Cache, CachePool, RateCheck};
use crate::rates::{get_btc_rate, get_etc_rate, get_zee_rate};
use async_trait::async_trait;

#[async_trait]
pub(crate) trait RateCheckProvider {
    async fn get_current_rate(&self) -> anyhow::Result<f64>;
    async fn get_last_rates(&self) -> anyhow::Result<Vec<RateCheck>>;
    async fn add_last_rate(&self, rate: &RateCheck) -> anyhow::Result<()>;
    fn step(&self) -> f64;
    fn coin(&self) -> &'static str;
}

pub(crate) struct ETHCheckProvider {
    cache: Cache,
}

impl ETHCheckProvider {
    pub fn new(cache_pool: CachePool) -> Self {
        Self {
            cache: Cache::new(cache_pool),
        }
    }
}

#[async_trait]
impl RateCheckProvider for ETHCheckProvider {
    async fn get_current_rate(&self) -> anyhow::Result<f64> {
        get_etc_rate().await
    }

    async fn get_last_rates(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.cache.get_last_eth_rate().await
    }

    async fn add_last_rate(&self, rate: &RateCheck) -> anyhow::Result<()> {
        self.cache.add_last_eth_rate(rate).await
    }

    fn step(&self) -> f64 {
        100.
    }

    fn coin(&self) -> &'static str {
        "ETH"
    }
}

pub(crate) struct BTCCheckProvider {
    cache: Cache,
}

impl BTCCheckProvider {
    pub fn new(cache_pool: CachePool) -> Self {
        Self {
            cache: Cache::new(cache_pool),
        }
    }
}

#[async_trait]
impl RateCheckProvider for BTCCheckProvider {
    async fn get_current_rate(&self) -> anyhow::Result<f64> {
        get_btc_rate().await
    }

    async fn get_last_rates(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.cache.get_last_btc_rate().await
    }

    async fn add_last_rate(&self, rate: &RateCheck) -> anyhow::Result<()> {
        self.cache.add_last_btc_rate(rate).await
    }

    fn step(&self) -> f64 {
        1000.
    }

    fn coin(&self) -> &'static str {
        "BTC"
    }
}

pub(crate) struct ZEECheckProvider {
    cache: Cache,
}

impl ZEECheckProvider {
    pub fn new(cache_pool: CachePool) -> Self {
        Self {
            cache: Cache::new(cache_pool),
        }
    }
}

#[async_trait]
impl RateCheckProvider for ZEECheckProvider {
    async fn get_current_rate(&self) -> anyhow::Result<f64> {
        get_zee_rate().await
    }

    async fn get_last_rates(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.cache.get_last_zee_rate().await
    }

    async fn add_last_rate(&self, rate: &RateCheck) -> anyhow::Result<()> {
        self.cache.add_last_zee_rate(rate).await
    }

    fn step(&self) -> f64 {
        0.01
    }

    fn coin(&self) -> &'static str {
        "ZEE"
    }
}
