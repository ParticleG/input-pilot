use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::thread;
use parking_lot::Mutex;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::model::app_config::AppConfig;
use crate::model::hotkey_binding::HotkeyActionType;
use crate::model::macro_sequence::MacroSequence;
use super::hotkey_service::{HotkeyService, WM_HOTKEY_DOWN, WM_HOTKEY_UP};
use super::macro_runner::MacroRunner;
use super::macro_repository::MacroRepository;
use super::recorder_service::{RecorderOptions, RecorderService};
use super::macro_serialization;

/// The hotkey daemon runs a Win32 message loop on a dedicated thread,
/// dispatching hotkey events to the MacroRunner.
pub struct HotkeyDaemon {
    inner: Arc<Mutex<DaemonInner>>,
    running: Arc<AtomicBool>,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
    /// The HWND of the daemon's message-only window, stored as isize for Send safety.
    thread_hwnd: Arc<AtomicIsize>,
}

// Safety: HotkeyDaemon only stores the HWND as an atomic isize.
// The actual HWND is only used on the daemon thread that created it.
unsafe impl Send for HotkeyDaemon {}
unsafe impl Sync for HotkeyDaemon {}

struct DaemonInner {
    config: Option<AppConfig>,
    recordings_directory: String,
}

// Custom window messages to signal the daemon thread
const WM_DAEMON_RELOAD: u32 = 0x8000 + 10; // WM_APP + 10
const WM_DAEMON_STOP: u32 = 0x8000 + 11;   // WM_APP + 11

