use serialport::SerialPort;

use std::io::Result as IoResult;

mod cmds;
mod format;

use cmds::{CutMode, EscPosCmd};
pub use format::{FormattedStr, FmtStr};

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
        let text = format!("{}: This is a test!", "Malte".reverse());
        self.write(text)?;
        self.exec(EscPosCmd::PrintAndFeedLines(10))?;
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
