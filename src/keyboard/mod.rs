use crate::{
    interrupts::{PICInterrupt, PICS},
    prelude::*,
    vga::VGA,
};
use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

mod buffer;
mod layout;

use buffer::KEYBOARD_BUFFER;
use layout::{KeyType, Layout, Layoutable};

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new()); // TODO: add layout to keyboard
}

struct Keyboard {
    shift_pressed: bool,
    caps_lock_on: bool,
    layout: Layout,
}

impl Keyboard {
    fn new() -> Keyboard {
        Keyboard {
            caps_lock_on: false,
            shift_pressed: false,
            layout: Layout::Moonlander,
        }
    }

    fn caps_lock(&mut self) {
        self.caps_lock_on = !self.caps_lock_on;
    }

    fn shift(&mut self) {
        self.shift_pressed = true;
    }

    fn shift_released(&mut self) {
        self.shift_pressed = false;
    }

    // see if we need to uppercase letter
    fn to_ascii(&self, scancode: u8) -> char {
        let ascii = self.layout.to_ascii(scancode);

        match (self.caps_lock_on, self.shift_pressed) {
            (true, true) => ascii,
            (true, false) => ascii.to_ascii_uppercase(),
            (false, true) => ascii.to_ascii_uppercase(),
            (false, false) => ascii,
        }
    }
}

/// Handles an interrupt for a keyboard event (should not lock VGA since it will likely deadlock)
pub extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: InterruptStackFrame) {
    let scancode: u8 = unsafe {
        let mut port = Port::new(0x60);
        port.read()
    };

    let keytype = KEYBOARD.lock().layout.to_keytype(scancode);
    match keytype {
        KeyType::Shift => KEYBOARD.lock().shift(),
        KeyType::ShiftReleased => KEYBOARD.lock().shift_released(),
        KeyType::CapsLock => KEYBOARD.lock().caps_lock(),
        KeyType::Letter => {
            let ascii = KEYBOARD.lock().to_ascii(scancode);
            // acquire lock for buffer
            // put char in buffer
            KEYBOARD_BUFFER.lock().push(ascii);
        }
        KeyType::Backspace => {
            if KEYBOARD_BUFFER.lock().is_empty() {
                // TODO: need to lock VGA here but can't (deadlock)
            } else {
                KEYBOARD_BUFFER.lock().pop_end();
            }
        }
        KeyType::ESC => (),
        KeyType::Ctrl => (),
        KeyType::Alt => (),
        KeyType::ArrowUp => (),
        KeyType::ArrowDown => (),
        KeyType::ArrowLeft => (),
        KeyType::ArrowRight => (),
        KeyType::Unknown => (), // do nothing
    }
    //     let ascii_key = scancode2ascii(key);
    //     logf!(Level::Info, "{ascii_key}"); // logf does not insert newlines

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(PICInterrupt::Keyboard as u8)
    };
}

/// Reads characters from the keyboard buffer into string until \n or is reached. Consumes `\n`.
/// Returns number of read characters.
pub async fn scanf(string: &mut String) -> usize {
    // while KEYBOARD_BUFFER.lock().is_empty() {} // wait until buffer has chars

    let mut len = 0;

    // TODO: implement futures_util::Stream for KEYBOARD_BUFFER and use StreamExt's functions
    let mut c = KEYBOARD_BUFFER.lock().pop();
    // while ome(c) = stream.next().await
    while c != '\n' {
        if c == '\0' {
            // TODO: yield here
            continue;
        }
        string.push(c);
        len += 1;
        c = KEYBOARD_BUFFER.lock().pop();
    }

    len
}

// fn scancode2ascii(scancode: u8) -> char {
//     let sc2ascii = [ESC'a', 'b'];
//     'a'
// }
