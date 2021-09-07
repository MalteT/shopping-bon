use std::fmt;

use crate::cmds::UnderlineMode;

use super::{cmds::PrintMode, EscPosCmd};

/// String with applied formatting.
/// This can be used to preformat strings before printing.
#[derive(Debug, Default)]
pub struct FormattedStr<S> {
    mode: PrintMode,
    reverse_color: bool,
    text: S,
    bold: bool,
    underline: UnderlineMode,
}

pub trait FmtStr<S> {
    fn emph(self) -> FormattedStr<S>;
    fn higher(self) -> FormattedStr<S>;
    fn wider(self) -> FormattedStr<S>;
    fn underline1(self) -> FormattedStr<S>;
    fn underline2(self) -> FormattedStr<S>;
    fn reverse(self) -> FormattedStr<S>;
    fn small(self) -> FormattedStr<S>;
}

impl<S: fmt::Display> fmt::Display for FormattedStr<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EscPosCmd::*;
        macro_rules! maybe_print {
            ($q:expr, $val:expr) => {
                if $q {
                    $val
                } else {
                    EscPosCmd::Text("")
                }
            };
        }
        write!(
            f,
            "{}{}{}{}{}{}{}",
            maybe_print!(self.reverse_color, SelectReversePrinting(true)),
            maybe_print!(
                self.underline != UnderlineMode::Off,
                SelectUnderlineMode(self.underline)
            ),
            SelectPrintMode(self.mode),
            self.text,
            SelectPrintMode(PrintMode::empty()),
            maybe_print!(
                self.underline != UnderlineMode::Off,
                SelectUnderlineMode(UnderlineMode::Off)
            ),
            maybe_print!(self.reverse_color, SelectReversePrinting(false)),
        )
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

    fn underline1(self) -> FormattedStr<&'s str> {
        FormattedStr {
            underline: UnderlineMode::OneDot,
            text: self,
            ..Default::default()
        }
    }

    fn underline2(self) -> FormattedStr<&'s str> {
        FormattedStr {
            underline: UnderlineMode::TwoDot,
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

    fn small(self) -> FormattedStr<&'s str> {
        FormattedStr {
            mode: PrintMode::FONT_B,
            text: self,
            ..Default::default()
        }
    }
}

impl<S> FmtStr<S> for FormattedStr<S> {
    fn emph(self) -> FormattedStr<S> {
        FormattedStr {
            mode: self.mode | PrintMode::EMPHASIZED,
            ..self
        }
    }

    fn higher(self) -> FormattedStr<S> {
        FormattedStr {
            mode: self.mode | PrintMode::DOUBLE_HEIGHT,
            ..self
        }
    }

    fn wider(self) -> FormattedStr<S> {
        FormattedStr {
            mode: self.mode | PrintMode::DOUBLE_WIDTH,
            ..self
        }
    }

    fn underline1(self) -> FormattedStr<S> {
        FormattedStr {
            underline: UnderlineMode::OneDot,
            ..self
        }
    }

    fn underline2(self) -> FormattedStr<S> {
        FormattedStr {
            underline: UnderlineMode::TwoDot,
            ..self
        }
    }

    fn reverse(self) -> FormattedStr<S> {
        FormattedStr {
            reverse_color: true,
            ..self
        }
    }

    fn small(self) -> FormattedStr<S> {
        FormattedStr {
            mode: self.mode | PrintMode::FONT_B,
            ..self
        }
    }
}
