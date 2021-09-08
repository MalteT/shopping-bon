use tracing::{error, info, warn};

use std::error::Error;

pub trait ResultExt<R, E: Error> {
    fn log_info(self, msg: &str) -> Option<R>;
    fn log_warn(self, msg: &str) -> Option<R>;
    fn log_err(self, msg: &str) -> Option<R>;
}

impl<R, E: Error> ResultExt<R, E> for Result<R, E> {
    fn log_err(self, msg: &str) -> Option<R> {
        match self {
            Ok(val) => Some(val),
            Err(why) => {
                error!("{}: {}", msg, why);
                None
            }
        }
    }

    fn log_info(self, msg: &str) -> Option<R> {
        match self {
            Ok(val) => Some(val),
            Err(why) => {
                info!("{}: {}", msg, why);
                None
            }
        }
    }

    fn log_warn(self, msg: &str) -> Option<R> {
        match self {
            Ok(val) => Some(val),
            Err(why) => {
                warn!("{}: {}", msg, why);
                None
            }
        }
    }
}
