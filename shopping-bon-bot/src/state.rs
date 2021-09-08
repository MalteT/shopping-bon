use telegram_bot::{CanSendMessage, User};
use tracing::error;

use std::fmt;

use crate::{
    bot::TelegramBot,
    settings::{Role, SETTINGS},
    storage::{CategoryDB, ItemDB},
    Command, CommandKind, ResultExt,
};

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
            CommandKind::Print => self.print(&cmd.source).await,
            CommandKind::AddItems(ref items) => self.add_items(items, &cmd.source).await,
            CommandKind::ListCategories => self.list_categories(&cmd.source).await,
        }
    }

    async fn list_items(&mut self, source: &User) {
        match SETTINGS.get_role(source.id) {
            Some(Role { read, .. }) if *read => {
                let mut string = self
                    .itemdb
                    .read(|items| {
                        items
                            .iter()
                            .map(|item| format!("- {}\n", item))
                            .fold(String::new(), |s, item| s + &item)
                    })
                    .log_err("failed to fetch items from database")
                    .unwrap_or_else(|| String::from("*empty*"));
                if string.is_empty() {
                    string = String::from("*empty*");
                }
                self.msg(&source, &string).await;
            }
            _ => {
                self.msg(&source, "*missing permissions*").await;
            }
        }
    }

    async fn print(&mut self, source: &User) {
        match SETTINGS.get_role(source.id) {
            Some(Role { print, .. }) if *print => {
                error!("Printing is not implemented yet");
                let msg = source.text("Printing unimplemented");
                self.bot.send_or_ignore(msg).await;
            }
            _ => {
                self.msg(&source, "*missing permissions*").await;
            }
        }
    }

    async fn add_item(&mut self, item: &str, source: &User) {
        match self
            .itemdb
            .add_item(item.to_string())
            .log_err("failed to add item to database")
        {
            Some(_) => {
                let msg = format!("Added '{}'.", item);
                self.msg(source, &msg).await
            }
            None => self.msg(source, &format!("*failed to add {}*", item)).await,
        }
    }

    async fn msg<C: CanSendMessage + fmt::Debug>(&self, chat: &C, msg: &str) {
        if msg.is_empty() {
            error!("Msg should never be empty");
        } else {
            let msg = chat.text(msg);
            self.bot.send_or_ignore(&msg).await;
        }
    }

    async fn add_items(&mut self, items: &str, source: &User) {
        match SETTINGS.get_role(source.id) {
            Some(Role { write, .. }) if *write => {
                for item in items.split('\n') {
                    self.add_item(item, source).await
                }
            }
            _ => {
                self.msg(&source, "*missing permissions*").await;
            }
        }
    }

    async fn list_categories(&mut self, source: &User) {
        match SETTINGS.get_role(source.id) {
            Some(Role { read, .. }) if *read => {
                let mut string = self
                    .categorydb
                    .read(|cats| {
                        cats.iter()
                            .map(|cat| format!("{} {}\n", cat.icon, cat.name))
                            .fold(String::new(), |s, cat| s + &cat)
                    })
                    .log_err("failed to read categories from database")
                    .unwrap_or_else(|| String::from("*empty*"));
                if string.is_empty() {
                    string = String::from("*empty*");
                }
                self.msg(source, &string).await;
            }
            _ => {
                self.msg(&source, "*missing permissions*").await;
            }
        }
    }
}
