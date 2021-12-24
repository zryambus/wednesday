use crate::cache::{Cache, CachePool};
use crate::database::{Database, Pool, UpdateKind};
use crate::rates;

use anyhow::{Result, Error, anyhow};
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::{try_join, Stream};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::{types::{Sticker, User}};
use teloxide::utils::command::BotCommand;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing_futures::Instrument;
use tracing::instrument;

#[derive(BotCommand, Debug)]
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
    #[command(description = "show rate of BCH to USD")]
    BCH,
    #[command(description = "show rate of ADA to USD")]
    ADA,
    #[command(description = "show rate of ZEE to USD")]
    ZEE,
    #[command(description = "show statistics")]
    Stats,
    #[command(description = "show BTC and ETH dominance")]
    Dominance,
    #[command(description = "show USD rate")]
    USD,
    #[command(description = "cast all in members in chat")]
    All
}

#[instrument]
pub async fn commands_handler(
    cx: UpdateWithCx<Bot, Message>,
    command: Command,
    pool: Pool,
    cache_pool: CachePool,
) -> Result<()> {
    let db = Database::new(pool.clone()).await?;

    update_users_mapping(&cx.requester, cx.update.from(), pool.clone()).await?;

    match command {
        Command::Help => {
            cx.answer(Command::descriptions())
                .send()
                .in_current_span()
                .await?;
        }
        Command::Start => {
            on_start(cx, db).in_current_span().await?;
        }
        Command::Stop => {
            on_stop(cx, db).in_current_span().await?;
        }
        Command::Status => {
            on_status(cx, db).in_current_span().await?;
        }
        Command::WhenLambo => on_crypto_start(cx, db).in_current_span().await?,
        Command::NotToday => on_crypto_stop(cx, db).in_current_span().await?,
        Command::Stonks => on_crypto_status(cx, db).in_current_span().await?,
        Command::Rates => on_rates(cx).in_current_span().await?,
        Command::BTC => {
            on_coin_with_24hr_change(cx, "BTC", &rates::get_btc_rate_with_24hr_change)
                .in_current_span()
                .await?
        }
        Command::ETH => {
            on_coin_with_24hr_change(cx, "ETH", &rates::get_eth_rate_with_24hr_change)
                .in_current_span()
                .await?
        }
        Command::LTC => {
            on_coin(cx, "LTC", &rates::get_ltc_rate)
                .in_current_span()
                .await?
        }
        Command::ETC => {
            on_coin(cx, "ETC", &rates::get_etc_rate)
                .in_current_span()
                .await?
        }
        Command::BCH => {
            on_coin(cx, "BCH", &rates::get_bch_rate)
                .in_current_span()
                .await?
        }
        Command::ADA => {
            on_coin(cx, "ADA", &rates::get_ada_rate)
                .in_current_span()
                .await?
        }
        Command::ZEE => {
            on_coin_with_24hr_change(cx, "ZEE", &rates::get_zee_rate_with_24hr_change)
                .in_current_span()
                .await?
        }
        Command::Stats => on_stats(cx, db).in_current_span().await?,
        Command::Dominance => {
            on_dominance(cx, cache_pool.clone())
                .in_current_span()
                .await?
        },
        Command::USD => {
            on_usd(cx).await?
        },
        Command::All => {
            on_all(cx).await?
        },
    };

    Ok(())
}

#[instrument(skip(db))]
pub async fn on_start(cx: UpdateWithCx<Bot, Message>, db: Database) -> Result<()> {
    if !db.is_active(cx.chat_id()).await? {
        db.add(cx.chat_id()).await?;
        cx.requester
            .send_message(cx.chat_id(), "✅ Chat was added to list")
            .send()
            .await?;
    } else {
        cx.requester
            .send_message(cx.chat_id(), "⚠ Current chat is already in the list")
            .send()
            .await?;
    }

    Ok(())
}

