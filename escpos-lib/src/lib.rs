use bitflags::bitflags;
use serialport::SerialPort;

mod CMDS {
    const ESC: &str = "0x1b";
    const LF: &str = "0x0a";
    const INITIALIZE_PRINTER: char = '@';
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
    pub fn new(port: P) -> Self {
        let mut printer = Printer { port };
        printer.exec(EscPosCmd::InitializePrinter);
        printer
    }

    pub fn exec(&mut self, cmd: EscPosCmd) {
        use CMDS::{ESC, LF, INITIALIZE_PRINTER};
        match cmd {
            EscPosCmd::InitializePrinter => {
                write!(self.port, "{}{}", ESC, INITIALIZE_PRINTER);
            }
            EscPosCmd::PrintAndLineFeed => {
                write!(self.port, "{}", LF);
            }
            EscPosCmd::SelectPrintMode(mode) => {
                write!(self.port, "{}!{}", ESC, mode.bits() as char);
            }
            EscPosCmd::SelectUnderlineMode(mode) => {
                let param = match mode {
                    UnderlineMode::Off => '0',
                    UnderlineMode::OneDot => '1',
                    UnderlineMode::TwoDot => '2',
                };
                write!(self.port, "{}-{}", ESC, param);
            }
            EscPosCmd::SelectEmphasized(enable) => {
                write!(self.port, "{}E{}", ESC, if enable { '1' } else { '0' });
            }
            EscPosCmd::SelectDoubleStrike(enable) => {
                write!(self.port, "{}G{}", ESC, if enable { '1' } else { '0' });
            }
            EscPosCmd::SelectFont(font) => {
                let param = match font {
                    Font::A => '0',
                    Font::B => '1',
                    Font::C => '2',
                };
                write!(self.port, "{}M{}", ESC, param);
            }
            EscPosCmd::SelectJustification(justification) => {
                let param = match justification {
                    Justification::Left => '0',
                    Justification::Center => '1',
                    Justification::Right => '2',
                };
                write!(self.port, "{}a{}", ESC, param);
            }
            EscPosCmd::SelectPaperSensorMode(mode) => {
                todo!()
            }
            EscPosCmd::PrintAndFeedLines(lines) => {
                write!(self.port, "{}d{}", ESC, lines as char);
            }
            EscPosCmd::PrintAndReverseFeedLines(lines) => {
                write!(self.port, "{}e{}", ESC, lines as char);
            }
            _ => todo!(),
        }
    }
}

pub enum EscPosCmd {
    InitializePrinter,
    PrintAndLineFeed,
    SelectPrintMode(PrintMode),
    SelectUnderlineMode(UnderlineMode),
    SelectEmphasized(bool),
    SelectDoubleStrike(bool),
    SelectFont(Font),
    SelectJustification(Justification),
    SelectPaperSensorMode(PaperSensorMode), // FIXME: Implement
    PrintAndFeedLines(u8),
    PrintAndReverseFeedLines(u8), // FIXME: Implement
    GeneratePulse(bool),          // FIXME: Implement
    SelectPrintColor(bool),       // FIXME: Implement
    SelectCharCodeTable(CharCodeTable),
    SelectReversePrinting(bool),
    CutPaper(CutMode),
    SelectBarCodeHeight(u8),
    PrintBarCode(u8), // TODO: Actually implement
}

pub enum CutMode {
    TODO,
}

pub struct CharCodeTable {
    TODO: u8,
}

pub struct PaperSensorMode {
    TODO: u8,
}

pub enum Justification {
    Left,
    Center,
    Right,
}

pub enum Font {
    A,
    B,
    C, // TODO: Does this work with tm88iii?
}

pub enum UnderlineMode {
    Off,
    OneDot,
    TwoDot,
}

bitflags! {
    pub struct PrintMode: u8 {
        const FONT_B = 0b0000_0001;
        const EMPHASIZED = 0b0000_1000;
        const DOUBLE_HEIGHT = 0b0001_0000;
        const DOUBLE_WIDTH = 0b0010_0000;
        const UNDERLINE = 0b1000_0000;
    }
}
