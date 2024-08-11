use anyhow::Result;
use config;

pub struct Cfg {
    pub bot_name: String,
    pub token: String,
    pub sentry_url: String,
    pub coin_market_api_key: String,
    pub db: String,
    pub cache: String,
    pub traces_sample_rate: f32,
    pub admin_user_id: AdminUserId,
}

impl Cfg {
    pub fn new() -> Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .add_source(config::Environment::with_prefix("WEDNESDAY"))
            .build()?;
        Ok(Self {
            bot_name: settings.get_string("bot_name")?,
            token: settings.get_string("token")?,
            sentry_url: settings.get_string("sentry_url")?,
            coin_market_api_key: settings.get_string("coin_market_api_key")?,
            db: settings.get_string("db")?,
            cache: settings.get_string("cache")?,
            traces_sample_rate: settings.get_float("traces_sample_rate")? as f32,
            admin_user_id: AdminUserId(settings.get_int("admin_user_id")?),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AdminUserId(pub i64);
