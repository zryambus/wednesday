use crate::cache::{Cache, CachePool};
use crate::database::{Database, Pool, UpdateKind};
use crate::rates;

use anyhow::{Result, Error, anyhow};
use futures::{try_join};
use serde::Deserialize;
use teloxide::{
    prelude2::*, types::{Sticker, User, Update},
    utils::command::BotCommand
};
use tracing::instrument;

mod wednesday;

use wednesday::WednesdayBot;

#[derive(BotCommand, Clone, Debug)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start send toads")]
    Start,
    #[command(description = "stop send toads")]
    Stop,
    #[command(description = "show status of this chat")]
    Status,
    #[command(description = "start send BTC rate")]
    WhenLambo,
    #[command(description = "stop send BTC rate")]
    NotToday,
    #[command(description = "show status of this crypto chat")]
    Stonks,
    #[command(description = "show rates of BTC, ETH and LTC to USD")]
    Rates,
    #[command(description = "show rate of BTC to USD")]
    BTC,
    #[command(description = "show rate of ETH to USD")]
    ETH,
    #[command(description = "show rate of LTC to USD")]
    LTC,
    #[command(description = "show rate of ETC to USD")]
    ETC,
    #[command(description = "show rate of SOL to USD")]
    SOL,
    #[command(description = "show rate of ADA to USD")]
    ADA,
    #[command(description = "show rate of ZEE to USD")]
    ZEE,
    #[command(description = "show rate of BNB to USD")]
    BNB,
    #[command(description = "show rate of LUNA to USD")]
    LUNA,
    #[command(description = "show statistics")]
    Stats,
    #[command(description = "show BTC and ETH dominance")]
    Dominance,
    #[command(description = "show USD rate")]
    USD,
    #[command(description = "cast all in members in chat")]
    All
}

pub async fn commands_endpoint(bot: Bot, msg: Message, command: Command, pool: Pool, cache_pool: CachePool) -> Result<()> {
    let db = Database::new(pool.clone()).await?;

    update_users_mapping(&bot, msg.from(), pool.clone()).await?;

    match command {
        Command::Help => {
            bot.send_message(msg.chat_id(), Command::descriptions())
                .send()
                .await?;
        }
        Command::Start => {
            on_start(bot, msg, db).await?;
        }
        Command::Stop => {
            on_stop(bot, msg, db).await?;
        }
        Command::Status => {
            on_status(bot, msg, db).await?;
        }
        Command::WhenLambo => on_crypto_start(bot, msg, db).await?,
        Command::NotToday => on_crypto_stop(bot, msg, db).await?,
        Command::Stonks => on_crypto_status(bot, msg, db).await?,
        Command::Rates => on_rates(bot, msg).await?,
        Command::BTC => {
            on_coin_with_24hr_change(bot, msg, "BTC", &rates::get_btc_rate_with_24hr_change)
                .await?
        }
        Command::ETH => {
            on_coin_with_24hr_change(bot, msg, "ETH", &rates::get_eth_rate_with_24hr_change)
                .await?
        }
        Command::LTC => {
            on_coin(bot, msg, "LTC", &rates::get_ltc_rate)
                .await?
        }
        Command::ETC => {
            on_coin(bot, msg, "ETC", &rates::get_etc_rate)
                .await?
        }
        Command::SOL => {
            on_coin_with_24hr_change(bot, msg, "SOL", &rates::get_sol_rate_with_24hr_change)
                .await?
        }
        Command::ADA => {
            on_coin(bot, msg, "ADA", &rates::get_ada_rate)
                .await?
        }
        Command::ZEE => {
            on_coin_with_24hr_change(bot, msg, "ZEE", &rates::get_zee_rate_with_24hr_change)
                .await?
        }
        Command::BNB => {
            on_coin_with_24hr_change(bot, msg, "BNB", &rates::get_bnb_rate_with_24hr_change)
                .await?
        }
        Command::LUNA => {
            on_coin_with_24hr_change(bot, msg, "LUNA", &rates::get_luna_rate_with_24hr_change)
                .await?
        }
        Command::Stats => on_stats(bot, msg, db).await?,
        Command::Dominance => {
            on_dominance(bot, msg, cache_pool.clone())
                .await?
        },
        Command::USD => {
            on_usd(bot, msg).await?
        },
        Command::All => {
            on_all(bot, msg).await?
        },
    };

    Ok(())
}

