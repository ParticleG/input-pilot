// VK to USB HID usage code mapping table.
// Extracted from IbInputSimulator (MIT, Copyright 2021 Chaoses-Ib).

pub fn keyboard_vk_to_usage(vk: u8) -> u8 {
    match vk {
        0x00 => 0x00,
        0x03 => 0x9B, // VK_CANCEL
        0x08 => 0x2A, // VK_BACK
        0x09 => 0x2B, // VK_TAB
        0x0D => 0x28, // VK_RETURN
        0x13 => 0x48, // VK_PAUSE
        0x14 => 0x39, // VK_CAPITAL
        0x1B => 0x29, // VK_ESCAPE
        0x20 => 0x2C, // VK_SPACE
        0x21 => 0x4B, // VK_PRIOR
        0x22 => 0x4E, // VK_NEXT
        0x23 => 0x4D, // VK_END
        0x24 => 0x4A, // VK_HOME
        0x25 => 0x50, // VK_LEFT
        0x26 => 0x52, // VK_UP
        0x27 => 0x4F, // VK_RIGHT
        0x28 => 0x51, // VK_DOWN
        0x2C => 0x46, // VK_SNAPSHOT
        0x2D => 0x49, // VK_INSERT
        0x2E => 0x4C, // VK_DELETE
        0x5B => 0xE3, // VK_LWIN
        0x5C => 0xE7, // VK_RWIN
        0x5D => 0x65, // VK_APPS
        0x6A => 0x55, // VK_MULTIPLY
        0x6B => 0x57, // VK_ADD
        0x6D => 0x56, // VK_SUBTRACT
        0x6E => 0x63, // VK_DECIMAL
        0x6F => 0x54, // VK_DIVIDE
        0x90 => 0x53, // VK_NUMLOCK
        0x91 => 0x47, // VK_SCROLL
        0xA0 => 0xE1, // VK_LSHIFT
        0xA1 => 0xE5, // VK_RSHIFT
        0xA2 => 0xE0, // VK_LCONTROL
        0xA3 => 0xE4, // VK_RCONTROL
        0xA4 => 0xE2, // VK_LMENU
        0xA5 => 0xE6, // VK_RMENU
        0xBA => 0x33, // VK_OEM_1  ;:
        0xBB => 0x2E, // VK_OEM_PLUS
        0xBC => 0x36, // VK_OEM_COMMA
        0xBD => 0x2D, // VK_OEM_MINUS
        0xBE => 0x37, // VK_OEM_PERIOD
        0xBF => 0x38, // VK_OEM_2  /?
        0xC0 => 0x35, // VK_OEM_3  `~
        0xDB => 0x2F, // VK_OEM_4  [{
        0xDC => 0x31, // VK_OEM_5  \|
        0xDD => 0x30, // VK_OEM_6  ]}
        0xDE => 0x34, // VK_OEM_7  '"
        0xE2 => 0x64, // VK_OEM_102  <>
        _ => {
            // A-Z
            if vk >= b'A' && vk <= b'Z' {
                return 0x04 + vk - b'A';
            }
            // 0-9
            if vk >= b'0' && vk <= b'9' {
                return if vk == b'0' { 0x27 } else { 0x1E + vk - b'1' };
            }
            // VK_NUMPAD0..9 (0x60..0x69)
            if (0x60..=0x69).contains(&vk) {
                return if vk == 0x60 { 0x62 } else { 0x59 + vk - 0x61 };
            }
            // VK_F1..VK_F24 (0x70..0x87)
            if (0x70..=0x87).contains(&vk) {
                return if vk <= 0x7B {
                    0x3A + vk - 0x70 // F1..F12
                } else {
                    0x68 + vk - 0x7C // F13..F24
                };
            }
            0x00
        }
    }
}
