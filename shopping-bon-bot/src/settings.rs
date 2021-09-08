use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use telegram_bot::UserId;
use tracing::{info, warn};

use std::{
    fs::{self, File},
    io::Write,
    marker::PhantomData,
    path::PathBuf,
};

use crate::Error;

lazy_static! {
    pub static ref SETTINGS: Settings =
        Settings::load_or_create_default().expect("Failed to open settings");
    static ref SETTINGS_PATH: PathBuf = dirs::config_dir()
        .expect("Could not determine config path. Adjust XDG_CONFIG_DIR.")
        .join("printer-bot/settings.toml");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub read: bool,
    pub write: bool,
    pub print: bool,
    #[serde(default, skip_serializing)]
    _cannot_create: PhantomData<()>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub role: String,
    #[serde(default, skip_serializing)]
    _cannot_create: PhantomData<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub roles: Vec<Role>,
    pub users: Vec<User>,
    pub printer: Printer,
    pub bot: Bot,
    #[serde(default, skip_serializing)]
    _cannot_create: PhantomData<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    pub token: String,
    #[serde(default, skip_serializing)]
    _cannot_create: PhantomData<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Printer {
    pub path: String,
    pub baud_rate: u32,
    #[serde(default, skip_serializing)]
    _cannot_create: PhantomData<()>,
}

impl Settings {
    pub fn get_role(&self, id: UserId) -> Option<&Role> {
        self.get_user(id).and_then(|user| {
            let role_name = &user.role;
            self.roles.iter().find(|role| role.name == *role_name)
        })
    }

    pub fn get_user(&self, id: UserId) -> Option<&User> {
        self.users.iter().find(|user| {
            let user_id = user.id.parse().expect("User ID not an int");
            id == UserId::new(user_id)
        })
    }

    fn load_or_create_default() -> Result<Self, Error> {
        match fs::read_to_string(&*SETTINGS_PATH).map_err(Error::OpeningSettingsFile) {
            Ok(content) => toml::from_str(&content).map_err(Error::ParsingSettingsFile),
            Err(why) => {
                warn!("{}", why);
                info!("creating default settings");
                let settings = Settings {
                    roles: vec![Role {
                        name: String::from("admin"),
                        read: true,
                        write: true,
                        print: true,
                        _cannot_create: PhantomData,
                    }],
                    users: vec![User {
                        id: String::from("228223333"),
                        role: String::from("admin"),
                        _cannot_create: PhantomData,
                    }],
                    printer: Printer {
                        path: String::from("/dev/null"),
                        baud_rate: 9600,
                        _cannot_create: PhantomData,
                    },
                    bot: Bot {
                        token: String::from("[YOUR TELEGRAM BOT TOKEN]"),
                        _cannot_create: PhantomData,
                    },
                    _cannot_create: PhantomData,
                };
                let content =
                    toml::to_string_pretty(&settings).expect("BUG: Invalid default config");
                let mut file =
                    File::create(&*SETTINGS_PATH).map_err(Error::CreatingSettingsFile)?;
                write!(file, "{}", content).map_err(Error::CreatingSettingsFile)?;
                Ok(settings)
            }
        }
    }
}