#[instrument(skip(db))]
pub async fn on_stop(cx: UpdateWithCx<Bot, Message>, db: Database) -> Result<()> {
    if db.is_active(cx.chat_id()).await? {
        db.remove(cx.chat_id()).await?;
        cx.requester
            .send_message(cx.chat_id(), "✅ Chat was removed from list")
            .send()
            .await?;
    } else {
        cx.requester
            .send_message(cx.chat_id(), "⚠ Current chat is not in the list")
            .send()
            .await?;
    }
    Ok(())
}

#[instrument(skip(db))]
pub async fn on_status(cx: UpdateWithCx<Bot, Message>, db: Database) -> Result<()> {
    let active = db.is_active(cx.chat_id()).await?;
    let text = format!(
        "Current chat is {}",
        if active {
            "in the list ✅"
        } else {
            "not in the list ❌"
        }
    );
    cx.requester.send_message(cx.chat_id(), text).send().await?;
    Ok(())
}

#[instrument(skip(db))]
pub async fn on_crypto_start(cx: UpdateWithCx<Bot, Message>, db: Database) -> Result<()> {
    if !db.is_active_crypto(cx.chat_id()).await? {
        db.add_crypto(cx.chat_id()).await?;
        cx.requester
            .send_message(cx.chat_id(), "✅ Chat was added to crypto list")
            .send()
            .await?;
    } else {
        cx.requester
            .send_message(cx.chat_id(), "⚠ Current chat is already in the crypto list")
            .send()
            .await?;
    }

    Ok(())
}

#[instrument(skip(db))]
pub async fn on_crypto_stop(cx: UpdateWithCx<Bot, Message>, db: Database) -> Result<()> {
    if db.is_active_crypto(cx.chat_id()).await? {
        db.remove_crypto(cx.chat_id()).await?;
        cx.requester
            .send_message(cx.chat_id(), "✅ Chat was removed from crypto list")
            .send()
            .await?;
    } else {
        cx.requester
            .send_message(cx.chat_id(), "⚠ Current chat is not in the crypto list")
            .send()
            .await?;
    }
    Ok(())
}

#[instrument(skip(db))]
pub async fn on_crypto_status(cx: UpdateWithCx<Bot, Message>, db: Database) -> Result<()> {
    let active = db.is_active_crypto(cx.chat_id()).await?;
    let text = format!(
        "Current crypto chat is {}",
        if active {
            "in the list ✅"
        } else {
            "not in the list ❌"
        }
    );
    cx.requester.send_message(cx.chat_id(), text).send().await?;
    Ok(())
}

#[instrument]
pub async fn on_rates(cx: UpdateWithCx<Bot, Message>) -> Result<()> {
    let ref bot = cx.requester;
    let chat = cx.chat_id();

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
    cx: UpdateWithCx<Bot, Message>,
    coin: &str,
    callback: impl FnOnce() -> Fut,
) -> Result<()>
where
    Fut: std::future::Future<Output = Result<f64>> + Send,
{
    let ref bot = cx.requester;
    let chat = cx.chat_id();

    let rate = callback().await?;

    let text = format!("Курс {} = {}$", coin, rate);

    bot.send_message(chat, &text).send().await?;
    Ok(())
}

#[instrument(skip(callback))]
async fn on_coin_with_24hr_change<Fut>(
    cx: UpdateWithCx<Bot, Message>,
    coin: &str,
    callback: impl FnOnce() -> Fut,
) -> Result<()>
where
    Fut: std::future::Future<Output = Result<(f64, f64)>> + Send,
{
    let ref bot = cx.requester;
    let chat = cx.chat_id();

    let (rate, change) = callback().await?;

    let text = format!("Курс {} = {}$ ({:.2}%)", coin, rate, change);

    bot.send_message(chat, &text).send().await?;
    Ok(())
}

#[instrument(skip(db))]
async fn on_stats(cx: UpdateWithCx<Bot, Message>, db: Database) -> Result<()> {
    let user_id = cx.update.from().unwrap().id;
    let chat_id = cx.update.chat.id;
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

    cx.answer(text).send().await?;

    Ok(())
}

