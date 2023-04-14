use std::sync::{Arc, RwLock};

use crate::cache::{Cache, CachePool};
use crate::database::{Database, Pool, UpdateKind};
use crate::rates;

use anyhow::{anyhow, Error, Result};
use futures::try_join;
use serde::Deserialize;
use teloxide::dispatching::{UpdateFilterExt, DpHandlerDescription};
use teloxide::types::{InlineQueryResultArticle, InputMessageContent, InputMessageContentText};
use teloxide::{
    prelude::*,
    types::{Sticker, Update, User},
    utils::command::BotCommands,
};
use tracing::instrument;

mod wednesday;

use wednesday::WednesdayBot;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
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
    #[command(description = "show rates of BTC, ETH and BNB to USD")]
    Rates,
    #[command(description = "show rate of BTC to USD")]
    Btc,
    #[command(description = "show rate of ETH to USD")]
    Eth,
    #[command(description = "show rate of LTC to USD")]
    Ltc,
    #[command(description = "show rate of ETC to USD")]
    Etc,
    #[command(description = "show rate of SOL to USD")]
    Sol,
    #[command(description = "show rate of ADA to USD")]
    Ada,
    #[command(description = "show rate of ZEE to USD")]
    Zee,
    #[command(description = "show rate of BNB to USD")]
    Bnb,
    #[command(description = "show rate of LUNA to USD")]
    Luna,
    #[command(description = "show statistics")]
    Stats,
    #[command(description = "show BTC and ETH dominance")]
    Dominance,
    #[command(description = "show USD rate")]
    Usd,
    #[command(description = "cast all in members in chat")]
    All,
}

pub async fn commands_endpoint(
    bot: Bot,
    msg: Message,
    command: Command,
    pool: Pool,
    cache_pool: CachePool,
) -> Result<()> {
    let db = Database::new(pool.clone()).await?;

    update_users_mapping(&bot, msg.from(), pool.clone()).await
        .map_err(|e| {
            tracing::error!("Failed to update users mapping: {}", e);
            e
        }).ok();

    match command {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
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
        Command::Btc => {
            on_coin_with_24hr_change(bot, msg, "BTC", &rates::get_btc_rate_with_24hr_change).await?
        }
        Command::Eth => {
            on_coin_with_24hr_change(bot, msg, "ETH", &rates::get_eth_rate_with_24hr_change).await?
        }
        Command::Ltc => on_coin(bot, msg, "LTC", &rates::get_ltc_rate).await?,
        Command::Etc => on_coin(bot, msg, "ETC", &rates::get_etc_rate).await?,
        Command::Sol => {
            on_coin_with_24hr_change(bot, msg, "SOL", &rates::get_sol_rate_with_24hr_change).await?
        }
        Command::Ada => on_coin(bot, msg, "ADA", &rates::get_ada_rate).await?,
        Command::Zee => {
            on_coin_with_24hr_change(bot, msg, "ZEE", &rates::get_zee_rate_with_24hr_change).await?
        }
        Command::Bnb => {
            on_coin_with_24hr_change(bot, msg, "BNB", &rates::get_bnb_rate_with_24hr_change).await?
        }
        Command::Luna => {
            on_coin_with_24hr_change(bot, msg, "LUNA", &rates::get_luna_rate_with_24hr_change)
                .await?
        }
        Command::Stats => on_stats(bot, msg, db).await?,
        Command::Dominance => on_dominance(bot, msg, cache_pool.clone()).await?,
        Command::Usd => on_usd(bot, msg).await?,
        Command::All => on_all(bot, msg).await?,
    };

    Ok(())
}

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
pub enum AdminCommand {
    #[command(description = "display users mapping.")]
    Mapping,
}

pub async fn admin_commands_endpoint(
    bot: Bot,
    msg: Message,
    command: AdminCommand,
    pool: Pool,
) -> Result<()> {
    let db = Database::new(pool.clone()).await?;

    match command {
        AdminCommand::Mapping => {
            let mapping = db.get_mapping().await?;
            let text = mapping.iter()
                .map(|(id, username)| format!("{}: {}", id, username))
                .collect::<Vec<String>>()
                .join("\n");
            bot.send_message(msg.chat.id, text)
                .send()
                .await?;
        }
    };

    Ok(())
}


#[instrument(skip(db))]
pub async fn on_start(bot: Bot, msg: Message, db: Database) -> Result<()> {
    if !db.is_active(msg.chat.id.0).await? {
        db.add(msg.chat.id.0).await?;
        bot.send_message(msg.chat.id, "✅ Chat was added to list")
            .send()
            .await?;
    } else {
        bot.send_message(msg.chat.id, "⚠ Current chat is already in the list")
            .send()
            .await?;
    }

    Ok(())
}

