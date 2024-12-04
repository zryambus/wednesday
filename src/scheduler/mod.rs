mod rate_check_providers;
#[macro_use]
mod retry;

use crate::cache::{CachePool, RateCheck};
use crate::database::{Database, Pool};

use clokwerk::{Interval::*, Job, TimeUnits};
use std::time::Duration;
use teloxide::{prelude::*, types::ChatId, ApiError, RequestError};
use tokio::task::JoinHandle;

use rate_check_providers::{
    BNBRateCheckProvider, BTCRateCheckProvider, ETHRateCheckProvider, NOTRateCheckProvider,
};

use self::rate_check_providers::RateCheckProvider;

#[derive(Debug, Clone)]
enum Task {
    Wednesday,
    Crypto,
    BTC,
    ETH,
    BNB,
    NOT,
    Heartbeat,
}

pub struct Scheduler {
    _schedule_handle: JoinHandle<()>,
}

impl Scheduler {
    pub fn new(bot: teloxide::Bot, pool: Pool, cache_pool: CachePool) -> Self {
        let mut scheduler = clokwerk::AsyncScheduler::with_tz(
            chrono::FixedOffset::east_opt(3 * 3600).expect("Could not set tz for scheduler"),
        );

        let (tx, rx) = tokio::sync::mpsc::channel::<Task>(10);
        async fn emit_task(tx: tokio::sync::mpsc::Sender<Task>, task: Task) {
            if let Err(e) = tx.send(task.clone()).await {
                tracing::error!("Failed to emit {:?} task: {}", task, e);
            }
        }

        let t = tx.clone();
        scheduler
            .every(Wednesday)
            .at("9:00 am")
            .run(move || emit_task(t.clone(), Task::Wednesday));

        let t = tx.clone();
        scheduler
            .every(1.day())
            .at("6:00 am")
            .and_every(1.day())
            .at("6:00 pm")
            .run(move || emit_task(t.clone(), Task::Crypto));

        let t = tx.clone();
        scheduler
            .every(10.minute())
            .run(move || emit_task(t.clone(), Task::BTC));
        let t = tx.clone();
        scheduler
            .every(10.minute())
            .run(move || emit_task(t.clone(), Task::ETH));
        let t = tx.clone();
        scheduler
            .every(10.minute())
            .run(move || emit_task(t.clone(), Task::BNB));
        let t = tx.clone();
        scheduler
            .every(10.minute())
            .run(move || emit_task(t.clone(), Task::NOT));

        let t = tx.clone();
        scheduler.every(1.hour()).run(move || {
            tracing::info!("emitting heartbeat");
            emit_task(t.clone(), Task::Heartbeat)
        });

        let handle = tokio::task::spawn(async move {
            loop {
                scheduler.run_pending().await;
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        let _thread = tokio::spawn(Self::worker(bot, pool, cache_pool, rx));

        Self {
            _schedule_handle: handle,
        }
    }

    async fn worker(
        bot: teloxide::Bot,
        pool: Pool,
        cache_pool: CachePool,
        mut rx: tokio::sync::mpsc::Receiver<Task>,
    ) {
        loop {
            tokio::select! {
                Some(task) = rx.recv() => {
                    tracing::info!("Scheduler worker received a task: {:?}", task);

                    let res = match task {
                            Task::Wednesday => Self::send_toads(bot.clone(), pool.clone()).await,
                            Task::Crypto => Self::send_rates(bot.clone(), pool.clone()).await,
                            Task::BTC => {
                                let provider = BTCRateCheckProvider::from(cache_pool.clone());
                                Self::check_rate(bot.clone(), pool.clone(), provider).await
                            },
                            Task::ETH => {
                                let provider = ETHRateCheckProvider::from(cache_pool.clone());
                                Self::check_rate(bot.clone(), pool.clone(), provider).await
                            },
                            Task::BNB => {
                                let provider = BNBRateCheckProvider::from(cache_pool.clone());
                                Self::check_rate(bot.clone(), pool.clone(), provider).await
                            },
                            Task::NOT => {
                                let provider = NOTRateCheckProvider::from(cache_pool.clone());
                                Self::check_rate(bot.clone(), pool.clone(), provider).await
                            },
                            Task::Heartbeat => {
                                tracing::info!("received heartbeat");
                                Ok(())
                            }
                    };
                    if let Err(e) = res {
                        tracing::error!("Scheduled task {:?} finished with error: {}", task, e);
                    }
                }
            }
        }
    }

    #[tracing::instrument]
    async fn send_toads(bot: Bot, pool: Pool) -> anyhow::Result<()> {
        tracing::info!("Sending toads");
        let db = Database::new(pool.clone()).await?;
        let chats = retry! { db.get_all_active_chats().await, 3, 1000 }?;
        let url = crate::toads::get_toad();
        let mapping = retry! { db.get_mapping().await }?;

        for chat in chats {
            let name = mapping
                .get(&chat)
                .cloned()
                .unwrap_or(String::from("(empty)"));

            tracing::info!("Sending toad to dude {}, name = {}", chat, name);
            if let Err(e) = bot.send_message(ChatId(chat), &url).send().await {
                sentry::capture_error(&e);

                match e {
                    RequestError::Api(ref kind) => match kind {
                        ApiError::BotBlocked => {
                            tracing::warn!(
                                "Chat {} ({}) blocked the bot. Removing from active chats",
                                chat,
                                name
                            );
                            db.remove(chat).await?;
                        }
                        ApiError::ChatNotFound => {
                            tracing::warn!(
                                "Chat {} ({}) was not found. Removing from active chats",
                                chat,
                                name
                            );
                            db.remove(chat).await?;
                        }
                        ApiError::UserDeactivated => {
                            tracing::warn!(
                                "Chat {} ({}) was deactivated. Removing from active chats",
                                chat,
                                name
                            );
                            db.remove(chat).await?;
                        }
                        _ => {}
                    },
                    RequestError::MigrateToChatId(chat_id) => {
                        tracing::warn!("Chat {} was migrated to {}. Replacing", chat, chat_id);
                        db.remove(chat).await?;
                        db.add(chat_id).await?;
                        bot.send_message(ChatId(chat_id), &url).send().await.ok();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument]
    async fn send_rates(bot: Bot, pool: Pool) -> anyhow::Result<()> {
        tracing::info!("Send rates");

        let db = Database::new(pool.clone()).await?;
        let chats = retry! { db.get_all_active_crypto_chats().await, 3, 1000 }?;

        let rate = crate::rates::get_btc_rate().await?;

        let text = if rate > 100_000. {
            format!("Когда ламба? Сегодня! Курс BTC = {}$", rate)
        } else {
            format!("Когда ламба? Не сегодня. Курс BTC = {}$", rate)
        };

        for chat in chats {
            bot.send_message(ChatId(chat), &text).send().await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(provider), fields(coin = provider.coin()))]
    async fn check_rate(
        bot: Bot,
        pool: Pool,
        provider: impl RateCheckProvider,
    ) -> anyhow::Result<()> {
        let db = Database::new(pool.clone()).await?;

        let prev_rates = provider.get_last_rates().await?;
        let current_rate = provider.get_current_rate().await?;

        let mut last_rate_check: Option<RateCheck> = None;

        if prev_rates.is_empty() {
            let rate = RateCheck {
                grow: true,
                rate: current_rate,
            };
            provider.add_last_rate(&rate).await?;
            return Ok(());
        }

        if !prev_rates.is_empty() {
            let prev = (prev_rates[0].rate / provider.step()) as i64;
            let curr = (current_rate / provider.step()) as i64;

            if prev == curr {
                return Ok(());
            }

            let grow = curr > prev;
            last_rate_check = Some(RateCheck {
                grow,
                rate: current_rate,
            });
            provider
                .add_last_rate(last_rate_check.as_ref().unwrap())
                .await?;
        }

        if prev_rates.len() < 3 {
            return Ok(());
        }

        let last_rate_check = last_rate_check.clone().unwrap();

        if last_rate_check.grow != prev_rates[0].grow {
            return Ok(());
        }

        let chats = retry! { db.get_all_active_crypto_chats().await, 3, 1000 }?;

        tracing::info!(
            "send {} rate change {} for chats {:?}",
            provider.coin(),
            last_rate_check.rate,
            chats
        );

        for chat in chats {
            let text = format!(
                "{} rate now is {}$ {}",
                provider.coin(),
                last_rate_check.rate,
                if last_rate_check.grow { "📈" } else { "📉" }
            );
            bot.send_message(ChatId(chat), text).send().await?;
        }

        Ok(())
    }
}