#[instrument]
pub async fn on_dominance(cx: UpdateWithCx<Bot, Message>, cache_pool: CachePool) -> Result<(), Error> {
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
    cx.answer(text).send().await?;

    Ok(())
}

pub async fn repl<N, Cmd>(
    requestor: Bot,
    handlers: Arc<impl BotHandlersT<Cmd> + Sync + Send + 'static>,
    bot_name: N,
) where
    N: Into<String> + Send + 'static,
    Cmd: BotCommand + Send + Sync,
{
    let cloned_requestor = requestor.clone();
    let listener = teloxide::dispatching::update_listeners::polling(
        cloned_requestor,
        Some(Duration::from_secs(5)),
        None,
        None,
    );

    let bot_name = bot_name.into();
    let handlers = handlers.clone();

    Dispatcher::new(requestor)
        .messages_handler(move |rx: DispatcherHandlerRx<Bot, Message>| {
            UnboundedReceiverStream::new(rx)
                .messages()
                .for_each_concurrent(None, move |(cx, message)| {
                    let bot_name = bot_name.clone();
                    let handlers = handlers.clone();

                    async move {
                        let mut user: Option<sentry::User> = None;

                        if let Some(tg_user) = cx.update.from() {
                            user = Some(Default::default());
                            if let Some(ref mut u) = user {
                                u.username = tg_user.username.clone();
                                u.id = Some(tg_user.id.to_string());
                            }
                        }

                        let result: anyhow::Result<()> = if let Some(text) = message.text() {
                            let text = text.to_string();

                            if let Ok(cmd) = Cmd::parse(&text, bot_name) {
                                handlers.commands_handler(cx, cmd).await
                            } else {
                                handlers.text_handler(cx, text).await
                            }
                        } else {
                            handlers.messages_handler(cx, message).await
                        };

                        if let Err(ref e) = result {
                            sentry::with_scope(
                                |scope| {
                                    scope.set_user(user);
                                },
                                || {
                                    sentry::integrations::anyhow::capture_anyhow(e);
                                },
                            )
                        }

                        result.log_on_error().await;
                    }
                })
        })
        .setup_ctrlc_handler()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}

#[instrument]
pub async fn text_handler(cx: UpdateWithCx<Bot, Message>, _text: String, pool: Pool) -> Result<()> {
    let db = Database::new(pool.clone()).await?;
    let user_id = if let Some(from) = cx.update.from() {
        from.id
    } else {
        tracing::debug!("Could not update statistics for message: {:?}", cx);
        return Ok(());
    };
    let chat_id = cx.update.chat.id;

    update_users_mapping(&cx.requester, cx.update.from(), pool.clone()).await?;

    db.update_statistics(chat_id, user_id, crate::database::UpdateKind::TextMessage)
        .await?;
    Ok(())
}

#[instrument]
pub async fn messages_handler(
    cx: UpdateWithCx<Bot, Message>,
    message: Message,
    pool: Pool,
) -> Result<()> {
    async fn impl_fn(cx: UpdateWithCx<Bot, Message>, message: Message, pool: Pool) -> Result<()> {
        let db = Database::new(pool.clone()).await?;
        let user_id = cx.update.from().unwrap().id;
        let chat_id = cx.update.chat.id;

        update_users_mapping(&cx.requester, cx.update.from(), pool.clone()).await?;

        if let Some(sticker) = message.sticker() {
            db.update_statistics(chat_id, user_id, UpdateKind::Sticker)
                .await?;
            process_sticker(cx, sticker).await?;
        }
        if let Some(_forwarded_from) = message.forward_from_message_id() {
            db.update_statistics(chat_id, user_id, UpdateKind::ForwardedMeme)
                .await?;
        }

        Ok(())
    }

    if let Err(e) = impl_fn(cx, message, pool).await {
        sentry::integrations::anyhow::capture_anyhow(&e);
    }

    Ok(())
}

pub trait DispatcherHandler<R> {
    fn messages(self) -> BoxStream<'static, (UpdateWithCx<R, Message>, Message)>
    where
        Self: Stream<Item = UpdateWithCx<R, Message>>,
        R: Send + 'static;
}