#[instrument(skip(db))]
pub async fn on_stop(bot: Bot, msg: Message, db: Database) -> Result<()> {
    if db.is_active(msg.chat.id.0).await? {
        db.remove(msg.chat.id.0).await?;
        bot.send_message(msg.chat.id, "✅ Chat was removed from list")
            .send()
            .await?;
    } else {
        bot.send_message(msg.chat.id, "⚠ Current chat is not in the list")
            .send()
            .await?;
    }
    Ok(())
}

#[instrument(skip(db))]
pub async fn on_status(bot: Bot, msg: Message, db: Database) -> Result<()> {
    let active = db.is_active(msg.chat.id.0).await?;
    let text = format!(
        "Current chat is {}",
        if active {
            "in the list ✅"
        } else {
            "not in the list ❌"
        }
    );
    bot.send_message(msg.chat.id, text).send().await?;
    Ok(())
}

#[instrument(skip(db))]
pub async fn on_crypto_start(bot: Bot, msg: Message, db: Database) -> Result<()> {
    if !db.is_active_crypto(msg.chat.id.0).await? {
        db.add_crypto(msg.chat.id.0).await?;
        bot.send_message(msg.chat.id, "✅ Chat was added to crypto list")
            .send()
            .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            "⚠ Current chat is already in the crypto list",
        )
        .send()
        .await?;
    }

    Ok(())
}

#[instrument(skip(db))]
pub async fn on_crypto_stop(bot: Bot, msg: Message, db: Database) -> Result<()> {
    if db.is_active_crypto(msg.chat.id.0).await? {
        db.remove_crypto(msg.chat.id.0).await?;
        bot.send_message(msg.chat.id, "✅ Chat was removed from crypto list")
            .send()
            .await?;
    } else {
        bot.send_message(msg.chat.id, "⚠ Current chat is not in the crypto list")
            .send()
            .await?;
    }
    Ok(())
}

#[instrument(skip(db))]
pub async fn on_crypto_status(bot: Bot, msg: Message, db: Database) -> Result<()> {
    let active = db.is_active_crypto(msg.chat.id.0).await?;
    let text = format!(
        "Current crypto chat is {}",
        if active {
            "in the list ✅"
        } else {
            "not in the list ❌"
        }
    );
    bot.send_message(msg.chat.id, text).send().await?;
    Ok(())
}

#[instrument]
pub async fn on_rates(bot: Bot, msg: Message) -> Result<()> {
    let chat = msg.chat.id;

    let (btc, eth, bnb) = try_join!(
        rates::get_btc_rate(),
        rates::get_eth_rate(),
        rates::get_bnb_rate(),
    )?;

    let text = format!("BTC = {}$\nETH = {}$\nBNB = {}$", btc, eth, bnb);

    bot.send_message(chat, &text).send().await?;
    Ok(())
}

#[instrument(skip(callback))]
async fn on_coin<Fut>(
    bot: Bot,
    msg: Message,
    coin: &str,
    callback: impl FnOnce() -> Fut,
) -> Result<()>
where
    Fut: std::future::Future<Output = Result<f64>> + Send,
{
    let bot = WednesdayBot::new(bot, msg);
    let chat = bot.chat_id();

    let rate = callback().await;

    let text = match rate {
        Ok(rate) => {
            format!("Курс {} = {}$", coin, rate)
        },
        Err(e) => {
            tracing::error!("on_coin callback finished with error: {}", e);
            format!("Error: {}", e)
        }
    };

    bot.send_text(chat, text).await?;
    Ok(())
}

#[instrument(skip(callback))]
async fn on_coin_with_24hr_change<Fut>(
    bot: Bot,
    msg: Message,
    coin: &str,
    callback: impl FnOnce() -> Fut,
) -> Result<()>
where
    Fut: std::future::Future<Output = Result<(f64, f64)>> + Send,
{
    let chat = msg.chat.id;

    let (rate, change) = callback().await?;

    let text = format!("Курс {} = {}$ ({:.2}%)", coin, rate, change);

    bot.send_message(chat, &text).send().await?;
    Ok(())
}

#[instrument(skip(db))]
async fn on_stats(bot: Bot, msg: Message, db: Database) -> Result<()> {
    let user_id = msg.from().unwrap().id;
    let chat_id = msg.chat.id;
    let stats = db.get_statistics(chat_id.0, user_id.0).await?;

    let mut text = String::from("Your statistics for today:\n");
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

    bot.send_message(msg.chat.id, text).send().await?;

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
    bot.send_message(msg.chat.id, text).send().await?;

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

    db.update_statistics(chat_id.0, user_id.0, UpdateKind::TextMessage)
        .await?;
    Ok(())
}

#[instrument]
pub async fn messages_handler(bot: Bot, msg: Message, message: Message, pool: Pool) -> Result<()> {
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
            db.update_statistics(chat_id.0, user_id.0, UpdateKind::Sticker)
                .await?;
            process_sticker(bot, msg, sticker).await?;
        }
        if let Some(_forwarded_from) = message.forward_from_message_id() {
            db.update_statistics(chat_id.0, user_id.0, UpdateKind::ForwardedMeme)
                .await?;
        }

        Ok(())
    }

    if let Err(e) = impl_fn(bot, msg, message, pool).await {
        sentry::integrations::anyhow::capture_anyhow(&e);
    }

    Ok(())
}

