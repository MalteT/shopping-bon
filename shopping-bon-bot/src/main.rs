use async_trait::async_trait;
use futures::StreamExt;
use lazy_static::lazy_static;
use storage::{CategoryDB, ItemDB};
use telegram_bot::{
    Api, CanSendMessage, Error as TelegramError, Message, MessageKind, UpdateKind, UpdatesStream,
    User, UserId,
};
use tracing::{error, info, warn};

use std::{env, error::Error, fmt};

mod storage;

lazy_static! {
    static ref ADMIN_ID: UserId = {
        let id = env::var("ADMIN_ID")
            .expect("ADMIN_ID not set")
            .parse()
            .expect("ADMIN_ID is not a number");
        UserId::new(id)
    };
    static ref USER_IDS: Vec<UserId> = {
        env::var("USER_IDS")
            .expect("USER_IDS not set")
            .split(',')
            .map(|id| id.trim())
            .map(str::parse)
            .filter_map(Result::ok)
            .map(UserId::new)
            .collect()
    };
}

#[async_trait]
pub trait InformAdminAboutErr<R, E>
where
    R: Send,
    E: Error + fmt::Display + Send,
{
    async fn send_err(self, api: &Api) -> Option<R>;
}

#[async_trait]
impl<R, E> InformAdminAboutErr<R, E> for Result<R, E>
where
    R: Send,
    E: Error + fmt::Display + Send,
{
    async fn send_err(self, api: &Api) -> Option<R>
    where
        E: Error + fmt::Display,
    {
        match self {
            Ok(res) => Some(res),
            Err(why) => {
                let text = format!("An error occured: {}", why);
                let msg = ADMIN_ID.text(&text);
                if let Err(why) = api.send(msg).await {
                    error!("Failed to inform admin about error: {}", why);
                }
                None
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandKind {
    AddItems(String),
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
            bot: TelegramBot::init(),
            itemdb: ItemDB::init(),
            categorydb: CategoryDB::init(),
        }
    }

    pub async fn handle(&mut self, cmd: &Command) {
        match cmd.kind {
            CommandKind::List => self.list_items(&cmd.source).await,
            CommandKind::Print => self.print(cmd).await,
            CommandKind::AddItems(ref items) => self.add_items(items, &cmd.source).await,
            CommandKind::ListCategories => self.list_categories(&cmd.source).await,
        }
    }

    async fn list_items(&mut self, source: &User) {
        let mut string = self
            .itemdb
            .read(|items| {
                items
                    .iter()
                    .map(|item| format!("- {}\n", item))
                    .fold(String::new(), |s, item| s + &item)
            })
            .send_err(&self.bot.api)
            .await
            .unwrap();
        if string.is_empty() {
            string = "*empty*".into();
        }
        self.msg(&source, &string).await;
    }

    async fn print(&mut self, cmd: &Command) {
        error!("Printing is not implemented yet");
        let msg = cmd.source.text("Printing unimplemented");
        self.bot.api.send(msg).await.send_err(&self.bot.api).await;
    }

    async fn add_item(&mut self, item: &str, source: &User) {
        if let Some(()) = self
            .itemdb
            .add_item(item.to_string())
            .send_err(&self.bot.api)
            .await
        {
            let msg = format!("Added '{}'.", item);
            self.msg(source, &msg).await;
        }
    }

    async fn msg<C: CanSendMessage + fmt::Debug>(&self, chat: &C, msg: &str) {
        if msg.is_empty() {
            error!("Msg should never be empty");
        } else {
            let msg = chat.text(msg);
            self.bot.api.send(&msg).await.send_err(&self.bot.api).await;
        }
    }

    async fn add_items(&mut self, items: &str, source: &User) {
        for item in items.split('\n') {
            self.add_item(item, source).await
        }
    }

    async fn list_categories(&mut self, source: &User) {
        let string = self
            .categorydb
            .read(|cats| {
                cats.iter()
                    .map(|cat| format!("{} {}\n", cat.icon, cat.name))
                    .fold(String::new(), |s, cat| s + &cat)
            })
            .send_err(&self.bot.api)
            .await
            .unwrap();
        self.msg(source, &string).await;
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
    pub fn init() -> Self {
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
