// Logitech Gaming Software driver interface.
// Extracted and rewritten from IbInputSimulator (MIT, Copyright 2021 Chaoses-Ib).
// Uses ntdll NtOpenDirectoryObject/NtQueryDirectoryObject to find LGS device,
// then DeviceIoControl to send HID keyboard/mouse reports.

use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::IO::DeviceIoControl;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::usb_hid_table::keyboard_vk_to_usage;

// IOCTL codes for Logitech driver
const IOCTL_KEYBOARD: u32 = 0x2A200C;
const IOCTL_MOUSE: u32 = 0x2A2010;

// NT API types and functions (loaded from ntdll)
#[repr(C)]
struct UnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}

#[repr(C)]
struct ObjectAttributes {
    length: u32,
    root_directory: HANDLE,
    object_name: *mut UnicodeString,
    attributes: u32,
    security_descriptor: *mut std::ffi::c_void,
    security_quality_of_service: *mut std::ffi::c_void,
}

#[repr(C)]
struct ObjectDirectoryInformation {
    name: UnicodeString,
    type_name: UnicodeString,
}

extern "system" {
    fn NtOpenDirectoryObject(
        directory_handle: *mut HANDLE,
        desired_access: u32,
        object_attributes: *mut ObjectAttributes,
    ) -> i32;

    fn NtQueryDirectoryObject(
        directory_handle: HANDLE,
        buffer: *mut std::ffi::c_void,
        length: u32,
        return_single_entry: u8,
        restart_scan: u8,
        context: *mut u32,
        return_length: *mut u32,
    ) -> i32;

    fn RtlInitUnicodeString(
        destination_string: *mut UnicodeString,
        source_string: *const u16,
    );
}

const STATUS_SUCCESS: i32 = 0;
const STATUS_MORE_ENTRIES: i32 = 0x00000105;
const DIRECTORY_QUERY: u32 = 0x0001;

// HID report structures (packed)
#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
struct MouseHidReport {
    buttons: u8,
    x: i8,
    y: i8,
    wheel: u8,
    padding: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
struct KeyboardHidReport {
    modifiers: u8,
    reserved: u8,
    keys: [u8; 6],
}

// Mouse button bit flags
const MOUSE_LBUTTON: u8 = 0x01;
const MOUSE_RBUTTON: u8 = 0x02;
const MOUSE_MBUTTON: u8 = 0x04;

// Keyboard modifier bit flags (USB HID)
const MOD_LCTRL: u8 = 0x01;
const MOD_LSHIFT: u8 = 0x02;
const MOD_LALT: u8 = 0x04;
const MOD_LGUI: u8 = 0x08;
const MOD_RCTRL: u8 = 0x10;
const MOD_RSHIFT: u8 = 0x20;
const MOD_RALT: u8 = 0x40;
const MOD_RGUI: u8 = 0x80;

// ---------------------------------------------------------------------------
// Low-level driver: find device, open handle, send HID reports
// ---------------------------------------------------------------------------
struct LogitechDriverInner {
    device: HANDLE,
}

impl LogitechDriverInner {
    fn new() -> Self {
        Self {
            device: HANDLE(std::ptr::null_mut()),
        }
    }

    fn initialize(&mut self) -> Result<(), &'static str> {
        let device_name = Self::find_device();
        if device_name.is_empty() {
            return Err("DeviceNotFound");
        }

        let wide: Vec<u16> = device_name.encode_utf16().chain(std::iter::once(0)).collect();
        let pcwstr = windows::core::PCWSTR(wide.as_ptr());

        let handle: windows::core::Result<HANDLE> = unsafe {
            windows::Win32::Storage::FileSystem::CreateFileW(
                pcwstr,
                GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            )
        };

        match handle {
            Ok(h) => {
                if h.is_invalid() {
                    return Err("DeviceOpenFailed");
                }
                self.device = h;
                Ok(())
            }
            Err(_) => Err("DeviceOpenFailed"),
        }
    }

    fn destroy(&mut self) {
        if !self.device.is_invalid() && !self.device.0.is_null() {
            unsafe { let _ = CloseHandle(self.device); }
            self.device = HANDLE(std::ptr::null_mut());
        }
    }

    fn is_initialized(&self) -> bool {
        !self.device.is_invalid() && !self.device.0.is_null()
    }

    fn send_keyboard_report(&self, report: &KeyboardHidReport) -> bool {
        if !self.is_initialized() { return false; }
        let mut bytes_returned = 0u32;
        let ok = unsafe {
            DeviceIoControl(
                self.device,
                IOCTL_KEYBOARD,
                Some(report as *const KeyboardHidReport as *const std::ffi::c_void),
                std::mem::size_of::<KeyboardHidReport>() as u32,
                None,
                0,
                Some(&mut bytes_returned),
                None,
            )
        };
        ok.is_ok()
    }