pub async fn update_users_mapping(_bot: &Bot, user: Option<&User>, pool: Pool) -> Result<()> {
    let user = match user {
        Some(u) => u,
        None => return Ok(()),
    };

    let db = Database::new(pool).await?;
    let mut mapping = vec![];
    if let Some(ref username) = user.username {
        mapping.push((user.id.0, username.clone()));
    } else {
        let first_name = &user.first_name;
        let username = if let Some(ref last_name) = user.last_name {
            format!("{} {}", first_name, last_name)
        } else {
            format!("{} {}", first_name, user.id)
        };
        mapping.push((user.id.0, username));
    }
    db.update_mapping(mapping).await?;
    Ok(())
}

#[instrument]
pub async fn process_sticker(bot: Bot, msg: Message, sticker: &Sticker) -> Result<()> {
    const FORBIDDEN_STICKER_ID: &str = "AgADvgADzHD_Ag";

    if sticker.file.unique_id.as_str() == FORBIDDEN_STICKER_ID {
        bot.delete_message(msg.chat.id, msg.id).send().await?;

        let from = match msg.from() {
            Some(from) => from,
            _ => return Ok(()),
        };

        let username = from.username.as_ref().or(Some(&from.first_name)).unwrap();
        let text = format!("@{}, хватит душить котов!", username);

        bot.send_message(msg.chat.id, text).send().await?;
    }

    Ok(())
}

#[instrument]
async fn on_usd(bot: Bot, msg: Message) -> Result<()> {
    let chat = msg.chat.id;

    let rate = rates::get_usd_rate().await?;

    let text = format!("Курс {} = {:.2}₽", "USD", rate);

    bot.send_message(chat, &text).send().await?;
    Ok(())
}

#[instrument]
async fn on_all(bot: Bot, msg: Message) -> Result<()> {
    let _bot = &bot;

    let _user_id = msg.from().unwrap().id;
    let _chat_id = msg.chat.id;

    Ok(())
}

#[derive(Clone)]
pub struct Gauss {
    mean: f64,
    dev: f64,

    ready: bool,
    second: f64,
}

impl Gauss {
    pub fn new(mean: f64, dev: f64) -> Self {
        Self {
            mean,
            dev,
            ready: false,
            second: 0.,
        }
    }

    pub fn next(&mut self) -> f64 {
        if self.ready {
            self.ready = false;
            self.second * self.dev + self.mean
        } else {
            let mut u: f64;
            let mut v: f64;
            let mut s: f64;

            loop {
                u = 2.0 * alea::f64() - 1.0;
                v = 2.0 * alea::f64() - 1.0;
                s = u * u + v * v;

                if !(s > 1. || s == 0.) {
                    break;
                }
            }

            let r = (-2. * s.log2() / s).sqrt();
            self.second = r * u;
            self.ready = true;

            r * v * self.dev + self.mean
        }
    }
}

pub fn get_handler(admin_user_id: i64) -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription> {
    async fn dummy() -> Result<()> {
        Ok(())
    }

    async fn inline_endpoint(bot: Bot, query: InlineQuery, g: Arc<RwLock<Gauss>>) -> Result<()> {
        let cocksize = g.write().map_err(|e| anyhow!("{}", e))?.next() as i32;
        let emoji = match cocksize {
            0..=3 => "продуло 🥶",
            4..=9 => "😒",
            10..=13 => "😐",
            14..=18 => "🤗",
            19..=25 => "😎",
            26..=40 => "👬",
            _ => "😮"
        };
        let response = InlineQueryResultArticle::new(
            format!("{}", query.from.id),
            "Share your cock size",
            InputMessageContent::Text(InputMessageContentText::new(format!(
                "My cock size is {}cm {}",
                cocksize,
                emoji
            ))),
        );
        let mut answer = bot.answer_inline_query(query.id, vec![response.into()]);
        answer.cache_time = Some(60); // is secs
        answer.is_personal = Some(true); // cached for specific user
        answer.send().await?;

        Ok(())
    }

    let h = Update::filter_message()
        .branch(
            // set sentry user middleware
            dptree::filter(|msg: Message| {
                let username = msg.chat.username().map(ToOwned::to_owned);
                sentry::configure_scope(|scope| {
                    scope.set_user(Some(sentry::User {
                        username,
                        ..Default::default()
                    }))
                });
                return false;
            })
            .endpoint(dummy),
        )
        .branch(
            dptree::entry()
                .filter_command::<AdminCommand>()
                .filter(move |msg: Message| msg.chat.id.0 == admin_user_id)
                .endpoint(admin_commands_endpoint),
        )
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(commands_endpoint),
        )
        .branch(dptree::entry().endpoint(text_handler));

    let i = Update::filter_inline_query().endpoint(inline_endpoint);

    dptree::entry().branch(h).branch(i)
}
