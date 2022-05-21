mod bot;
mod cache;
mod config;
mod database;
mod errors;
mod rates;
mod scheduler;
mod toads;

use std::sync::{Arc, RwLock};

use crate::{bot::Gauss, database::Database};

use anyhow::Result;
use bb8_postgres::tokio_postgres::NoTls;
use teloxide::prelude::*;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, registry::Registry};

async fn try_main(cfg: config::Cfg) -> Result<()> {
    let db_path = cfg.db()?;
    let cache_path = cfg.cache()?;

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

    let token = cfg.token()?;
    let bot = teloxide::Bot::new(token);

    let _scheduler = scheduler::Scheduler::new(bot.clone(), pool.clone(), cache_pool.clone());

    Dispatcher::builder(bot, crate::bot::get_handler())
        .dependencies(dptree::deps![
            pool.clone(),
            cache_pool.clone(),
            cfg.bot_name()?,
            Arc::new(RwLock::new(Gauss::new(17., 4.)))
        ])
        .default_handler(|upd| async move {
            tracing::warn!("Unhandled update: {:?}", upd);
        })
        // If the dispatcher fails for some reason, execute this handler.
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;

    Ok(())
}

#[tokio::main]
async fn main() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        hook(info);
        std::process::exit(1);
    }));

    let cfg = config::Cfg::new().expect("Could not initialize config");

    let mut options = sentry::ClientOptions::new();
    options.release = sentry::release_name!();
    options.traces_sample_rate = cfg.traces_sample_rate()
        .expect("Could not read traces_sample_rate value from config");

    #[cfg(debug_assertions)]
    {
        options.debug = true;
    }

    let _guard = sentry::init((cfg.sentry_url().unwrap(), options));

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_filter(LevelFilter::INFO);

    Registry::default()
        .with(sentry::integrations::tracing::layer())
        .with(fmt_layer)
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

#[cfg(test)]
mod tests {
    use crate::bot::Gauss;

    #[test]
    fn gauss_test() {
        let mut gauss = Gauss::new(17., 4.);
        for _ in 0..12 {
            println!("{}", gauss.next());
        }
    }
}
