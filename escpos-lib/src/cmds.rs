use bitflags::bitflags;

use std::fmt;

use super::chars;

pub enum EscPosCmd<'s> {
    InitializePrinter,
    PrintAndLineFeed,
    SelectPrintMode(PrintMode),
    SelectUnderlineMode(UnderlineMode),
    SelectEmphasized(bool),
    SelectDoubleStrike(bool),
    SelectFont(Font),
    SelectJustification(Justification),
    SelectPaperSensorMode(PaperSensorMode),
    PrintAndFeedLines(u8),
    PrintAndReverseFeedLines(u8),
    GeneratePulse(bool),
    SelectPrintColor(bool),
    SelectCharCodeTable(CharCodeTable),
    SelectReversePrinting(bool),
    CutPaper(CutMode),
    SelectBarCodeHeight(u8),
    PrintBarCode(u8),
    Text(&'s str),
}

pub enum CutMode {
    Partial,
    Full,
}

pub enum CharCodeTable {
    // USA: Standard Europe
    PC437,
    Katakana,
    // Multilingual
    PC850,
    // Portuguese
    PC860,
    // Canadian-French
    PC863,
    // Nordic
    PC865,
    WPC1252,
    // Cyrillic #2
    PC866,
    // Latin 2
    PC852,
    // Euro
    PC858,
    ThaiCharCode42,
    ThaiCharCode11,
    ThaiCharCode13,
    ThaiCharCode14,
    ThaiCharCode16,
    ThaiCharCode17,
    ThaiCharCode18,
    UserDefined1,
    UserDefined2,
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

impl Default for PrintMode {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for EscPosCmd<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use chars::{ESC, GS, INITIALIZE_PRINTER, LF};
        match self {
            EscPosCmd::InitializePrinter => {
                write!(f, "{}{}", ESC, INITIALIZE_PRINTER)
            }
            EscPosCmd::PrintAndLineFeed => {
                write!(f, "{}", LF)
            }
            EscPosCmd::SelectPrintMode(mode) => {
                write!(f, "{}!{}", ESC, mode.bits() as char)
            }
            EscPosCmd::SelectUnderlineMode(mode) => {
                let param = match mode {
                    UnderlineMode::Off => '\x00',
                    UnderlineMode::OneDot => '\x01',
                    UnderlineMode::TwoDot => '\x02',
                };
                write!(f, "{}-{}", ESC, param)
            }
            EscPosCmd::SelectEmphasized(enable) => {
                write!(f, "{}E{}", ESC, if *enable { '1' } else { '0' })
            }
            EscPosCmd::SelectDoubleStrike(enable) => {
                write!(f, "{}G{}", ESC, if *enable { '1' } else { '0' })
            }
            EscPosCmd::SelectFont(font) => {
                let param = match font {
                    Font::A => '0',
                    Font::B => '1',
                    Font::C => '2',
                };
                write!(f, "{}M{}", ESC, param)
            }
            EscPosCmd::SelectJustification(justification) => {
                let param = match justification {
                    Justification::Left => '0',
                    Justification::Center => '1',
                    Justification::Right => '2',
                };
                write!(f, "{}a{}", ESC, param)
            }
            EscPosCmd::SelectPaperSensorMode(_mode) => {
                todo!()
            }
            EscPosCmd::PrintAndFeedLines(lines) => {
                write!(f, "{}d{}", ESC, *lines as char)
            }
            EscPosCmd::PrintAndReverseFeedLines(lines) => {
                write!(f, "{}e{}", ESC, *lines as char)
            }
            EscPosCmd::GeneratePulse(_) => {
                todo!()
            }
            EscPosCmd::SelectPrintColor(second_color) => {
                write!(f, "{}r{}", ESC, if *second_color { '1' } else { '0' })
            }
            EscPosCmd::SelectCharCodeTable(table) => {
                let code = match table {
                    CharCodeTable::PC437 => 0_u8,
                    CharCodeTable::Katakana => 1,
                    CharCodeTable::PC850 => 2,
                    CharCodeTable::PC860 => 3,
                    CharCodeTable::PC863 => 4,
                    CharCodeTable::PC865 => 5,
                    CharCodeTable::WPC1252 => 16,
                    CharCodeTable::PC866 => 17,
                    CharCodeTable::PC852 => 18,
                    CharCodeTable::PC858 => 19,
                    CharCodeTable::ThaiCharCode42 => 20,
                    CharCodeTable::ThaiCharCode11 => 21,
                    CharCodeTable::ThaiCharCode13 => 22,
                    CharCodeTable::ThaiCharCode14 => 23,
                    CharCodeTable::ThaiCharCode16 => 24,
                    CharCodeTable::ThaiCharCode17 => 25,
                    CharCodeTable::ThaiCharCode18 => 26,
                    CharCodeTable::UserDefined1 => 254,
                    CharCodeTable::UserDefined2 => 255,
                };
                write!(f, "{}t{}", ESC, code as char)
            }
            EscPosCmd::SelectReversePrinting(enable) => {
                write!(f, "{}B{}", GS, if *enable { '1' } else { '0' })
            }
            EscPosCmd::CutPaper(mode) => {
                let param = match mode {
                    CutMode::Full => '0',
                    CutMode::Partial => '1',
                };
                write!(f, "{}V{}", GS, param)
            }
            EscPosCmd::SelectBarCodeHeight(height) => {
                write!(f, "{}h{}", GS, height)
            }
            EscPosCmd::PrintBarCode(_bar_code) => {
                todo!()
            }
            EscPosCmd::Text(text) => {
                write!(f, "{}", text)
            }
        }
    }
}
