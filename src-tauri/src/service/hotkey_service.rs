use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use parking_lot::Mutex;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

use crate::model::hotkey_binding::*;

pub const WM_HOTKEY_DOWN: u32 = 0x8000 + 1; // WM_APP + 1
pub const WM_HOTKEY_UP: u32 = 0x8000 + 2;   // WM_APP + 2

const NULL_HOOK: HHOOK = HHOOK(std::ptr::null_mut());

struct HookBinding {
    id: i32,
    modifiers: u32,
    virtual_key: u32,
    trigger_mode: TriggerMode,
}

pub struct HotkeyService {
    registered_hotkeys: Vec<HotkeyBinding>,
    hook_bindings: Vec<HookBinding>,
    hook: HHOOK,
    target_hwnd: HWND,
    pressed_keys: Mutex<HashSet<u32>>,
    active_hotkeys: Mutex<HashSet<i32>>,
}

static mut HOTKEY_INSTANCE: Option<*const HotkeyService> = None;

impl HotkeyService {
    pub fn new(target_hwnd: HWND) -> Self {
        Self {
            registered_hotkeys: Vec::new(),
            hook_bindings: Vec::new(),
            hook: NULL_HOOK,
            target_hwnd,
            pressed_keys: Mutex::new(HashSet::new()),
            active_hotkeys: Mutex::new(HashSet::new()),
        }
    }

    pub fn register_hotkeys(&mut self, hotkeys: Vec<HotkeyBinding>) -> bool {
        self.unregister_all();
        self.registered_hotkeys = hotkeys;

        let mut once_hotkeys = Vec::new();
        let mut hook_hotkeys = Vec::new();

        for hotkey in &self.registered_hotkeys {
            match hotkey.trigger_mode {
                TriggerMode::Once => once_hotkeys.push(hotkey),
                TriggerMode::Toggle | TriggerMode::Hold | TriggerMode::Phased => {
                    hook_hotkeys.push(hotkey);
                }
            }
        }

        // Register Once mode hotkeys with RegisterHotKey
        for hotkey in once_hotkeys {
            unsafe {
                let result = RegisterHotKey(
                    Some(self.target_hwnd),
                    hotkey.id,
                    HOT_KEY_MODIFIERS(hotkey.modifiers),
                    hotkey.virtual_key,
                );
                if result.is_err() {
                    log::warn!("Failed to register hotkey id={}", hotkey.id);
                }
            }
        }

        // Set up hook for other modes
        if !hook_hotkeys.is_empty() {
            self.hook_bindings = hook_hotkeys.iter().map(|h| HookBinding {
                id: h.id,
                modifiers: h.modifiers,
                virtual_key: h.virtual_key,
                trigger_mode: h.trigger_mode,
            }).collect();

            unsafe {
                HOTKEY_INSTANCE = Some(self as *const _);
                let hinstance = GetModuleHandleW(None).ok().map(|h| h.into());
                
                self.hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(Self::keyboard_hook_proc),
                    hinstance,
                    0,
                ).unwrap_or(NULL_HOOK);

                if self.hook.0.is_null() {
                    log::error!("Failed to install keyboard hook");
                    HOTKEY_INSTANCE = None;
                    return false;
                }
            }
        }

