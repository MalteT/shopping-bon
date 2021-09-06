use serialport::SerialPort;

pub struct Printer<P> where P: SerialPort {
    port: P,
}

impl<P> Printer<P> where P: SerialPort {
    pub fn new(port: P) -> Self {
        let printer = Printer { port };
        printer.exec(EscPosCmd::InitializePrinter);
        printer
    }

    pub fn exec(&self, cmd: EscPosCmd) {
        match cmd {
            EscPosCmd::InitializePrinter => {
                write!(self.port, "\x1b@");
            }
            _ => todo!()
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
    GeneratePulse(bool), // FIXME: Implement
    SelectPrintColor(bool), // FIXME: Implement
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

pub struct PrintMode {
    TODO: u8,
}
