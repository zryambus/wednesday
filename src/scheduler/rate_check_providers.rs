use std::marker::PhantomData;

use crate::cache::{Cache, CachePool, RateCheck};
use crate::rates::{get_bnb_rate, get_btc_rate, get_eth_rate, get_not_rate, get_zee_rate, get_ton_rate};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub(crate) struct CheckProvider<T: Clone> {
    pub cache: Cache,
    phantom_data: PhantomData<T>,
}

impl<T: Clone> From<CachePool> for CheckProvider<T> {
    fn from(pool: CachePool) -> Self {
        Self { cache: Cache::new(pool), phantom_data: PhantomData::default() }
    }
}

#[async_trait]
pub(crate) trait RateCheckProvider {
    async fn get_current_rate(&self) -> anyhow::Result<f64>;
    async fn get_last_rates(&self) -> anyhow::Result<Vec<RateCheck>>;
    async fn add_last_rate(&self, rate: &RateCheck) -> anyhow::Result<()>;
    fn step(&self) -> f64;
    fn coin(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub(crate) struct ETH;
pub(crate) type ETHRateCheckProvider = CheckProvider<ETH>;

#[async_trait]
impl RateCheckProvider for ETHRateCheckProvider {
    async fn get_current_rate(&self) -> anyhow::Result<f64> {
        get_eth_rate().await
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

#[derive(Debug, Clone)]
pub(crate) struct BTC;
pub(crate) type BTCRateCheckProvider = CheckProvider<BTC>;

#[async_trait]
impl RateCheckProvider for BTCRateCheckProvider {
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

#[derive(Debug, Clone)]
pub(crate) struct ZEE;
pub(crate) type ZEERateCheckProvider = CheckProvider<ZEE>;

#[async_trait]
impl RateCheckProvider for ZEERateCheckProvider {
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

#[derive(Debug, Clone)]
pub(crate) struct BNB;
pub(crate) type BNBRateCheckProvider = CheckProvider<BNB>;

#[async_trait]
impl RateCheckProvider for BNBRateCheckProvider {
    async fn get_current_rate(&self) -> anyhow::Result<f64> {
        get_bnb_rate().await
    }

    async fn get_last_rates(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.cache.get_last_bnb_rate().await
    }

    async fn add_last_rate(&self, rate: &RateCheck) -> anyhow::Result<()> {
        self.cache.add_last_bnb_rate(rate).await
    }

    fn step(&self) -> f64 {
        10.
    }

    fn coin(&self) -> &'static str {
        "BNB"
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NOT;
pub(crate) type NOTRateCheckProvider = CheckProvider<NOT>;

#[async_trait]
impl RateCheckProvider for NOTRateCheckProvider {
    async fn get_current_rate(&self) -> anyhow::Result<f64> {
        get_not_rate().await
    }

    async fn get_last_rates(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.cache.get_last_not_rate().await
    }

    async fn add_last_rate(&self, rate: &RateCheck) -> anyhow::Result<()> {
        self.cache.add_last_not_rate(rate).await
    }

    fn step(&self) -> f64 {
        0.001
    }

    fn coin(&self) -> &'static str {
        "NOT"
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TON;
pub(crate) type TONRateCheckProvider = CheckProvider<TON>;

#[async_trait]
impl RateCheckProvider for TONRateCheckProvider {
    async fn get_current_rate(&self) -> anyhow::Result<f64> {
        get_ton_rate().await
    }

    async fn get_last_rates(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.cache.get_last_ton_rate().await
    }

    async fn add_last_rate(&self, rate: &RateCheck) -> anyhow::Result<()> {
        self.cache.add_last_ton_rate(rate).await
    }

    fn step(&self) -> f64 {
        0.1
    }

    fn coin(&self) -> &'static str {
        "TON"
    }
}