impl HotkeyDaemon {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DaemonInner {
                config: None,
                recordings_directory: "recordings".to_string(),
            })),
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: Mutex::new(None),
            thread_hwnd: Arc::new(AtomicIsize::new(0)),
        }
    }

    /// Update the config. If the daemon is running, it will reload hotkeys.
    pub fn update_config(&self, config: AppConfig) {
        let recordings_dir = config.recordings_directory.clone();
        {
            let mut inner = self.inner.lock();
            inner.config = Some(config);
            inner.recordings_directory = recordings_dir;
        }

        // Signal the daemon thread to reload
        if self.running.load(Ordering::Relaxed) {
            let hwnd_val = self.thread_hwnd.load(Ordering::Relaxed);
            if hwnd_val != 0 {
                let hwnd = HWND(hwnd_val as *mut _);
                unsafe {
                    let _ = PostMessageW(Some(hwnd), WM_DAEMON_RELOAD, WPARAM(0), LPARAM(0));
                }
            }
        }
    }

    /// Start the hotkey daemon on a dedicated thread.
    pub fn start(&self) -> bool {
        if self.running.load(Ordering::Relaxed) {
            return true; // Already running
        }

        let inner = Arc::clone(&self.inner);
        let running = Arc::clone(&self.running);
        let thread_hwnd = Arc::clone(&self.thread_hwnd);

        running.store(true, Ordering::Relaxed);

        let handle = thread::spawn(move || {
            Self::daemon_thread(inner, running, thread_hwnd);
        });

        *self.thread_handle.lock() = Some(handle);
        log::info!("Hotkey daemon started");
        true
    }

    /// Stop the hotkey daemon.
    pub fn stop(&self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        // Signal the thread to stop
        let hwnd_val = self.thread_hwnd.load(Ordering::Relaxed);
        if hwnd_val != 0 {
            let hwnd = HWND(hwnd_val as *mut _);
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_DAEMON_STOP, WPARAM(0), LPARAM(0));
            }
        }

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.lock().take() {
            let _ = handle.join();
        }

        log::info!("Hotkey daemon stopped");
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn daemon_thread(
        inner: Arc<Mutex<DaemonInner>>,
        running: Arc<AtomicBool>,
        thread_hwnd_slot: Arc<AtomicIsize>,
    ) {
        // Create a message-only window for receiving messages
        let hwnd = Self::create_message_window();
        if hwnd.0.is_null() {
            log::error!("Failed to create daemon message window");
            running.store(false, Ordering::Relaxed);
            return;
        }

        thread_hwnd_slot.store(hwnd.0 as isize, Ordering::Relaxed);

        // Build initial state
        let repo = {
            let inner_lock = inner.lock();
            if let Some(config) = &inner_lock.config {
                Arc::new(MacroRepository::new(config.clone()))
            } else {
                Arc::new(MacroRepository::new(AppConfig::new()))
            }
        };

        let mut hotkey_service = HotkeyService::new(hwnd);
        let macro_runner = MacroRunner::new(Arc::clone(&repo));
        let recorder = RecorderService::new();

        // Register hotkeys from config
        Self::register_hotkeys_from_repo(&mut hotkey_service, &repo);

        // Message loop — use MsgWaitForMultipleObjectsEx to ensure LL hook callbacks
        // are dispatched. GetMessageW alone can sometimes cause Windows to silently
        // remove LL hooks if the thread is not considered "responsive".
        unsafe {
            let mut msg = MSG::default();
            loop {
                // Wait for messages with a timeout to keep the thread responsive
                let _wait_result = MsgWaitForMultipleObjectsEx(
                    None,
                    100, // 100ms timeout — keeps thread "alive" for hook dispatch
                    QS_ALLINPUT,
                    MWMO_INPUTAVAILABLE,
                );

                // Process all pending messages
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    match msg.message {
                        WM_DAEMON_STOP => {
                            // Cleanup and exit
                            hotkey_service.unregister_all();
                            drop(macro_runner);
                            let _ = DestroyWindow(hwnd);
                            thread_hwnd_slot.store(0, Ordering::Relaxed);
                            running.store(false, Ordering::Relaxed);
                            return;
                        }
                        WM_DAEMON_RELOAD => {
                            // Reload config
                            let new_repo = {
                                let inner_lock = inner.lock();
                                if let Some(config) = &inner_lock.config {
                                    Arc::new(MacroRepository::new(config.clone()))
                                } else {
                                    continue;
                                }
                            };

                            // Re-register hotkeys
                            hotkey_service.unregister_all();
                            Self::register_hotkeys_from_repo(&mut hotkey_service, &new_repo);

                            // Hot-swap repo without losing toggle states
                            macro_runner.update_repo(new_repo);

                            log::info!("Hotkey daemon reloaded config");
                        }
                        WM_HOTKEY => {
                            // RegisterHotKey-based hotkey (Once mode)
                            let id = msg.wParam.0 as i32;
                            if let Some(hotkey) = hotkey_service.find_hotkey_by_id(id) {
                                let hotkey = hotkey.clone();
                                Self::handle_hotkey_action(
                                    &hotkey,
                                    &macro_runner,
                                    &recorder,
                                    &inner,
                                );
                            }
                        }
                        WM_HOTKEY_DOWN => {
                            let id = msg.wParam.0 as i32;
                            if let Some(hotkey) = hotkey_service.find_hotkey_by_id(id) {
                                let hotkey = hotkey.clone();
                                macro_runner.on_hotkey_down(&hotkey);
                            }
                        }
                        WM_HOTKEY_UP => {
                            let id = msg.wParam.0 as i32;
                            if let Some(hotkey) = hotkey_service.find_hotkey_by_id(id) {
                                let hotkey = hotkey.clone();
                                macro_runner.on_hotkey_up(&hotkey);
                            }
                        }
                        _ => {
                            let _ = TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    }
                }

                // Check if we should stop (in case WM_DAEMON_STOP was missed)
                if !running.load(Ordering::Relaxed) {
                    break;
                }
            }

            // Cleanup
            hotkey_service.unregister_all();
            drop(macro_runner);
            let _ = DestroyWindow(hwnd);
        }

        thread_hwnd_slot.store(0, Ordering::Relaxed);
        running.store(false, Ordering::Relaxed);
    }

    fn handle_hotkey_action(
        hotkey: &crate::model::hotkey_binding::HotkeyBinding,
        macro_runner: &MacroRunner,
        recorder: &RecorderService,
        inner: &Arc<Mutex<DaemonInner>>,
    ) {
        match hotkey.action {
            HotkeyActionType::PlayMacro | HotkeyActionType::PlayFile => {
                macro_runner.on_hotkey_down(hotkey);
            }
            HotkeyActionType::RecordToggle => {
                if recorder.is_recording() {
                    if let Some(sequence) = recorder.stop() {
                        // Save recording
                        let recordings_dir = inner.lock().recordings_directory.clone();
                        Self::save_recording(&recordings_dir, &sequence);
                    }
                } else {
                    let options = RecorderOptions {
                        target_name: hotkey.target_name.clone(),
                        dispatch_mode: hotkey.dispatch_mode,
                        stop_virtual_keys: Vec::new(), // Stop via next hotkey press
                    };
                    recorder.start(options);
                }
            }
        }
    }

    fn save_recording(recordings_dir: &str, sequence: &MacroSequence) {
        let path = format!("{}/last_recorded.macro", recordings_dir);

        // Create parent directory
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let lines = macro_serialization::serialize_macro(sequence);
        let content = lines.join("\n");

        match std::fs::write(&path, &content) {
            Ok(_) => log::info!("Recording saved to {}", path),
            Err(e) => log::error!("Failed to save recording to {}: {}", path, e),
        }
    }

    fn register_hotkeys_from_repo(service: &mut HotkeyService, repo: &MacroRepository) {
        let hotkeys = repo.config().hotkeys.clone();
        if !hotkeys.is_empty() {
            service.register_hotkeys(hotkeys);
        }
    }

    fn create_message_window() -> HWND {
        use windows::core::*;
        use windows::Win32::System::LibraryLoader::GetModuleHandleW;

        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("InputPilotDaemonMsgWindow");

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(Self::wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };

            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("InputPilotDaemon"),
                WINDOW_STYLE::default(),
                0, 0, 0, 0,
                Some(HWND_MESSAGE), // Message-only window
                None,
                Some(hinstance.into()),
                None,
            );

            hwnd.unwrap_or(HWND(std::ptr::null_mut()))
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

impl Drop for HotkeyDaemon {
    fn drop(&mut self) {
        self.stop();
    }
}