#[instrument(skip(db))]
pub async fn on_start(bot: Bot, msg: Message, db: Database) -> Result<()> {
    if !db.is_active(msg.chat_id()).await? {
        db.add(msg.chat_id()).await?;
        bot
            .send_message(msg.chat_id(), "✅ Chat was added to list")
            .send()
            .await?;
    } else {
        bot
            .send_message(msg.chat_id(), "⚠ Current chat is already in the list")
            .send()
            .await?;
    }

    Ok(())
}

#[instrument(skip(db))]
pub async fn on_stop(bot: Bot, msg: Message, db: Database) -> Result<()> {
    if db.is_active(msg.chat_id()).await? {
        db.remove(msg.chat_id()).await?;
        bot
            .send_message(msg.chat_id(), "✅ Chat was removed from list")
            .send()
            .await?;
    } else {
        bot
            .send_message(msg.chat_id(), "⚠ Current chat is not in the list")
            .send()
            .await?;
    }
    Ok(())
}

#[instrument(skip(db))]
pub async fn on_status(bot: Bot, msg: Message, db: Database) -> Result<()> {
    let active = db.is_active(msg.chat_id()).await?;
    let text = format!(
        "Current chat is {}",
        if active {
            "in the list ✅"
        } else {
            "not in the list ❌"
        }
    );
    bot.send_message(msg.chat_id(), text).send().await?;
    Ok(())
}

#[instrument(skip(db))]
pub async fn on_crypto_start(bot: Bot, msg: Message, db: Database) -> Result<()> {
    if !db.is_active_crypto(msg.chat_id()).await? {
        db.add_crypto(msg.chat_id()).await?;
        bot
            .send_message(msg.chat_id(), "✅ Chat was added to crypto list")
            .send()
            .await?;
    } else {
        bot
            .send_message(msg.chat_id(), "⚠ Current chat is already in the crypto list")
            .send()
            .await?;
    }

    Ok(())
}

#[instrument(skip(db))]
pub async fn on_crypto_stop(bot: Bot, msg: Message, db: Database) -> Result<()> {
    if db.is_active_crypto(msg.chat_id()).await? {
        db.remove_crypto(msg.chat_id()).await?;
        bot
            .send_message(msg.chat_id(), "✅ Chat was removed from crypto list")
            .send()
            .await?;
    } else {
        bot
            .send_message(msg.chat_id(), "⚠ Current chat is not in the crypto list")
            .send()
            .await?;
    }
    Ok(())
}

#[instrument(skip(db))]
pub async fn on_crypto_status(bot: Bot, msg: Message, db: Database) -> Result<()> {
    let active = db.is_active_crypto(msg.chat_id()).await?;
    let text = format!(
        "Current crypto chat is {}",
        if active {
            "in the list ✅"
        } else {
            "not in the list ❌"
        }
    );
    bot.send_message(msg.chat_id(), text).send().await?;
    Ok(())
}

#[instrument]
pub async fn on_rates(bot: Bot, msg: Message ) -> Result<()> {
    let ref bot = bot;
    let chat = msg.chat_id();

    let (btc, eth, zee) = try_join!(
        rates::get_btc_rate(),
        rates::get_eth_rate(),
        rates::get_zee_rate(),
    )?;

    let text = format!("BTC = {}$\nETH = {}$\nZEE = {}$", btc, eth, zee);

    bot.send_message(chat, &text).send().await?;
    Ok(())
}