impl<R, T> DispatcherHandler<R> for T
where
    T: Send + 'static,
{
    fn messages(self) -> BoxStream<'static, (UpdateWithCx<R, Message>, Message)>
    where
        Self: Stream<Item = UpdateWithCx<R, Message>>,
        R: Send + 'static,
    {
        self.map(move |cx| {
            let message = cx.update.clone();
            (cx, message)
        })
        .boxed()
    }
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
pub async fn process_sticker(cx: UpdateWithCx<Bot, Message>, sticker: &Sticker) -> Result<()> {
    const FORBIDDEN_STICKER_ID: &'static str = "AgADvgADzHD_Ag";

    if sticker.file_unique_id.as_str() == FORBIDDEN_STICKER_ID {
        cx.requester
            .delete_message(cx.chat_id(), cx.update.id)
            .send()
            .await?;

        let from = match cx.update.from() {
            Some(from) => from,
            _ => return Ok(()),
        };

        let username = from.username.as_ref().or(Some(&from.first_name)).unwrap();
        let text = format!("@{}, хватит душить котов!", username);

        cx.requester.send_message(cx.chat_id(), text).send().await?;
    }

    return Ok(());

    // let user = if let Some(user) = cx.update.from() {
    //     user
    // } else {
    //     return Ok(());
    // };
    //
    // if user.id == 203295139 {
    //     if rand::random::<f64>().round() as i64 != 1 {
    //         return Ok(());
    //     }
    //
    //     cx.requester
    //         .delete_message(cx.chat_id(), cx.update.id)
    //         .send()
    //         .await?;
    //
    //     let username = user.username.as_ref().or(Some(&user.first_name)).unwrap();
    //     let text = format!("@{}, иди увольняй работяг!", username);
    //
    //     cx.requester.send_message(cx.chat_id(), text).send().await?;
    // }
    //
    // Ok(())
}

#[instrument]
async fn on_usd(cx: UpdateWithCx<Bot, Message>) -> Result<()> {
    let ref bot = cx.requester;
    let chat = cx.chat_id();

    let rate = rates::get_usd_rate().await?;

    let text = format!("Курс {} = {:.2}₽", "USD", rate);

    bot.send_message(chat, &text).send().await?;
    Ok(())
}

#[instrument]
async fn on_all(cx: UpdateWithCx<Bot, Message>) -> Result<()> {
    let ref _bot = cx.requester;
    let ref msg = cx.update;
    
    let _user_id = msg.from().unwrap().id;
    let _chat_id = msg.chat.id;

    Ok(())
}

#[async_trait]
pub trait BotHandlersT<Cmd: Send> {
    async fn commands_handler(&self, cx: UpdateWithCx<Bot, Message>, cmd: Cmd) -> Result<()>;
    async fn text_handler(&self, cx: UpdateWithCx<Bot, Message>, text: String) -> Result<()>;
    async fn messages_handler(&self, cx: UpdateWithCx<Bot, Message>, msg: Message) -> Result<()>;
}

pub struct BotHandlers {
    db_pool: Pool,
    cache_pool: CachePool,
}

impl BotHandlers {
    pub fn new(db_pool: Pool, cache_pool: CachePool) -> Self {
        Self {
            db_pool,
            cache_pool,
        }
    }
}

#[async_trait]
impl BotHandlersT<Command> for BotHandlers {
    async fn commands_handler(&self, cx: UpdateWithCx<Bot, Message>, cmd: Command) -> Result<()> {
        commands_handler(cx, cmd, self.db_pool.clone(), self.cache_pool.clone()).await
    }
    async fn text_handler(&self, cx: UpdateWithCx<Bot, Message>, text: String) -> Result<()> {
        text_handler(cx, text, self.db_pool.clone()).await
    }
    async fn messages_handler(&self, cx: UpdateWithCx<Bot, Message>, msg: Message) -> Result<()> {
        messages_handler(cx, msg, self.db_pool.clone()).await
    }
}
