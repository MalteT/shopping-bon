use thiserror::Error;

use std::io;

#[derive(Debug, Error)]
pub enum Error {
    #[error("could not open settings file")]
    OpeningSettingsFile(#[source] io::Error),
    #[error("could not parse settings file to toml")]
    ParsingSettingsFile(#[source] toml::de::Error),
    #[error("could not create settings file")]
    CreatingSettingsFile(#[source] io::Error),
    #[error("could not send message")]
    SendingMessage(#[source] telegram_bot::Error),
    #[error("printing error")]
    Printing(#[source] io::Error),
}
