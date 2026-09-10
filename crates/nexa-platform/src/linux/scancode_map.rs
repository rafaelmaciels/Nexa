/// Constantes evdev correspondentes aos códigos de tecla do kernel Linux (linux/input-event-codes.h)
pub mod evdev_keys {
    pub const KEY_ESC: u16 = 1;
    pub const KEY_1: u16 = 2;
    pub const KEY_2: u16 = 3;
    pub const KEY_3: u16 = 4;
    pub const KEY_4: u16 = 5;
    pub const KEY_5: u16 = 6;
    pub const KEY_6: u16 = 7;
    pub const KEY_7: u16 = 8;
    pub const KEY_8: u16 = 9;
    pub const KEY_9: u16 = 10;
    pub const KEY_0: u16 = 11;
    pub const KEY_MINUS: u16 = 12;
    pub const KEY_EQUAL: u16 = 13;
    pub const KEY_BACKSPACE: u16 = 14;
    pub const KEY_TAB: u16 = 15;
    pub const KEY_Q: u16 = 16;
    pub const KEY_W: u16 = 17;
    pub const KEY_E: u16 = 18;
    pub const KEY_R: u16 = 19;
    pub const KEY_T: u16 = 20;
    pub const KEY_Y: u16 = 21;
    pub const KEY_U: u16 = 22;
    pub const KEY_I: u16 = 23;
    pub const KEY_O: u16 = 24;
    pub const KEY_P: u16 = 25;
    pub const KEY_ENTER: u16 = 28;
    pub const KEY_LEFTCTRL: u16 = 29;
    pub const KEY_A: u16 = 30;
    pub const KEY_S: u16 = 31;
    pub const KEY_D: u16 = 32;
    pub const KEY_F: u16 = 33;
    pub const KEY_G: u16 = 34;
    pub const KEY_H: u16 = 35;
    pub const KEY_J: u16 = 36;
    pub const KEY_K: u16 = 37;
    pub const KEY_L: u16 = 38;
    pub const KEY_LEFTSHIFT: u16 = 42;
    pub const KEY_Z: u16 = 44;
    pub const KEY_X: u16 = 45;
    pub const KEY_C: u16 = 46;
    pub const KEY_V: u16 = 47;
    pub const KEY_B: u16 = 48;
    pub const KEY_N: u16 = 49;
    pub const KEY_M: u16 = 50;
    pub const KEY_RIGHTSHIFT: u16 = 54;
    pub const KEY_LEFTALT: u16 = 56;
    pub const KEY_SPACE: u16 = 57;
    pub const KEY_CAPSLOCK: u16 = 58;
    pub const KEY_F1: u16 = 59;
    pub const KEY_F2: u16 = 60;
    pub const KEY_F3: u16 = 61;
    pub const KEY_F4: u16 = 62;
    pub const KEY_F5: u16 = 63;
    pub const KEY_F6: u16 = 64;
    pub const KEY_F7: u16 = 65;
    pub const KEY_F8: u16 = 66;
    pub const KEY_F9: u16 = 67;
    pub const KEY_F10: u16 = 68;
    pub const KEY_NUMLOCK: u16 = 69;
    pub const KEY_SCROLLLOCK: u16 = 70;
    pub const KEY_F11: u16 = 87;
    pub const KEY_F12: u16 = 88;
    pub const KEY_RIGHTCTRL: u16 = 97;
    pub const KEY_RIGHTALT: u16 = 100; // AltGr
    pub const KEY_UP: u16 = 103;
    pub const KEY_LEFT: u16 = 105;
    pub const KEY_RIGHT: u16 = 106;
    pub const KEY_DOWN: u16 = 108;
    pub const KEY_LEFTMETA: u16 = 125; // Super / Windows key
    pub const KEY_RIGHTMETA: u16 = 126;
}

