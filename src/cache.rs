use bb8_redis::redis::{from_redis_value, RedisResult, RedisWrite, Value};
use bb8_redis::{
    bb8,
    redis::{pipe, AsyncCommands, FromRedisValue, ToRedisArgs},
};

use serde::{Deserialize, Serialize};

pub type CachePool = bb8::Pool<bb8_redis::RedisConnectionManager>;
pub type CacheConnection<'a> = bb8::PooledConnection<'a, bb8_redis::RedisConnectionManager>;

#[derive(Debug, Clone)]
pub struct Cache {
    pool: CachePool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RateCheck {
    pub rate: f64,
    pub grow: bool,
}

impl ToRedisArgs for RateCheck {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        let data = serde_cbor::to_vec(self).expect("Could not serialize RateCheck value");
        out.write_arg(&data);
    }
}

impl FromRedisValue for RateCheck {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let data: Vec<u8> = from_redis_value(v)?;
        Ok(serde_cbor::from_slice(&data).expect("Could not serialize RateCheck value"))
    }
}

impl Cache {
    const KEY_BTC_DOMINANCE: &'static str = "BTC_DOMINANCE";
    const KEY_ETH_DOMINANCE: &'static str = "ETH_DOMINANCE";
    const KEY_BTC_LAST_RATE: &'static str = "BTC_LAST_RATE";
    const KEY_ETH_LAST_RATE: &'static str = "ETH_LAST_RATE";
    const KEY_ZEE_LAST_RATE: &'static str = "ZEE_LAST_RATE";
    const KEY_BNB_LAST_RATE: &'static str = "BNB_LAST_RATE";

    pub fn new(pool: CachePool) -> Self {
        Self { pool }
    }

    async fn connection(&self) -> anyhow::Result<CacheConnection<'_>> {
        Ok(self.pool.get().await?)
    }

    pub async fn get_btc_dominance(&self) -> anyhow::Result<Option<f64>> {
        let value: Option<f64> = self
            .connection()
            .await?
            .get(Self::KEY_BTC_DOMINANCE)
            .await?;
        Ok(value)
    }

    pub async fn get_eth_dominance(&self) -> anyhow::Result<Option<f64>> {
        let value: Option<f64> = self
            .connection()
            .await?
            .get(Self::KEY_ETH_DOMINANCE)
            .await?;
        Ok(value)
    }

    pub async fn set_btc_dominance(&self, value: f64) -> anyhow::Result<()> {
        pipe()
            .set(Self::KEY_BTC_DOMINANCE, value)
            .expire(Self::KEY_BTC_DOMINANCE, 20 * 60)
            .query_async(&mut *self.connection().await?)
            .await?;
        Ok(())
    }

    pub async fn set_eth_dominance(&self, value: f64) -> anyhow::Result<()> {
        pipe()
            .set(Self::KEY_ETH_DOMINANCE, value)
            .expire(Self::KEY_ETH_DOMINANCE, 20 * 60)
            .query_async(&mut *self.connection().await?)
            .await?;
        Ok(())
    }

    async fn get_last_rate(&self, key: &'static str) -> anyhow::Result<Vec<RateCheck>> {
        let value: Option<Vec<RateCheck>> = self.connection().await?.lrange(key, 0, -1).await?;

        let value = match value {
            Some(v) => v,
            None => return Ok(vec![]),
        };

        Ok(value)
    }

    async fn add_last_rate(&self, key: &str, value: &RateCheck) -> anyhow::Result<()> {
        let mut connection = self.connection().await?;

        connection.lpush(key, value).await?;

        let len: usize = connection.llen(key).await?;

        if len > 3 {
            connection.ltrim(key, 0, 2).await?;
        }

        Ok(())
    }

    pub async fn get_last_btc_rate(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.get_last_rate(Self::KEY_BTC_LAST_RATE).await
    }

    pub async fn add_last_btc_rate(&self, value: &RateCheck) -> anyhow::Result<()> {
        self.add_last_rate(Self::KEY_BTC_LAST_RATE, value).await
    }

    pub async fn get_last_eth_rate(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.get_last_rate(Self::KEY_ETH_LAST_RATE).await
    }

    pub async fn add_last_eth_rate(&self, value: &RateCheck) -> anyhow::Result<()> {
        self.add_last_rate(Self::KEY_ETH_LAST_RATE, value).await
    }

    pub async fn get_last_zee_rate(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.get_last_rate(Self::KEY_ZEE_LAST_RATE).await
    }

    pub async fn add_last_zee_rate(&self, value: &RateCheck) -> anyhow::Result<()> {
        self.add_last_rate(Self::KEY_ZEE_LAST_RATE, value).await
    }

    pub async fn get_last_bnb_rate(&self) -> anyhow::Result<Vec<RateCheck>> {
        self.get_last_rate(Self::KEY_BNB_LAST_RATE).await
    }

    pub async fn add_last_bnb_rate(&self, value: &RateCheck) -> anyhow::Result<()> {
        self.add_last_rate(Self::KEY_BNB_LAST_RATE, value).await
    }
}
