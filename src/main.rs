use futures::StreamExt;
use storage::{CategoryDB, ItemDB};
use telegram_bot::{
    Api, CanSendMessage, Error as TelegramError, Message, MessageKind, UpdateKind, UpdatesStream,
    User,
};
use tracing::{error, info, warn};

use std::env;

mod storage;

#[derive(Debug, PartialEq, Eq)]
pub enum CommandKind {
    AddItem(String),
    List,
    ListCategories,
    Print,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub source: User,
    pub kind: CommandKind,
}

pub struct TelegramBot {
    api: Api,
    stream: UpdatesStream,
}

/// Assembly of all relevant items
pub struct State {
    pub bot: TelegramBot,
    pub itemdb: ItemDB,
    pub categorydb: CategoryDB,
}

impl State {
    pub fn init() -> Self {
        State {
            bot: TelegramBot::new(),
            itemdb: ItemDB::init(),
            categorydb: CategoryDB::init(),
        }
    }

    pub async fn handle(&mut self, cmd: &Command) {
        match cmd.kind {
            CommandKind::List => {
                let string = self
                    .itemdb
                    .read(|items| {
                        items
                            .iter()
                            .map(|item| format!("- {}\n", item))
                            .fold(String::new(), |s, item| s + &item)
                    })
                    .unwrap();
                let msg = cmd.source.text(&string);
                self.bot.api.send(msg).await.unwrap();
            }
            CommandKind::Print => {
                error!("Printing is not implemented yet");
                let msg = cmd.source.text("Printing unimplemented");
                self.bot.api.send(msg).await.unwrap();
            }
            CommandKind::AddItem(ref item) => {
                self.itemdb.write(|items| items.push(item.clone())).unwrap();
                let msg = cmd.source.text(format!("Added {}", item));
                self.bot.api.send(msg).await.unwrap();
            }
            CommandKind::ListCategories => {
                let string = self
                    .categorydb
                    .read(|cats| {
                        cats.iter()
                            .map(|cat| format!("{} {}\n", cat.icon, cat.name))
                            .fold(String::new(), |s, cat| s + &cat)
                    })
                    .unwrap();
                let msg = cmd.source.text(dbg!(&string));
                self.bot.api.send(msg).await.unwrap();
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), telegram_bot::Error> {
    // Read environment
    dotenv::dotenv().ok();
    // Initializer logger
    tracing_subscriber::fmt().pretty().init();
    // Initialize everything
    let mut state = State::init();
    // Start polling messages from telegram
    info!("Started!");
    loop {
        match state.bot.poll().await {
            Ok(None) => {}
            Ok(Some(cmd)) => {
                info!("Received {:?} from {}", cmd.kind, cmd.source.first_name);
                state.handle(&cmd).await;
            }
            Err(why) => warn!("Bot error: {}", why),
        }
    }
}

impl TelegramBot {
    pub fn new() -> Self {
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
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
            Some(CommandKind::AddItem(data.into()))
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