        log::info!("Registered {} hotkeys", self.registered_hotkeys.len());
        true
    }

    pub fn unregister_all(&mut self) {
        // Unregister RegisterHotKey hotkeys
        for hotkey in &self.registered_hotkeys {
            if hotkey.trigger_mode == TriggerMode::Once {
                unsafe {
                    let _ = UnregisterHotKey(Some(self.target_hwnd), hotkey.id);
                }
            }
        }

        // Remove hook
        if !self.hook.0.is_null() {
            unsafe {
                let _ = UnhookWindowsHookEx(self.hook);
                self.hook = NULL_HOOK;
                HOTKEY_INSTANCE = None;
            }
        }

        self.registered_hotkeys.clear();
        self.hook_bindings.clear();
        self.pressed_keys.lock().clear();
        self.active_hotkeys.lock().clear();
    }

    unsafe extern "system" fn keyboard_hook_proc(
        ncode: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if ncode >= 0 {
            if let Some(instance_ptr) = HOTKEY_INSTANCE {
                let instance = &*instance_ptr;
                let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
                
                // Filter injected events
                if (kb.flags & LLKHF_INJECTED).0 != 0 {
                    return CallNextHookEx(None, ncode, wparam, lparam);
                }

                let is_down = wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
                let is_up = wparam.0 as u32 == WM_KEYUP || wparam.0 as u32 == WM_SYSKEYUP;

                if is_down {
                    instance.pressed_keys.lock().insert(kb.vkCode);
                    instance.check_hotkey_match(true);
                } else if is_up {
                    instance.pressed_keys.lock().remove(&kb.vkCode);
                    instance.check_hotkey_match(false);
                }
            }
        }
        unsafe { CallNextHookEx(None, ncode, wparam, lparam) }
    }

    fn check_hotkey_match(&self, is_key_down: bool) {
        let pressed = self.pressed_keys.lock();
        let mut active = self.active_hotkeys.lock();

        for binding in &self.hook_bindings {
            let modifiers_match = self.check_modifiers(&pressed, binding.modifiers);
            let key_match = pressed.contains(&binding.virtual_key);
            let is_active = active.contains(&binding.id);

            if modifiers_match && key_match {
                if is_key_down && !is_active {
                    // Hotkey pressed
                    active.insert(binding.id);
                    unsafe {
                        let _ = PostMessageW(
                            Some(self.target_hwnd),
                            WM_HOTKEY_DOWN,
                            WPARAM(binding.id as usize),
                            LPARAM(0),
                        );
                    }
                }
            } else if is_active {
                // Hotkey released
                active.remove(&binding.id);
                unsafe {
                    let _ = PostMessageW(
                        Some(self.target_hwnd),
                        WM_HOTKEY_UP,
                        WPARAM(binding.id as usize),
                        LPARAM(0),
                    );
                }
            }
        }
    }

    fn check_modifiers(&self, pressed: &HashSet<u32>, modifiers: u32) -> bool {
        let needs_alt = (modifiers & MOD_ALT.0) != 0;
        let needs_ctrl = (modifiers & MOD_CONTROL.0) != 0;
        let needs_shift = (modifiers & MOD_SHIFT.0) != 0;
        let needs_win = (modifiers & MOD_WIN.0) != 0;

        let has_alt = pressed.contains(&(VK_MENU.0 as u32)) 
            || pressed.contains(&(VK_LMENU.0 as u32)) 
            || pressed.contains(&(VK_RMENU.0 as u32));
        let has_ctrl = pressed.contains(&(VK_CONTROL.0 as u32)) 
            || pressed.contains(&(VK_LCONTROL.0 as u32)) 
            || pressed.contains(&(VK_RCONTROL.0 as u32));
        let has_shift = pressed.contains(&(VK_SHIFT.0 as u32)) 
            || pressed.contains(&(VK_LSHIFT.0 as u32)) 
            || pressed.contains(&(VK_RSHIFT.0 as u32));
        let has_win = pressed.contains(&(VK_LWIN.0 as u32)) 
            || pressed.contains(&(VK_RWIN.0 as u32));

        (needs_alt == has_alt) && (needs_ctrl == has_ctrl) 
            && (needs_shift == has_shift) && (needs_win == has_win)
    }

    pub fn find_hotkey_by_id(&self, id: i32) -> Option<&HotkeyBinding> {
        self.registered_hotkeys.iter().find(|h| h.id == id)
    }
}

impl Drop for HotkeyService {
    fn drop(&mut self) {
        self.unregister_all();
    }
}