#[instrument(skip(callback), fields(error))]
async fn on_coin<Fut>(
    bot: Bot, msg: Message,
    coin: &str,
    callback: impl FnOnce() -> Fut,
) -> Result<()>
where
    Fut: std::future::Future<Output = Result<f64>> + Send,
{
    let bot = WednesdayBot::new(bot, msg);
    let chat = bot.chat_id();

    let rate = callback().await?;

    let text = format!("Курс {} = {}$", coin, rate);

    bot.send_text(chat, text).await?;
    Ok(())
}

#[instrument(skip(callback))]
async fn on_coin_with_24hr_change<Fut>(
    bot: Bot, msg: Message,
    coin: &str,
    callback: impl FnOnce() -> Fut,
) -> Result<()>
where
    Fut: std::future::Future<Output = Result<(f64, f64)>> + Send,
{
    let ref bot = bot;
    let chat = msg.chat_id();

    let (rate, change) = callback().await?;

    let text = format!("Курс {} = {}$ ({:.2}%)", coin, rate, change);

    bot.send_message(chat, &text).send().await?;
    Ok(())
}

#[instrument(skip(db))]
async fn on_stats(bot: Bot, msg: Message, db: Database) -> Result<()> {
    let user_id = msg.from().unwrap().id;
    let chat_id = msg.chat.id;
    let stats = db.get_statistics(chat_id, user_id).await?;

    let mut text = format!("Your statistics for today:\n");
    for (kind, count) in stats {
        match kind {
            UpdateKind::TextMessage => {
                text.push_str(&format!("Messages: {}\n", count));
            }
            UpdateKind::Sticker => {
                text.push_str(&format!("Sticker: {}\n", count));
            }
            UpdateKind::ForwardedMeme => {
                text.push_str(&format!("Memes: {}\n", count));
            }
        }
    }

    bot.send_message(msg.chat_id(), text).send().await?;

    Ok(())
}

#[instrument]
pub async fn on_dominance(bot: Bot, msg: Message, cache_pool: CachePool) -> Result<(), Error> {
    let cache = Cache::new(cache_pool);
    let (cached_btc, cached_eth) = try_join!(cache.get_btc_dominance(), cache.get_eth_dominance())?;

    async fn request_dominance(cache: &Cache) -> Result<(f64, f64)> {
        let url = "https://pro-api.coinmarketcap.com/v1/global-metrics/quotes/latest";
        let cfg = crate::config::Cfg::new()?;
        let api_key = cfg.coin_market_api_key()?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-CMC_PRO_API_KEY",
            reqwest::header::HeaderValue::from_str(&api_key)?,
        );
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        #[derive(Deserialize, Debug)]
        struct APIResponse {
            // status: serde_json::Value,
            data: serde_json::Value,
        }

        #[derive(Deserialize, Debug)]
        struct Data {
            btc_dominance: f64,
            eth_dominance: f64,
        }

        let response = client.get(url).send().await?.json::<APIResponse>().await;
        let response = if let Ok(resp) = response {
            resp
        } else {
            let err = response.unwrap_err();
            tracing::error!("Error: {}", err);
            return Err(anyhow!(err));
        };
        
        let data: Data = serde_json::from_value(response.data)?;

        try_join!(
            cache.set_btc_dominance(data.btc_dominance),
            cache.set_eth_dominance(data.eth_dominance)
        )?;

        Ok((data.btc_dominance, data.eth_dominance))
    }

    let (btc, eth) = if let (Some(btc), Some(eth)) = (cached_btc, cached_eth) {
        (btc, eth)
    } else {
        request_dominance(&cache).await?
    };

    let text = format!("BTC dominance = {:.2}%\nETH dominance = {:.2}%", btc, eth);
    bot.send_message(msg.chat_id(), text).send().await?;

    Ok(())
}

