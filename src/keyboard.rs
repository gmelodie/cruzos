use x86_64::{
    instructions::port::Port,
    structures::idt::InterruptStackFrame,
};
use crate::{
    prelude::*,
    interrupts::{PICS, PICInterrupt},
};


lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new()); // TODO: add layout to keyboard
}

static UNDEFINED_KEY: char = '?'; // this is a block in code page 437

struct Keyboard {
    shift_pressed: bool,
}

impl Keyboard {
    fn new() -> Keyboard {
        Keyboard {
            shift_pressed: false
        }
    }

    fn shift(&mut self) {
        self.shift_pressed = true;
    }

    fn shift_released(&mut self) {
        self.shift_pressed = false;
    }

    fn to_ascii(&self, scancode: u8) -> char {
        let mut ascii = match scancode {
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
        };
        if self.shift_pressed {
            ascii = ascii.to_ascii_uppercase();
        }
        ascii
    }
}

#[repr(u8)]
enum KeyType {
    ESC = 1,
    Ctrl = 29,
    Shift = 42,
    Alt = 56,
    ArrowUp = 72,
    ArrowDown = 80,
    ArrowLeft = 77,
    ArrowRight = 75,
    ShiftReleased = 170,
    // TODO: backspace
    Letter,
    Unknown,
}

fn is_letter_scancode(scancode: u8) -> bool {
    // TODO: unify this with to_ascii list
    match scancode {
            30 => true,
            48 => true,
            46 => true,
            32 => true,
            18 => true,
            33 => true,
            34 => true,
            35 => true,
            23 => true,
            36 => true,
            37 => true,
            38 => true,
            50 => true,
            49 => true,
            24 => true,
            25 => true,
            16 => true,
            19 => true,
            31 => true,
            20 => true,
            22 => true,
            47 => true,
            17 => true,
            45 => true,
            21 => true,
            44 => true,
            57 => true, // space
            28 => true, // newline
            _ => false
    }
}

impl KeyType {
    fn from_scancode(scancode: u8) -> KeyType {
        if is_letter_scancode(scancode) {
            return KeyType::Letter; // also includes space and newline
        }
        match scancode {
            1 => KeyType::ESC,
            29 => KeyType::Ctrl,
            42 => KeyType::Shift,
            56 => KeyType::Alt,
            72 => KeyType::ArrowUp,
            80 => KeyType::ArrowDown,
            77 => KeyType::ArrowLeft,
            75 => KeyType::ArrowRight,
            170 => KeyType::ShiftReleased,
            _ => KeyType::Unknown,
        }
    }
}

pub extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: InterruptStackFrame) {
    let scancode: u8 = unsafe {
        let mut port = Port::new(0x60);
        port.read()
    };
    log!(Level::Debug, "Got input from keyboard: {scancode}");

    match KeyType::from_scancode(scancode) {
        KeyType::Shift => KEYBOARD.lock().shift(),
        KeyType::ShiftReleased => KEYBOARD.lock().shift_released(),
        KeyType::Letter => logf!(Level::Info, "{}", KEYBOARD.lock().to_ascii(scancode)),
        KeyType::Unknown => (), // do nothing
        _ => () // do nothing
    }
//     let ascii_key = scancode2ascii(key);
//     logf!(Level::Info, "{ascii_key}"); // logf does not insert newlines
    // TODO: keyboard buffer

    unsafe {PICS.lock().notify_end_of_interrupt(PICInterrupt::Keyboard as u8)};
}


// fn scancode2ascii(scancode: u8) -> char {
//     let sc2ascii = [ESC'a', 'b'];
//     'a'
// }
