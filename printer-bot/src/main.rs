use escpos_lib::{FmtStr, Printer};
use futures::StreamExt;
use serialport::SerialPort;
use telegram_bot::{
    Api, CanSendMessage, Error as TelegramError, Message, MessageKind, UpdateKind, UpdatesStream,
    User as TelegramUser, UserId,
};
use tracing::{info, warn};

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

mod error;
mod settings;

pub use error::Error;
pub use settings::SETTINGS;

/// All relevant state.
pub struct TelegramBot<P: SerialPort> {
    api: Api,
    stream: UpdatesStream,
    printer: Printer<P>,
    history: History,
}

/// History of executed command.
#[derive(Debug, Default)]
pub struct History {
    /// Instances the corresponding users last printed something.
    last_print: HashMap<UserId, Instant>,
}

impl History {
    /// Get the time passed since the given user last printed something.
    pub fn duration_since_last_print(&self, id: &UserId) -> Option<Duration> {
        self.last_print.get(id).map(|instant| instant.elapsed())
    }
    /// Update the last time the given user printed something.
    pub fn add_print(&mut self, id: &UserId) {
        self.last_print.insert(*id, Instant::now());
    }
}

/// Possible commands that can be executed.
#[derive(Debug, PartialEq, Eq)]
pub enum CommandKind {
    /// Print the given string.
    Print(String),
}

/// Command send via Telegram.
#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    /// User that send the command.
    pub source: TelegramUser,
    /// Command send by the user.
    pub kind: CommandKind,
}

impl<P: SerialPort> TelegramBot<P> {
    /// Initialize the bot with all corresponding data.
    ///
    /// # Arguments
    /// - `port`: The serial port the printer is connected to.
    pub fn init(port: P) -> Self {
        let token = &SETTINGS.bot.token;
        let api = Api::new(token);
        let stream = api.stream();
        let printer = Printer::new(port).expect("Failed to initialize printer");
        let history = History::default();
        TelegramBot {
            api,
            stream,
            printer,
            history,
        }
    }
    /// Poll for updates from the Telegram API.
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
    /// Handle the given command.
    pub async fn handle(&mut self, cmd: &Command) -> Result<(), Error> {
        let Command { source, kind } = cmd;
        match kind {
            CommandKind::Print(text) => self.handle_print_cmd(source, text).await,
        }
    }
    /// Handle the print command.
    ///
    /// Prints the data and sends feedback to the user who issued it.
    async fn handle_print_cmd(&mut self, source: &TelegramUser, text: &str) -> Result<(), Error> {
        if self.is_printing_allowed(source.id) {
            if self.is_print_length_allowed(source.id, text.len()) {
                self.print_message(source, text)?;
                info!("Printed message {:?} from id '{}'", text, source.id);
                self.send(source.id, "🖨️✅").await
            } else {
                self.send(source.id, "🖨️❌ That message is too long!")
                    .await?;
                info!(
                    "Rejected print command for long message from id: {}",
                    source.id
                );
                Ok(())
            }
        } else {
            self.send(source.id, "🖨️❌ You may not print now!").await?;
            info!("Rejected print command from id: {}", source.id);
            Ok(())
        }
    }
    /// Print a simple startup message to announce that the bot is running
    pub fn print_startup_message(&mut self) -> Result<(), Error> {
        self.printer
            .write_and_cut("*** Printer-bot started ***\n")
            .map_err(Error::Printing)
    }
    /// Print the given Message.
    ///
    /// Adjusts the history aswell.
    fn print_message(&mut self, source: &TelegramUser, text: &str) -> Result<(), Error> {
        let name = if let Some(ref last_name) = source.last_name {
            format!(" {} {} ", source.first_name, last_name)
        } else {
            format!(" {} ", source.first_name)
        };
        let text = any_ascii::any_ascii(text);
        let text = escpos_lib::escape(&text);
        let formatted = format!("{}: {}\n", name.reverse(), text);
        self.printer
            .write_and_cut(formatted)
            .map_err(Error::Printing)?;
        self.history.add_print(&source.id);
        Ok(())
    }
    /// Compares the message length with the permissions.
    fn is_print_length_allowed(&self, id: UserId, len: usize) -> bool {
        if let Some(role) = SETTINGS.get_role(id) {
            len <= role.max_print_len
        } else {
            false
        }
    }
    /// Checks whether the given user id is allowed to print right now.
    fn is_printing_allowed(&self, id: UserId) -> bool {
        if let Some(role) = SETTINGS.get_role(id) {
            let min_dur = Duration::from_secs(60 * role.minutes_between_prints as u64);
            if let Some(curr_dur) = self.history.duration_since_last_print(&id) {
                min_dur <= curr_dur
            } else {
                true
            }
        } else {
            false
        }
    }
    /// Send `text` to the user `id`.
    async fn send(&mut self, id: UserId, text: &str) -> Result<(), Error> {
        let msg = id.text(text);
        self.api.send(msg).await.map_err(Error::SendingMessage)?;
        Ok(())
    }
}

/// Parse a Telegram message into a [`Command`].
fn message_to_command(message: Message) -> Option<Command> {
    // We only care about text messages
    let kind = if let MessageKind::Text { data, .. } = message.kind {
        if data == "/start" {
            None
        } else {
            Some(CommandKind::Print(data))
        }
    } else {
        None
    };
    kind.map(|kind| Command {
        kind,
        source: message.from,
    })
}

/// Initialize the printer serial port.
fn init_printer_port() -> impl SerialPort {
    serialport::new(&SETTINGS.printer.path, SETTINGS.printer.baud_rate)
        .timeout(Duration::from_secs(10))
        .open_native()
        .expect("Init serial failed")
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // Read environment
    dotenv::dotenv().ok();
    // Initializer logger
    tracing_subscriber::fmt().pretty().init();
    // Initialize printer
    let port = init_printer_port();
    let mut bot = TelegramBot::init(port);
    info!("Started!");
    bot.print_startup_message()?;
    // Start polling messages from telegram
    loop {
        match bot.poll().await {
            Ok(None) => {}
            Ok(Some(cmd)) => {
                info!("Received {:?} from {}", cmd.kind, cmd.source.first_name);
                if let Err(why) = bot.handle(&cmd).await {
                    warn!("Bot error: {}", why);
                }
            }
            Err(why) => warn!("Bot error: {}", why),
        }
    }
}
