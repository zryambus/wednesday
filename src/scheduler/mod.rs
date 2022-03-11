mod rate_check_providers;

use crate::cache::{CachePool, RateCheck};
use crate::database::{Database, Pool};

use clokwerk::{Interval::*, ScheduleHandle, TimeUnits};
use std::sync::Arc;
use std::time::Duration;
use teloxide::{prelude2::*, ApiError, RequestError};

use rate_check_providers::{
    BTCCheckProvider, ETHCheckProvider, RateCheckProvider, ZEECheckProvider,
};

pub struct Scheduler {
    _schedule_handle: ScheduleHandle,
}

impl Scheduler {
    pub fn new(bot: teloxide::Bot, pool: Pool, cache_pool: CachePool) -> Self {
        let mut scheduler = clokwerk::Scheduler::with_tz(chrono::FixedOffset::east(3 * 3600));

        let b = bot.clone();
        let p = pool.clone();
        scheduler.every(Wednesday).at("9:00 am").run(move || {
            Self::async_task(|| Self::send_toads(b.clone(), p.clone()));
        });

        let b = bot.clone();
        let p = pool.clone();
        scheduler
            .every(1.day())
            .at("6:00 am")
            .and_every(1.day())
            .at("6:00 pm")
            .run(move || {
                Self::async_task(|| Self::send_rates(b.clone(), p.clone()));
            });

        let b = bot.clone();
        let p = pool.clone();
        let cp = cache_pool.clone();
        scheduler.every(1.minute()).run(move || {
            let provider = Arc::new(BTCCheckProvider::new(cp.clone()));
            Self::async_task(|| Self::check_rate(b.clone(), p.clone(), provider.clone()))
        });

        let b = bot.clone();
        let p = pool.clone();
        let cp = cache_pool.clone();
        scheduler.every(1.minute()).run(move || {
            let provider = Arc::new(ETHCheckProvider::new(cp.clone()));
            Self::async_task(|| Self::check_rate(b.clone(), p.clone(), provider.clone()))
        });

        scheduler.every(2.minute()).run(move || {
            let provider = Arc::new(ZEECheckProvider::new(cache_pool.clone()));
            Self::async_task(|| Self::check_rate(bot.clone(), pool.clone(), provider.clone()))
        });

        let _schedule_handle = scheduler.watch_thread(Duration::from_secs(1));
        Self { _schedule_handle }
    }

    fn async_task<Fut>(task: impl Fn() -> Fut)
    where
        Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
    {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                if let Err(ref e) = task().await {
                    sentry::integrations::anyhow::capture_anyhow(e);
                }
            });
    }

    #[tracing::instrument]
    async fn send_toads(bot: Bot, pool: Pool) -> anyhow::Result<()> {
        let db = Database::new(pool.clone()).await?;
        let chats = db.get_all_active_chats().await?;
        let url = crate::toads::get_toad();

        for chat in chats {
            if let Err(e) = bot.send_message(chat, &url).send().await {
                match e {
                    RequestError::Api(ref kind) => match kind {
                        ApiError::BotBlocked => {
                            log::warn!("Chat {} blocked the bot. Removing from active chats", chat);
                            db.remove(chat).await?;
                        }
                        _ => return Err(anyhow::anyhow!(e)),
                    },
                    _ => return Err(anyhow::anyhow!(e)),
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
            bot.send_message(chat, &text).send().await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(provider))]
    async fn check_rate(
        bot: Bot,
        pool: Pool,
        provider: Arc<impl RateCheckProvider>,
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
            bot.send_message(chat, text).send().await?;
        }

        Ok(())
    }
}