    fn send_mouse_report(&self, report: &MouseHidReport) -> bool {
        if !self.is_initialized() { return false; }
        let mut bytes_returned = 0u32;
        let ok = unsafe {
            DeviceIoControl(
                self.device,
                IOCTL_MOUSE,
                Some(report as *const MouseHidReport as *const std::ffi::c_void),
                std::mem::size_of::<MouseHidReport>() as u32,
                None,
                0,
                Some(&mut bytes_returned),
                None,
            )
        };
        ok.is_ok()
    }

    fn find_device() -> String {
        // Enumerate \\GLOBAL?? directory for Logitech device symlinks
        let dir_name_wide: Vec<u16> = "\\GLOBAL??\0".encode_utf16().collect();
        let mut uni_str = UnicodeString {
            length: ((dir_name_wide.len() - 1) * 2) as u16,
            maximum_length: (dir_name_wide.len() * 2) as u16,
            buffer: dir_name_wide.as_ptr() as *mut u16,
        };

        let mut obj_attr = ObjectAttributes {
            length: std::mem::size_of::<ObjectAttributes>() as u32,
            root_directory: HANDLE(std::ptr::null_mut()),
            object_name: &mut uni_str,
            attributes: 0,
            security_descriptor: std::ptr::null_mut(),
            security_quality_of_service: std::ptr::null_mut(),
        };

        let mut dir_handle = HANDLE(std::ptr::null_mut());
        let status = unsafe {
            NtOpenDirectoryObject(&mut dir_handle, DIRECTORY_QUERY, &mut obj_attr)
        };
        if status != STATUS_SUCCESS {
            return String::new();
        }

        let mut buffer = vec![0u8; 4096];
        let mut context: u32 = 0;
        let mut result = String::new();

        let mut status = unsafe {
            NtQueryDirectoryObject(
                dir_handle,
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                buffer.len() as u32,
                0, // ReturnSingleEntry = FALSE
                1, // RestartScan = TRUE
                &mut context,
                std::ptr::null_mut(),
            )
        };

        while status == STATUS_SUCCESS || status == STATUS_MORE_ENTRIES {
            let info = buffer.as_ptr() as *const ObjectDirectoryInformation;
            let mut found = false;

            unsafe {
                let mut i = 0;
                loop {
                    let entry = &*info.add(i);
                    if entry.name.buffer.is_null() {
                        break;
                    }
                    let name_len = entry.name.length as usize / 2;
                    let name_slice = std::slice::from_raw_parts(entry.name.buffer, name_len);
                    let name = String::from_utf16_lossy(name_slice);

                    if Self::match_logitech_device(&name) {
                        result = format!("\\??\\{}", name);
                        found = true;
                        break;
                    }
                    i += 1;
                }
            }

            if found || status != STATUS_MORE_ENTRIES {
                break;
            }

            status = unsafe {
                NtQueryDirectoryObject(
                    dir_handle,
                    buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    buffer.len() as u32,
                    0,
                    0, // RestartScan = FALSE
                    &mut context,
                    std::ptr::null_mut(),
                )
            };
        }

        unsafe { let _ = CloseHandle(dir_handle); }
        result
    }

    fn match_logitech_device(name: &str) -> bool {
        if !name.starts_with("ROOT#SYSTEM#") && !name.starts_with("Root#SYSTEM#") {
            return false;
        }
        // LGS device GUIDs
        name.ends_with("#{1abc05c0-c378-41b9-9cef-df1aba82b015}")
            || name.ends_with("#{df31f106-d870-453d-8fa1-ec8ab43fa1d2}")
            || name.ends_with("#{dfbedcdb-2148-416d-9e4d-cecc2424128c}")
            || name.ends_with("#{5bada891-842b-4296-a496-68ae931aa16c}")
    }
}

impl Drop for LogitechDriverInner {
    fn drop(&mut self) {
        self.destroy();
    }
}

// ---------------------------------------------------------------------------
// High-level sender: tracks keyboard/mouse state, converts inputs to HID
// ---------------------------------------------------------------------------
pub struct LogitechSender {
    driver: LogitechDriverInner,
    keyboard_report: KeyboardHidReport,
}

