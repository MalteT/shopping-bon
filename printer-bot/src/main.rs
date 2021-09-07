use escpos_lib::{FmtStr, Printer};
use futures::StreamExt;
use lazy_static::lazy_static;
use serialport::SerialPort;
use telegram_bot::{
    Api, CanSendMessage, Error as TelegramError, Message, MessageKind, UpdateKind, UpdatesStream,
    User as TelegramUser, UserId,
};
use tracing::{info, warn};

use std::{
    collections::HashMap,
    env,
    time::{Duration, Instant},
};

mod error;
mod settings;

pub use error::Error;
pub use settings::SETTINGS;

macro_rules! env_expect {
    ($env:literal) => {
        env::var($env).expect(stringify!($env not set))
    };
    ($env:literal, $default:literal) => {
        env::var($env).unwrap_or($default.into())
    };
    (parse: $env:literal) => {
        env::var($env)
            .expect(stringify!($env not set))
            .parse()
            .expect(stringify!($env contains invalid data))
    };
    (parse: $env:literal, $default:literal) => {
        env::var($env)
            .unwrap_or($default.into())
            .parse()
            .expect(stringify!($env contains invalid data))
    }
}

lazy_static! {
    static ref ADMIN_ID: UserId = {
        let id = env_expect!("PRINTER_BOT_ADMIN_ID")
            .parse()
            .expect("PRINTER_BOT_ADMIN_ID is not a number");
        UserId::new(id)
    };
    static ref USER_IDS: Vec<UserId> = {
        env_expect!("PRINTER_BOT_USER_IDS")
            .split(',')
            .map(|id| id.trim())
            .map(str::parse)
            .filter_map(|res| match res {
                Ok(nr) => Some(nr),
                Err(e) => {
                    warn!("PRINTER_BOT_USER_IDS contains non-numeric values: {}", e);
                    None
                }
            })
            .map(UserId::new)
            .collect()
    };
}

pub struct TelegramBot<P: SerialPort> {
    api: Api,
    stream: UpdatesStream,
    printer: Printer<P>,
    history: History,
}

pub struct History {
    last_print: HashMap<UserId, Instant>,
}

impl History {
    pub fn new() -> Self {
        History {
            last_print: HashMap::new(),
        }
    }
    pub fn duration_since_last_print(&self, id: &UserId) -> Option<Duration> {
        self.last_print.get(&id).map(|instant| instant.elapsed())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandKind {
    Print(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub source: TelegramUser,
    pub kind: CommandKind,
}

impl<P: SerialPort> TelegramBot<P> {
    pub fn init(port: P) -> Self {
        let token = env_expect!("PRINTER_BOT_TOKEN");
        let api = Api::new(token);
        let stream = api.stream();
        let printer = Printer::new(port).expect("Failed to initialize printer");
        let history = History::new();
        TelegramBot {
            api,
            stream,
            printer,
            history,
        }
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

    pub async fn handle(&mut self, cmd: &Command) -> Result<(), Error> {
        let Command { source, kind } = cmd;
        match kind {
            CommandKind::Print(text) => self.handle_print_cmd(source, text).await,
        }
    }

    async fn handle_print_cmd(&mut self, source: &TelegramUser, text: &String) -> Result<(), Error> {
        if self.is_printing_allowed(source.id) {
            if self.is_print_length_allowed(source.id, text.len()) {
                self.print_message(source, text)?;
                self.send(source.id, "🖨️✅").await
            } else {
                self.send(source.id, "🖨️❌ That message is too long!")
                    .await?;
                Ok(())
            }
        } else {
            self.send(source.id, "🖨️❌ You may not print now!").await?;
            Ok(())
        }
    }

    fn print_message(&mut self, source: &TelegramUser, text: &String) -> Result<(), Error> {
        let name = if let Some(ref last_name) = source.last_name {
            format!(" {} {} ", source.first_name, last_name)
        } else {
            format!(" {} ", source.first_name)
        };
        let formatted = format!("{}: {}\n", name.reverse(), text);
        self.printer
            .write_and_cut(formatted)
            .map_err(Error::Printing)
    }

    fn is_print_length_allowed(&self, id: UserId, len: usize) -> bool {
        if let Some(role) = SETTINGS.get_role(id) {
            len <= role.max_print_len
        } else {
            false
        }
    }

    fn is_printing_allowed(&self, id: UserId) -> bool {
        if let Some(role) = SETTINGS.get_role(id) {
            let expected_dur = Duration::from_secs(60 * role.minutes_between_prints as u64);
            if let Some(real_dur) = self.history.duration_since_last_print(&id) {
                expected_dur > real_dur
            } else {
                true
            }
        } else {
            false
        }
    }

    async fn send(&mut self, id: UserId, text: &str) -> Result<(), Error> {
        let msg = id.text(text);
        self.api.send(msg).await.map_err(Error::SendingMessage)?;
        Ok(())
    }
}

fn message_to_command(message: Message) -> Option<Command> {
    // We only care about text messages
    let kind = if let MessageKind::Text { data, .. } = message.kind {
        Some(CommandKind::Print(data))
    } else {
        None
    };
    kind.map(|kind| Command {
        kind,
        source: message.from,
    })
}

fn init_printer_port() -> impl SerialPort {
    let path = env_expect!("PRINTER_BOT_PRINTER_PATH");
    let baud_rate = env_expect!(parse: "PRINTER_BOT_PRINTER_BAUD_RATE", "9600");
    serialport::new(path, baud_rate)
        .timeout(Duration::from_secs(10))
        .open_native()
        .expect("Init serial failed")
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), telegram_bot::Error> {
    // Read environment
    dotenv::dotenv().ok();
    // Initializer logger
    tracing_subscriber::fmt().pretty().init();
    // Initialize printer
    let port = init_printer_port();
    let mut bot = TelegramBot::init(port);
    info!("Started!");
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