#[instrument]
pub async fn text_handler(bot: Bot, msg: Message, _text: String, pool: Pool) -> Result<()> {
    let db = Database::new(pool.clone()).await?;
    let user_id = if let Some(from) = msg.from() {
        from.id
    } else {
        tracing::debug!("Could not update statistics for message: {:?}", msg);
        return Ok(());
    };
    let chat_id = msg.chat.id;

    update_users_mapping(&bot, msg.from(), pool.clone()).await?;

    db.update_statistics(chat_id, user_id, UpdateKind::TextMessage)
        .await?;
    Ok(())
}

#[instrument]
pub async fn messages_handler(
    bot: Bot, msg: Message,
    message: Message,
    pool: Pool,
) -> Result<()> {
    async fn impl_fn(bot: Bot, msg: Message, message: Message, pool: Pool) -> Result<()> {
        let user_id = if let Some(from) = msg.from() {
            from.id
        } else {
            return Ok(());
        };

        let db = Database::new(pool.clone()).await?;
        let chat_id = msg.chat.id;

        update_users_mapping(&bot, msg.from(), pool.clone()).await?;

        if let Some(sticker) = message.sticker() {
            db.update_statistics(chat_id, user_id, UpdateKind::Sticker)
                .await?;
            process_sticker(bot, msg, sticker).await?;
        }
        if let Some(_forwarded_from) = message.forward_from_message_id() {
            db.update_statistics(chat_id, user_id, UpdateKind::ForwardedMeme)
                .await?;
        }

        Ok(())
    }

    if let Err(e) = impl_fn(bot, msg, message, pool).await {
        sentry::integrations::anyhow::capture_anyhow(&e);
    }

    Ok(())
}

#[instrument]
pub async fn update_users_mapping(_bot: &Bot, user: Option<&User>, pool: Pool) -> Result<()> {
    let user = match user {
        Some(u) => u,
        None => return Ok(()),
    };

    let db = Database::new(pool).await?;
    let mut mapping = vec![];
    if let Some(ref username) = user.username {
        mapping.push((user.id, username.clone()));
    } else {
        let ref first_name = user.first_name;
        let username = if let Some(ref last_name) = user.last_name {
            format!("{} {}", first_name, last_name)
        } else {
            format!("{} {}", first_name, user.id)
        };
        mapping.push((user.id, username.clone()));
    }
    db.update_mapping(mapping).await?;
    Ok(())
}

#[instrument]
pub async fn process_sticker(bot: Bot, msg: Message, sticker: &Sticker) -> Result<()> {
    const FORBIDDEN_STICKER_ID: &'static str = "AgADvgADzHD_Ag";

    if sticker.file_unique_id.as_str() == FORBIDDEN_STICKER_ID {
        bot
            .delete_message(msg.chat_id(), msg.id)
            .send()
            .await?;

        let from = match msg.from() {
            Some(from) => from,
            _ => return Ok(()),
        };

        let username = from.username.as_ref().or(Some(&from.first_name)).unwrap();
        let text = format!("@{}, хватит душить котов!", username);

        bot.send_message(msg.chat_id(), text).send().await?;
    }

    return Ok(());
}

#[instrument]
async fn on_usd(bot: Bot, msg: Message ) -> Result<()> {
    let ref bot = bot;
    let chat = msg.chat_id();

    let rate = rates::get_usd_rate().await?;

    let text = format!("Курс {} = {:.2}₽", "USD", rate);

    bot.send_message(chat, &text).send().await?;
    Ok(())
}

#[instrument]
async fn on_all(bot: Bot, msg: Message ) -> Result<()> {
    let ref _bot = bot;
    let ref msg = msg;
    
    let _user_id = msg.from().unwrap().id;
    let _chat_id = msg.chat.id;

    Ok(())
}

pub fn get_handler() -> Handler<'static, DependencyMap, Result<()>> {
    let h = Update::filter_message()
        .branch(
            dptree::entry()
            .filter_command::<Command>()
            .endpoint(commands_endpoint)
        )
        .branch(
            dptree::entry()
            .endpoint(text_handler)
        );
    h
}