impl LogitechSender {
    pub fn new() -> Self {
        Self {
            driver: LogitechDriverInner::new(),
            keyboard_report: KeyboardHidReport::default(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), &'static str> {
        self.driver.initialize()?;
        self.sync_key_states();
        log::info!("Logitech driver initialized (built-in)");
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.driver.is_initialized()
    }

    /// Send a mouse click (button down/up) through the Logitech driver.
    pub fn send_mouse_button(&self, button_bit: u8, down: bool) -> bool {
        let report = MouseHidReport {
            buttons: if down { button_bit } else { 0 },
            x: 0,
            y: 0,
            wheel: 0,
            padding: 0,
        };
        self.driver.send_mouse_report(&report)
    }

    /// Send a relative mouse move through the Logitech driver.
    pub fn send_mouse_move(&self, dx: i32, dy: i32) -> bool {

        // Send in ±127 chunks (i8 limit)
        let mut remaining_x = dx;
        let mut remaining_y = dy;

        while remaining_x.abs() > 127 || remaining_y.abs() > 127 {
            let chunk_x = remaining_x.clamp(-127, 127) as i8;
            let chunk_y = remaining_y.clamp(-127, 127) as i8;
            remaining_x -= chunk_x as i32;
            remaining_y -= chunk_y as i32;

            let report = MouseHidReport {
                buttons: 0,
                x: chunk_x,
                y: chunk_y,
                wheel: 0,
                padding: 0,
            };
            if !self.driver.send_mouse_report(&report) {
                return false;
            }
        }

        let report = MouseHidReport {
            buttons: 0,
            x: remaining_x as i8,
            y: remaining_y as i8,
            wheel: 0,
            padding: 0,
        };
        self.driver.send_mouse_report(&report)
    }

    /// Send a keyboard key through the Logitech driver.
    pub fn send_keyboard_key(&mut self, vk: u16, down: bool) -> bool {

        let vk_u8 = vk as u8;

        if Self::is_modifier(vk) {
            self.set_modifier_state(vk, down);
        } else {
            let usage = keyboard_vk_to_usage(vk_u8);
            if down {
                for key in &mut self.keyboard_report.keys {
                    if *key == 0 {
                        *key = usage;
                        break;
                    }
                }
            } else {
                for key in &mut self.keyboard_report.keys {
                    if *key == usage {
                        *key = 0;
                        break;
                    }
                }
            }
        }

        self.driver.send_keyboard_report(&self.keyboard_report)
    }

    /// Sync modifier key states with actual keyboard state.
    pub fn sync_key_states(&mut self) {
        unsafe {
            let mut mods = 0u8;
            if (GetAsyncKeyState(0xA2) & 0x8000u16 as i16) != 0 { mods |= MOD_LCTRL; }   // VK_LCONTROL
            if (GetAsyncKeyState(0xA3) & 0x8000u16 as i16) != 0 { mods |= MOD_RCTRL; }   // VK_RCONTROL
            if (GetAsyncKeyState(0xA0) & 0x8000u16 as i16) != 0 { mods |= MOD_LSHIFT; }  // VK_LSHIFT
            if (GetAsyncKeyState(0xA1) & 0x8000u16 as i16) != 0 { mods |= MOD_RSHIFT; }  // VK_RSHIFT
            if (GetAsyncKeyState(0xA4) & 0x8000u16 as i16) != 0 { mods |= MOD_LALT; }    // VK_LMENU
            if (GetAsyncKeyState(0xA5) & 0x8000u16 as i16) != 0 { mods |= MOD_RALT; }    // VK_RMENU
            if (GetAsyncKeyState(0x5B) & 0x8000u16 as i16) != 0 { mods |= MOD_LGUI; }    // VK_LWIN
            if (GetAsyncKeyState(0x5C) & 0x8000u16 as i16) != 0 { mods |= MOD_RGUI; }    // VK_RWIN
            self.keyboard_report.modifiers = mods;
        }
    }

    fn is_modifier(vk: u16) -> bool {
        matches!(vk, 0xA2 | 0xA3 | 0xA0 | 0xA1 | 0xA4 | 0xA5 | 0x5B | 0x5C)
    }

    fn set_modifier_state(&mut self, vk: u16, down: bool) {
        let bit = match vk {
            0xA2 => MOD_LCTRL,
            0xA3 => MOD_RCTRL,
            0xA0 => MOD_LSHIFT,
            0xA1 => MOD_RSHIFT,
            0xA4 => MOD_LALT,
            0xA5 => MOD_RALT,
            0x5B => MOD_LGUI,
            0x5C => MOD_RGUI,
            _ => return,
        };
        if down {
            self.keyboard_report.modifiers |= bit;
        } else {
            self.keyboard_report.modifiers &= !bit;
        }
    }
}

// Mouse button constants for external use
pub const LG_MOUSE_LEFT: u8 = MOUSE_LBUTTON;
pub const LG_MOUSE_RIGHT: u8 = MOUSE_RBUTTON;
pub const LG_MOUSE_MIDDLE: u8 = MOUSE_MBUTTON;

// Safety: LogitechSender only holds HANDLE (which is Send) and Mutex fields
unsafe impl Send for LogitechSender {}
