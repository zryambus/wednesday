mod rate_check_providers;

use crate::cache::{CachePool, RateCheck};
use crate::database::{Database, Pool};

use clokwerk::{Interval::*, TimeUnits, Job};
use std::time::Duration;
use teloxide::{prelude::*, ApiError, RequestError, types::ChatId};
use tokio::task::JoinHandle;

use rate_check_providers::{
    BTCRateCheckProvider, ETHRateCheckProvider, ZEERateCheckProvider, BNBRateCheckProvider
};

use self::rate_check_providers::RateCheckProvider;

pub struct Scheduler {
    _schedule_handle: JoinHandle<()>,
}

impl Scheduler {
    pub fn new(bot: teloxide::Bot, pool: Pool, cache_pool: CachePool) -> Self {
        let mut scheduler = clokwerk::AsyncScheduler::with_tz(
            chrono::FixedOffset::east_opt(3 * 3600)
                .expect("Could not set tz for scheduler")
        );

        let b = bot.clone();
        let p = pool.clone();

        scheduler.every(Wednesday).at("9:00 am").run(move || {
            let b = b.clone();
            let p = p.clone();
            Self::async_task(move || Self::send_toads(b.clone(), p.clone()))
        });

        let b = bot.clone();
        let p = pool.clone();
        scheduler
            .every(1.day())
            .at("6:00 am")
            .and_every(1.day())
            .at("6:00 pm")
            .run(move || {
                let b = b.clone();
                let p = p.clone();
                Self::async_task(move || Self::send_rates(b.clone(), p.clone()))
            });

        let b = bot.clone();
        let p = pool.clone();
        let cp = cache_pool.clone();
        scheduler.every(10.minute()).run(move || {
            let b = b.clone();
            let p = p.clone();
            let provider = BTCRateCheckProvider::from(cp.clone());
            Self::async_task(move || Self::check_rate(b.clone(), p.clone(), provider.clone()))
        });

        let b = bot.clone();
        let p = pool.clone();
        let cp = cache_pool.clone();
        scheduler.every(10.minute()).run(move || {
            let b = b.clone();
            let p = p.clone();
            let provider = ETHRateCheckProvider::from(cp.clone());
            Self::async_task(move || Self::check_rate(b.clone(), p.clone(), provider.clone()))
        });

        let b = bot.clone();
        let p = pool.clone();
        let cp = cache_pool.clone();
        scheduler.every(10.minute()).run(move || {
            let b = b.clone();
            let p = p.clone();
            let provider = BNBRateCheckProvider::from(cp.clone());
            Self::async_task(move || Self::check_rate(b.clone(), p.clone(), provider.clone()))
        });

        scheduler.every(10.minute()).run(move || {
            let b = bot.clone();
            let p = pool.clone();
            let provider = ZEERateCheckProvider::from(cache_pool.clone());
            Self::async_task(move || Self::check_rate(b.clone(), p.clone(), provider.clone()))
        });

        let handle = tokio::task::spawn(async move {
            loop {
                scheduler.run_pending().await;
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        Self { _schedule_handle: handle }
    }

    async fn async_task<Fut>(task: impl Fn() -> Fut)
    where
        Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
    {
        if let Err(ref e) = task().await {
            sentry::integrations::anyhow::capture_anyhow(e);
        }
    }

    #[tracing::instrument]
    async fn send_toads(bot: Bot, pool: Pool) -> anyhow::Result<()> {
        let db = Database::new(pool.clone()).await?;
        let chats = db.get_all_active_chats().await?;
        let url = crate::toads::get_toad();

        for chat in chats {
            tracing::warn!("Sending toad to dude {}", chat);
            if let Err(e) = bot.send_message(ChatId(chat), &url).send().await {
                sentry::capture_error(&e);

                match e {
                    RequestError::Api(ref kind) => match kind {
                        ApiError::BotBlocked => {
                            tracing::warn!("Chat {} blocked the bot. Removing from active chats", chat);
                            db.remove(chat).await?;
                        }
                        ApiError::ChatNotFound => {
                            tracing::warn!("Chat {} was not found. Removing from active chats", chat);
                            db.remove(chat).await?;
                        },
                        ApiError::UserDeactivated => {
                            tracing::warn!("Chat {} was deactivated. Removing from active chats", chat);
                            db.remove(chat).await?;
                        }
                        _ => {}
                    },
                    RequestError::MigrateToChatId(chat_id) => {
                        tracing::warn!("Chat {} was migrated to {}. Replacing", chat, chat_id);
                        db.remove(chat).await?;
                        db.add(chat_id).await?;
                        bot.send_message(ChatId(chat_id), &url).send().await.ok();
                    },
                    _ => {},
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument]
    async fn send_rates(bot: Bot, pool: Pool) -> anyhow::Result<()> {
        let db = Database::new(pool.clone()).await?;
        let chats = db.get_all_active_crypto_chats().await?;

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

    #[tracing::instrument(skip(provider))]
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

        let chats = db.get_all_active_crypto_chats().await?;
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
