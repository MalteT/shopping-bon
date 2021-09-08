use futures::StreamExt;
use telegram_bot::{
    Api, Error as TelegramError, Message, MessageKind, Request, UpdateKind, UpdatesStream,
};

use crate::{settings::SETTINGS, Command, CommandKind, ResultExt};

pub struct TelegramBot {
    api: Api,
    stream: UpdatesStream,
}

impl TelegramBot {
    pub fn init() -> Self {
        let token = &SETTINGS.bot.token;
        let api = Api::new(token);
        let stream = api.stream();
        TelegramBot { api, stream }
    }

    pub async fn poll(&mut self) -> Result<Option<Command>, TelegramError> {
        let cmd = self.stream.next().await.transpose()?.and_then(|update| {
            // If the received update contains a new message...
            if let UpdateKind::Message(message) = update.kind {
                message_to_command(message)
            } else {
                None
            }
        });
        Ok(cmd)
    }

    pub async fn send<R: Request>(&self, request: R) -> Result<(), TelegramError> {
        self.api.send(request).await?;
        Ok(())
    }

    pub async fn send_or_ignore<R: Request>(&self, request: R) {
        self.api
            .send(request)
            .await
            .log_err("failed to send message");
    }
}

fn message_to_command(message: Message) -> Option<Command> {
    // We only care about text messages
    let kind = if let MessageKind::Text { ref data, .. } = message.kind {
        let lower = data.to_lowercase();
        // Match commands, everything not starting with a slash
        // will be interpreted as a new item
        if lower == "/print" {
            Some(CommandKind::Print)
        } else if lower == "/list" {
            Some(CommandKind::List)
        } else if lower == "/categories" {
            Some(CommandKind::ListCategories)
        } else if !data.starts_with('/') {
            Some(CommandKind::AddItems(data.into()))
        } else {
            None
        }
    } else {
        None
    };
    kind.map(|kind| Command {
        kind,
        source: message.from,
    })
}
