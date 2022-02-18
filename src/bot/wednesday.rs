use anyhow::{Result, anyhow};
use teloxide::{prelude::*, RequestError, types::ChatId};

pub struct WednesdayBot {
    cx: UpdateWithCx<Bot, Message>,
}

impl WednesdayBot {
    pub fn new(cx: UpdateWithCx<Bot, Message>) -> Self {
        Self {
            cx,
        }
    }

    fn bot(&self) -> &Bot {
        &self.cx.requester
    }

    pub async fn send_text<C, T>(&self, chat_id: C, text: T) -> Result<()> 
    where C: Into<ChatId> + Clone, T: Into<String> + Clone
    {
        loop {
            let result = self.bot().send_message(chat_id.clone(), text.clone()).send().await;
            if let Err(e) = result {
                sentry::capture_error(&e);
                match e {
                    RequestError::RetryAfter(timeout) => {
                        tokio::time::sleep(std::time::Duration::from_secs(timeout as u64)).await;
                    },
                    RequestError::NetworkError(error) => {
                        tracing::debug!("Got network error while sending message: {}", error);
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    },
                    RequestError::MigrateToChatId(new_chat_id) => {
                        self.bot().send_message(new_chat_id, text.clone()).send().await?;
                        return Ok(());
                    }
                    _ => return Err(anyhow!(e))
                }
            } else {
                break;
            }
        }
        Ok(())
    }
    
}

impl GetChatId for WednesdayBot {
    fn chat_id(&self) -> i64 {
        self.cx.chat_id()
    }
}