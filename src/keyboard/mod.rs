use crate::{
    interrupts::{PICInterrupt, PICS},
    prelude::*,
};
use futures::stream::StreamExt;
use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

mod buffer;
mod layout;

use buffer::{sync, PopBufferStream, POP_BUFFER, PUSH_BUFFER};
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
    // 1. read the pressed caracter into PUSH_BUFFER
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
            PUSH_BUFFER.lock().push(ascii);
        }
        KeyType::Backspace => {
            PUSH_BUFFER.lock().pop_end();
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
    // 2. try to sync PUSH_BUFFER and POP_BUFFER (sometimes we can't cuz POP_BUFFER is locked
    //    somewhere else)
    if let Some(mut pop) = POP_BUFFER.try_lock() {
        sync(&mut PUSH_BUFFER.lock(), &mut pop);
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(PICInterrupt::Keyboard as u8)
    };
}

/// Reads characters from the keyboard buffer into string until \n or is reached. Consumes `\n`.
/// Returns number of read characters.
pub async fn scanf(string: &mut String) -> usize {
    // while POP_BUFFER.lock().is_empty() {} // wait until buffer has chars

    let mut len = 0;

    let mut stream = PopBufferStream::new();

    while let Some(c) = stream.next().await {
        print!("{c}");
        if c == '\n' {
            break;
        }
        string.push(c);
        len += 1;
    }
    len
}

// fn scancode2ascii(scancode: u8) -> char {
//     let sc2ascii = [ESC'a', 'b'];
//     'a'
// }
