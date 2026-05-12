use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use parking_lot::Mutex;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

use crate::model::dispatch_mode::DispatchMode;
use crate::model::macro_sequence::MacroSequence;
use crate::model::macro_step::*;

pub struct RecorderOptions {
    pub target_name: String,
    pub dispatch_mode: DispatchMode,
    pub stop_virtual_keys: Vec<u16>,
}

#[derive(Clone)]
struct RecordedEvent {
    event_type: RecordedEventType,
    data: u32,
    point: POINT,
    timestamp: Instant,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum RecordedEventType {
    Delay,
    KeyDown,
    KeyUp,
    MouseMove,
    MouseLeftDown,
    MouseLeftUp,
    MouseRightDown,
    MouseRightUp,
    MouseMiddleDown,
    MouseMiddleUp,
}

pub struct RecorderService {
    recording: AtomicBool,
    inner: Mutex<RecorderInner>,
}

struct RecorderInner {
    keyboard_hook: HHOOK,
    mouse_hook: HHOOK,
    options: RecorderOptions,
    events: Vec<RecordedEvent>,
    last_timestamp: Instant,
}

const NULL_HOOK: HHOOK = HHOOK(std::ptr::null_mut());

static mut RECORDER_INSTANCE: Option<*const RecorderService> = None;

impl RecorderService {
    pub fn new() -> Self {
        Self {
            recording: AtomicBool::new(false),
            inner: Mutex::new(RecorderInner {
                keyboard_hook: NULL_HOOK,
                mouse_hook: NULL_HOOK,
                options: RecorderOptions {
                    target_name: String::new(),
                    dispatch_mode: DispatchMode::SendInput,
                    stop_virtual_keys: Vec::new(),
                },
                events: Vec::new(),
                last_timestamp: Instant::now(),
            }),
        }
    }

    pub fn start(&self, options: RecorderOptions) -> bool {
        if self.recording.load(Ordering::Relaxed) {
            return false;
        }

        let mut inner = self.inner.lock();
        inner.options = options;
        inner.events.clear();
        inner.last_timestamp = Instant::now();

        unsafe {
            RECORDER_INSTANCE = Some(self as *const _);

            let hinstance = GetModuleHandleW(None).ok().map(|h| h.into());
            
            inner.keyboard_hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(Self::keyboard_hook_proc),
                hinstance,
                0,
            ).unwrap_or(NULL_HOOK);

            inner.mouse_hook = SetWindowsHookExW(
                WH_MOUSE_LL,
                Some(Self::mouse_hook_proc),
                hinstance,
                0,
            ).unwrap_or(NULL_HOOK);

            if inner.keyboard_hook.0.is_null() || inner.mouse_hook.0.is_null() {
                self.cleanup_hooks(&mut inner);
                RECORDER_INSTANCE = None;
                return false;
            }
        }

        self.recording.store(true, Ordering::Relaxed);
        log::info!("Recording started");
        true
    }

    pub fn stop(&self) -> Option<MacroSequence> {
        if !self.recording.load(Ordering::Relaxed) {
            return None;
        }

        self.recording.store(false, Ordering::Relaxed);

        let mut inner = self.inner.lock();
        unsafe {
            self.cleanup_hooks(&mut inner);
            RECORDER_INSTANCE = None;
        }

        // Trim trailing stop key chord
        let stop_keys = inner.options.stop_virtual_keys.clone();
        self.trim_stop_keys(&mut inner.events, &stop_keys);

        // Compress mouse moves
        self.compress_mouse_moves(&mut inner.events);

        // Convert to macro sequence
        let sequence = self.events_to_sequence(&inner.events, &inner.options);

        log::info!("Recording stopped, {} events", inner.events.len());
        Some(sequence)
    }

    pub fn is_recording(&self) -> bool {
        self.recording.load(Ordering::Relaxed)
    }

