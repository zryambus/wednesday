#![feature(async_closure, fn_traits)]

mod bot;
mod cache;
mod config;
mod database;
mod errors;
mod rates;
mod scheduler;
mod toads;

use crate::bot::BotHandlers;
use crate::database::Database;
use anyhow::Result;
use bb8_postgres::tokio_postgres::NoTls;
use std::sync::Arc;
use teloxide;
use tracing_subscriber::{prelude::*, registry::Registry};

async fn try_main(cfg: config::Cfg) -> Result<()> {
    let db_path = cfg.read()?.get_str("db")?;
    let cache_path = cfg.read()?.get_str("cache")?;

    let manager = bb8_postgres::PostgresConnectionManager::new(db_path.parse().unwrap(), NoTls);
    let pool = bb8::Pool::builder().build(manager).await?;

    let cache_manager = bb8_redis::RedisConnectionManager::new(format!("redis://{}", cache_path))?;
    let cache_pool = bb8::Pool::builder().build(cache_manager).await?;

    tracing::debug!("testing database connection...");
    {
        Database::init(pool.clone()).await?;

        {
            let connection = pool.get().await?;
            connection.execute("select $1::TEXT", &[&"WORKS"]).await?;

            let assets = database::SQLInit::get("functions.sql").unwrap();
            let sql = std::str::from_utf8(assets.data.as_ref())?;
            connection.simple_query(sql).await?;
        }

        tracing::debug!("database connection established");
    }

    let token = cfg.read()?.get_str("token")?;
    let bot = teloxide::Bot::new(token);

    let _scheduler = scheduler::Scheduler::new(bot.clone(), pool.clone(), cache_pool.clone());

    let handlers = Arc::new(BotHandlers::new(pool.clone(), cache_pool.clone()));

    bot::repl(bot, handlers, cfg.read()?.get_str("bot_name")?).await;

    Ok(())
}

#[tokio::main]
async fn main() {
    let cfg = config::Cfg::new().expect("Could not initialize config");

    let mut options = sentry::ClientOptions::new();
    options.release = sentry::release_name!();
    options.traces_sample_rate = 0.01;

    #[cfg(debug_assertions)]
    {
        options.debug = true;
    }

    let _guard = sentry::init((cfg.read().unwrap().get_str("sentry_url").unwrap(), options));

    Registry::default()
        .with(sentry::integrations::tracing::layer())
        .try_init()
        .unwrap();

    if let Err(ref e) = try_main(cfg).await {
        sentry::integrations::anyhow::capture_anyhow(e);

        eprintln!("Program finished with error: {}", e);
        e.chain()
            .skip(1)
            .for_each(|cause| eprintln!("because: {}", cause));
        std::process::exit(1);
    }
}
