use std::fmt;

use super::{cmds::PrintMode, EscPosCmd};

/// String with applied formatting.
/// This can be used to preformat strings before printing.
#[derive(Debug, Default)]
pub struct FormattedStr<S> {
    mode: PrintMode,
    reverse_color: bool,
    text: S,
}

pub trait FmtStr<S> {
    fn emph(self) -> FormattedStr<S>;
    fn higher(self) -> FormattedStr<S>;
    fn wider(self) -> FormattedStr<S>;
    fn underline(self) -> FormattedStr<S>;
    fn reverse(self) -> FormattedStr<S>;
}

impl<S: fmt::Display> fmt::Display for FormattedStr<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.reverse_color {
            write!(f, "{}", EscPosCmd::SelectReversePrinting(true))?;
        }
        write!(
            f,
            "{}{}{}",
            EscPosCmd::SelectPrintMode(self.mode),
            self.text,
            EscPosCmd::SelectPrintMode(PrintMode::empty())
        )?;
        if self.reverse_color {
            write!(f, "{}", EscPosCmd::SelectReversePrinting(false))?;
        }
        Ok(())
    }
}

impl<'s> FmtStr<&'s str> for &'s str {
    fn emph(self) -> FormattedStr<&'s str> {
        FormattedStr {
            mode: PrintMode::EMPHASIZED,
            text: self,
            ..Default::default()
        }
    }

    fn higher(self) -> FormattedStr<&'s str> {
        FormattedStr {
            mode: PrintMode::DOUBLE_HEIGHT,
            text: self,
            ..Default::default()
        }
    }

    fn wider(self) -> FormattedStr<&'s str> {
        FormattedStr {
            mode: PrintMode::DOUBLE_WIDTH,
            text: self,
            ..Default::default()
        }
    }

    fn underline(self) -> FormattedStr<&'s str> {
        FormattedStr {
            mode: PrintMode::UNDERLINE,
            text: self,
            ..Default::default()
        }
    }

    fn reverse(self) -> FormattedStr<&'s str> {
        FormattedStr {
            reverse_color: true,
            text: self,
            ..Default::default()
        }
    }
}