    fn cleanup_hooks(&self, inner: &mut RecorderInner) {
        unsafe {
            if !inner.keyboard_hook.0.is_null() {
                let _ = UnhookWindowsHookEx(inner.keyboard_hook);
                inner.keyboard_hook = NULL_HOOK;
            }
            if !inner.mouse_hook.0.is_null() {
                let _ = UnhookWindowsHookEx(inner.mouse_hook);
                inner.mouse_hook = NULL_HOOK;
            }
        }
    }

    unsafe extern "system" fn keyboard_hook_proc(
        ncode: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if ncode >= 0 {
            if let Some(instance_ptr) = RECORDER_INSTANCE {
                let instance = &*instance_ptr;
                if instance.recording.load(Ordering::Relaxed) {
                    let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
                    
                    // Filter injected events
                    if (kb.flags & LLKHF_INJECTED).0 != 0 {
                        return CallNextHookEx(None, ncode, wparam, lparam);
                    }

                    let event_type = match wparam.0 as u32 {
                        WM_KEYDOWN | WM_SYSKEYDOWN => RecordedEventType::KeyDown,
                        WM_KEYUP | WM_SYSKEYUP => RecordedEventType::KeyUp,
                        _ => {
                            return CallNextHookEx(None, ncode, wparam, lparam);
                        }
                    };

                    let mut inner = instance.inner.lock();
                    let now = Instant::now();
                    
                    inner.events.push(RecordedEvent {
                        event_type,
                        data: kb.vkCode,
                        point: POINT { x: 0, y: 0 },
                        timestamp: now,
                    });
                    inner.last_timestamp = now;

                    // Check for stop keys
                    if instance.check_stop_keys(&inner.events, &inner.options.stop_virtual_keys) {
                        // Will be stopped by external logic
                    }
                }
            }
        }
        unsafe { CallNextHookEx(None, ncode, wparam, lparam) }
    }

    unsafe extern "system" fn mouse_hook_proc(
        ncode: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if ncode >= 0 {
            if let Some(instance_ptr) = RECORDER_INSTANCE {
                let instance = &*instance_ptr;
                if instance.recording.load(Ordering::Relaxed) {
                    let ms = &*(lparam.0 as *const MSLLHOOKSTRUCT);
                    
                    // Filter injected events
                    if (ms.flags & LLMHF_INJECTED) != 0 {
                        return CallNextHookEx(None, ncode, wparam, lparam);
                    }

                    let event_type = match wparam.0 as u32 {
                        WM_MOUSEMOVE => RecordedEventType::MouseMove,
                        WM_LBUTTONDOWN => RecordedEventType::MouseLeftDown,
                        WM_LBUTTONUP => RecordedEventType::MouseLeftUp,
                        WM_RBUTTONDOWN => RecordedEventType::MouseRightDown,
                        WM_RBUTTONUP => RecordedEventType::MouseRightUp,
                        WM_MBUTTONDOWN => RecordedEventType::MouseMiddleDown,
                        WM_MBUTTONUP => RecordedEventType::MouseMiddleUp,
                        _ => {
                            return CallNextHookEx(None, ncode, wparam, lparam);
                        }
                    };

                    let mut inner = instance.inner.lock();
                    let now = Instant::now();
                    
                    inner.events.push(RecordedEvent {
                        event_type,
                        data: 0,
                        point: ms.pt,
                        timestamp: now,
                    });
                    inner.last_timestamp = now;
                }
            }
        }
        unsafe { CallNextHookEx(None, ncode, wparam, lparam) }
    }

    fn check_stop_keys(&self, events: &[RecordedEvent], stop_keys: &[u16]) -> bool {
        if stop_keys.is_empty() {
            return false;
        }

        // Check if all stop keys are currently pressed
        let mut pressed_keys = std::collections::HashSet::new();
        for event in events.iter().rev() {
            match event.event_type {
                RecordedEventType::KeyDown => {
                    pressed_keys.insert(event.data as u16);
                }
                RecordedEventType::KeyUp => {
                    pressed_keys.remove(&(event.data as u16));
                }
                _ => {}
            }
        }

        stop_keys.iter().all(|k| pressed_keys.contains(k))
    }

