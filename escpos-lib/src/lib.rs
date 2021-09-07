use serialport::SerialPort;

use std::io::Result as IoResult;

mod cmds;
mod format;

use cmds::{CutMode, EscPosCmd};
pub use format::{FmtStr, FormattedStr};

/// Special characters
mod chars {
    pub const ESC: char = '\x1b';
    pub const LF: char = '\x0a';
    pub const GS: char = '\x1d';
    pub const INITIALIZE_PRINTER: char = '@';
}

pub struct Printer<P>
where
    P: SerialPort,
{
    port: P,
}

impl<P> Printer<P>
where
    P: SerialPort,
{
    pub fn new(port: P) -> IoResult<Self> {
        let mut printer = Printer { port };
        printer.exec(EscPosCmd::InitializePrinter)?;
        Ok(printer)
    }

    pub fn print_test_page(&mut self) -> IoResult<()> {
        let header = format!("{}\nDies ist ein Test\n", " TEST ".reverse());
        let format_strings = vec![
            "Emphasized".emph(),
            "Higher".higher(),
            "Wider".wider(),
            "Underlined1".underline1(),
            "Reversed".reverse(),
            "Small".small(),
            "Emph Higher".emph().higher(),
            "Emph Wider".emph().wider(),
            "Emph Underlined1".emph().underline1(),
            "Emph Reversed".emph().reverse(),
            "Emph Small".emph().small(),
            "Higher Wider".higher().wider(),
            "Higher Underlined1".higher().underline1(),
            "Higher Reversed".higher().reverse(),
            "Higher Small".higher().small(),
            "Wider Underlined1".wider().underline1(),
            "Wider Reversed".wider().reverse(),
            "Wider Small".wider().small(),
            "Underlined1 Reversed".underline1().reverse(),
            "Underlined1 Small".underline1().small(),
            "Reversed Small".reverse().small(),
        ];
        for string in format_strings {
            self.write(&format!(" - {}\n", string))?
        }
        self.write(header)?;
        self.exec(EscPosCmd::PrintAndFeedLines(5))?;
        self.exec(EscPosCmd::CutPaper(CutMode::Full))?;
        Ok(())
    }

    pub fn write<S: Into<String>>(&mut self, text: S) -> IoResult<()> {
        write!(self.port, "{}", EscPosCmd::Text(&text.into()))
    }

    pub fn exec(&mut self, cmd: EscPosCmd) -> IoResult<()> {
        write!(self.port, "{}", cmd)
    }
}