/// Mapeia o scancode físico universal do protocolo Nexa para o código correspondente do kernel Linux evdev
pub fn nexa_scancode_to_linux_evdev(nexa_scancode: u16) -> u16 {
    let is_extended = (nexa_scancode & 0xE000) != 0;
    let base_code = (nexa_scancode & 0x00FF) as u8;

    if is_extended {
        match base_code {
            0x1D => evdev_keys::KEY_RIGHTCTRL,
            0x38 => evdev_keys::KEY_RIGHTALT, // AltGr
            0x48 => evdev_keys::KEY_UP,
            0x4B => evdev_keys::KEY_LEFT,
            0x4D => evdev_keys::KEY_RIGHT,
            0x50 => evdev_keys::KEY_DOWN,
            0x5B => evdev_keys::KEY_LEFTMETA, // Windows / Super
            0x5C => evdev_keys::KEY_RIGHTMETA,
            _ => base_code as u16,
        }
    } else {
        match base_code {
            0x01 => evdev_keys::KEY_ESC,
            0x02 => evdev_keys::KEY_1,
            0x03 => evdev_keys::KEY_2,
            0x04 => evdev_keys::KEY_3,
            0x05 => evdev_keys::KEY_4,
            0x06 => evdev_keys::KEY_5,
            0x07 => evdev_keys::KEY_6,
            0x08 => evdev_keys::KEY_7,
            0x09 => evdev_keys::KEY_8,
            0x0A => evdev_keys::KEY_9,
            0x0B => evdev_keys::KEY_0,
            0x0C => evdev_keys::KEY_MINUS,
            0x0D => evdev_keys::KEY_EQUAL,
            0x0E => evdev_keys::KEY_BACKSPACE,
            0x0F => evdev_keys::KEY_TAB,
            0x10 => evdev_keys::KEY_Q,
            0x11 => evdev_keys::KEY_W,
            0x12 => evdev_keys::KEY_E,
            0x13 => evdev_keys::KEY_R,
            0x14 => evdev_keys::KEY_T,
            0x15 => evdev_keys::KEY_Y,
            0x16 => evdev_keys::KEY_U,
            0x17 => evdev_keys::KEY_I,
            0x18 => evdev_keys::KEY_O,
            0x19 => evdev_keys::KEY_P,
            0x1C => evdev_keys::KEY_ENTER,
            0x1D => evdev_keys::KEY_LEFTCTRL,
            0x1E => evdev_keys::KEY_A,
            0x1F => evdev_keys::KEY_S,
            0x20 => evdev_keys::KEY_D,
            0x21 => evdev_keys::KEY_F,
            0x22 => evdev_keys::KEY_G,
            0x23 => evdev_keys::KEY_H,
            0x24 => evdev_keys::KEY_J,
            0x25 => evdev_keys::KEY_K,
            0x26 => evdev_keys::KEY_L,
            0x2A => evdev_keys::KEY_LEFTSHIFT,
            0x2C => evdev_keys::KEY_Z,
            0x2D => evdev_keys::KEY_X,
            0x2E => evdev_keys::KEY_C,
            0x2F => evdev_keys::KEY_V,
            0x30 => evdev_keys::KEY_B,
            0x31 => evdev_keys::KEY_N,
            0x32 => evdev_keys::KEY_M,
            0x36 => evdev_keys::KEY_RIGHTSHIFT,
            0x38 => evdev_keys::KEY_LEFTALT,
            0x39 => evdev_keys::KEY_SPACE,
            0x3A => evdev_keys::KEY_CAPSLOCK,
            0x3B => evdev_keys::KEY_F1,
            0x3C => evdev_keys::KEY_F2,
            0x3D => evdev_keys::KEY_F3,
            0x3E => evdev_keys::KEY_F4,
            0x3F => evdev_keys::KEY_F5,
            0x40 => evdev_keys::KEY_F6,
            0x41 => evdev_keys::KEY_F7,
            0x42 => evdev_keys::KEY_F8,
            0x43 => evdev_keys::KEY_F9,
            0x44 => evdev_keys::KEY_F10,
            0x57 => evdev_keys::KEY_F11,
            0x58 => evdev_keys::KEY_F12,
            other => other as u16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scancode_translation_keys() {
        assert_eq!(nexa_scancode_to_linux_evdev(0x1E), evdev_keys::KEY_A);
        assert_eq!(nexa_scancode_to_linux_evdev(0x30), evdev_keys::KEY_B);
        assert_eq!(nexa_scancode_to_linux_evdev(0x1C), evdev_keys::KEY_ENTER);
        assert_eq!(nexa_scancode_to_linux_evdev(0x39), evdev_keys::KEY_SPACE);
        assert_eq!(nexa_scancode_to_linux_evdev(0x2A), evdev_keys::KEY_LEFTSHIFT);
    }

    #[test]
    fn test_extended_keys_translation() {
        // Extended right ctrl
        assert_eq!(nexa_scancode_to_linux_evdev(0xE01D), evdev_keys::KEY_RIGHTCTRL);
        // Extended AltGr
        assert_eq!(nexa_scancode_to_linux_evdev(0xE038), evdev_keys::KEY_RIGHTALT);
        // Extended Super/Windows key
        assert_eq!(nexa_scancode_to_linux_evdev(0xE05B), evdev_keys::KEY_LEFTMETA);
        // Extended Arrow Down
        assert_eq!(nexa_scancode_to_linux_evdev(0xE050), evdev_keys::KEY_DOWN);
    }
}