    fn trim_stop_keys(&self, events: &mut Vec<RecordedEvent>, stop_keys: &[u16]) {
        if stop_keys.is_empty() {
            return;
        }

        // Remove trailing stop key events
        while let Some(last) = events.last() {
            match last.event_type {
                RecordedEventType::KeyDown | RecordedEventType::KeyUp => {
                    if stop_keys.contains(&(last.data as u16)) {
                        events.pop();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn compress_mouse_moves(&self, events: &mut Vec<RecordedEvent>) {
        let mut i = 0;
        while i < events.len() {
            if matches!(events[i].event_type, RecordedEventType::MouseMove) {
                let mut j = i + 1;
                while j < events.len() && matches!(events[j].event_type, RecordedEventType::MouseMove) {
                    j += 1;
                }
                if j > i + 1 {
                    // Keep only the last mouse move
                    events.drain(i..j - 1);
                }
            }
            i += 1;
        }
    }

    fn events_to_sequence(&self, events: &[RecordedEvent], options: &RecorderOptions) -> MacroSequence {
        let mut steps = Vec::new();
        let mut last_time = None;

        for event in events {
            // Add delay
            if let Some(last) = last_time {
                let delay_ms = event.timestamp.duration_since(last).as_millis() as u32;
                if delay_ms > 0 {
                    steps.push(MacroStep::Delay { milliseconds: delay_ms });
                }
            }
            last_time = Some(event.timestamp);

            // Add step
            match event.event_type {
                RecordedEventType::KeyDown => {
                    steps.push(MacroStep::Key {
                        virtual_key: event.data as u16,
                        action: KeyAction::Down,
                    });
                }
                RecordedEventType::KeyUp => {
                    steps.push(MacroStep::Key {
                        virtual_key: event.data as u16,
                        action: KeyAction::Up,
                    });
                }
                RecordedEventType::MouseMove => {
                    steps.push(MacroStep::MouseMove {
                        x: event.point.x,
                        y: event.point.y,
                        coordinate_mode: CoordinateMode::Screen,
                    });
                }
                RecordedEventType::MouseLeftDown => {
                    steps.push(MacroStep::MouseClick {
                        button: MouseButton::Left,
                        action: MouseButtonAction::Down,
                    });
                }
                RecordedEventType::MouseLeftUp => {
                    steps.push(MacroStep::MouseClick {
                        button: MouseButton::Left,
                        action: MouseButtonAction::Up,
                    });
                }
                RecordedEventType::MouseRightDown => {
                    steps.push(MacroStep::MouseClick {
                        button: MouseButton::Right,
                        action: MouseButtonAction::Down,
                    });
                }
                RecordedEventType::MouseRightUp => {
                    steps.push(MacroStep::MouseClick {
                        button: MouseButton::Right,
                        action: MouseButtonAction::Up,
                    });
                }
                RecordedEventType::MouseMiddleDown => {
                    steps.push(MacroStep::MouseClick {
                        button: MouseButton::Middle,
                        action: MouseButtonAction::Down,
                    });
                }
                RecordedEventType::MouseMiddleUp => {
                    steps.push(MacroStep::MouseClick {
                        button: MouseButton::Middle,
                        action: MouseButtonAction::Up,
                    });
                }
                RecordedEventType::Delay => {}
            }
        }

        MacroSequence {
            name: "recorded".to_string(),
            target_name: options.target_name.clone(),
            dispatch_mode: options.dispatch_mode,
            source_file: String::new(),
            steps,
            on_press_steps: Vec::new(),
            on_hold_steps: Vec::new(),
            on_release_steps: Vec::new(),
            has_phases: false,
        }
    }
}

impl Drop for RecorderService {
    fn drop(&mut self) {
        if self.recording.load(Ordering::Relaxed) {
            self.stop();
        }
    }
}
