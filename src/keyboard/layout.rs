use crate::prelude::*;

static UNDEFINED_KEY: char = '?'; // this is a block in code page 437

#[derive(Debug, Clone, Copy)]
pub enum KeyType {
    ESC,
    Ctrl,
    Shift,
    ShiftReleased,
    Alt,
    CapsLock,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Ascii,
    Unknown,
}

pub enum Layout {
    Moonlander,
}

pub trait Layoutable {
    fn to_keytype(&self, scancode: u8) -> KeyType;
    fn to_ascii(&self, scancode: u8) -> char;
}

impl Layoutable for Layout {
    fn to_keytype(&self, scancode: u8) -> KeyType {
        match self {
            Layout::Moonlander => moonlander_to_keytype(scancode),
        }
    }
    fn to_ascii(&self, scancode: u8) -> char {
        match self {
            Layout::Moonlander => moonlander_to_ascii(scancode),
        }
    }
}

// MOONLANDER Layout
fn moonlander_to_keytype(scancode: u8) -> KeyType {
    log!(
        Level::Debug,
        "moonlander_to_keytype called with scancode {scancode}"
    );
    let ascii = [
        02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 14, // 14 == backspace
        30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20, 22, 47, 17,
        45, 21, 44, 57, // space
        28, // newline
    ];

    if ascii.contains(&scancode) {
        return KeyType::Ascii; // also includes space and newline
    }

    match scancode {
        1 => KeyType::ESC,
        // 14 => KeyType::Backspace, (handled above)
        29 => KeyType::Ctrl,
        42 => KeyType::Shift,
        56 => KeyType::Alt,
        72 => KeyType::ArrowUp,
        80 => KeyType::ArrowDown,
        77 => KeyType::ArrowLeft,
        75 => KeyType::ArrowRight,
        170 => KeyType::ShiftReleased,
        58 => KeyType::CapsLock,
        _ => KeyType::Unknown,
    }
}

fn moonlander_to_ascii(scancode: u8) -> char {
    log!(
        Level::Debug,
        "moonlander_to_ascii called with scancode {scancode}"
    );
    match scancode {
        02 => '1',
        03 => '2',
        04 => '3',
        05 => '4',
        06 => '5',
        07 => '6',
        08 => '7',
        09 => '8',
        10 => '9',
        11 => '0',
        14 => 8 as char, // backspace (backspace is ascii 8)
        30 => 'a',
        48 => 'b',
        46 => 'c',
        32 => 'd',
        18 => 'e',
        33 => 'f',
        34 => 'g',
        35 => 'h',
        23 => 'i',
        36 => 'j',
        37 => 'k',
        38 => 'l',
        50 => 'm',
        49 => 'n',
        24 => 'o',
        25 => 'p',
        16 => 'q',
        19 => 'r',
        31 => 's',
        20 => 't',
        22 => 'u',
        47 => 'v',
        17 => 'w',
        45 => 'x',
        21 => 'y',
        44 => 'z',
        57 => ' ',
        28 => '\n',
        _ => UNDEFINED_KEY,
    }
}
