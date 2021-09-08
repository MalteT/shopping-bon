use telegram_bot::User;
use tracing::{info, warn};

mod bot;
mod error;
mod result_ext;
mod settings;
mod state;
mod storage;

pub use bot::TelegramBot;
pub use error::Error;
pub use result_ext::ResultExt;
pub use state::State;

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
