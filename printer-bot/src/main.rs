use escpos_lib::{FmtStr, Printer};
use futures::StreamExt;
use lazy_static::lazy_static;
use serialport::SerialPort;
use telegram_bot::{
    Api, CanSendMessage, Error as TelegramError, Message, MessageKind, UpdateKind, UpdatesStream,
    User, UserId,
};
use tracing::{error, info, warn};

use std::{env, time::Duration};

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
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandKind {
    Print(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub source: User,
    pub kind: CommandKind,
}

impl<P: SerialPort> TelegramBot<P> {
    pub fn init(port: P) -> Self {
        let token = env_expect!("PRINTER_BOT_TOKEN");
        let api = Api::new(token);
        let stream = api.stream();
        let printer = Printer::new(port).expect("Failed to initialize printer");
        TelegramBot {
            api,
            stream,
            printer,
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

    pub async fn handle(&mut self, cmd: &Command) -> Result<(), TelegramError> {
        let Command { source, kind } = cmd;
        match kind {
            CommandKind::Print(text) => {
                let allowance = if source.id == *ADMIN_ID {
                    Some(usize::MAX)
                } else if USER_IDS.contains(&source.id) {
                    Some(200)
                } else {
                    None
                };
                if let Some(max_length) = allowance {
                    if text.len() <= max_length {
                        let name = if let Some(ref last_name) = source.last_name {
                            format!(" {} {} ", source.first_name, last_name)
                        } else {
                            format!(" {} ", source.first_name)
                        };
                        let formatted = format!("{}: {}\n", name.reverse(), text);
                        if let Err(why) = self.printer.write(formatted) {
                            error!("Failed to print message: {}", why);
                        }
                    } else {
                        let msg = source.id.text(format!(
                            "You're only allowed to print {} characters, your message had {}!",
                            max_length,
                            text.len()
                        ));
                        if let Err(why) = self.api.send(msg).await {
                            error!("Failed to send message: {}", why);
                        }
                    }
                } else {
                    let msg = source
                        .id
                        .text("My mother always said not to talk to strangers..");
                    if let Err(why) = self.api.send(msg).await {
                        error!("Failed to send message: {}", why);
                    }
                }
            }
        }
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
