use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::collections::HashMap;
use parking_lot::Mutex;
use windows::Win32::Foundation::HWND;

use crate::model::hotkey_binding::*;
use crate::model::dispatch_mode::DispatchMode;
use crate::win32::input_dispatcher::*;
use crate::win32::send_input_dispatcher::SendInputDispatcher;
use crate::win32::message_dispatcher::MessageDispatcher;
use crate::win32::logitech_dispatcher::LogitechDispatcher;
use crate::win32::window_finder;
use super::macro_repository::MacroRepository;
use super::macro_executor::execute_steps;
use super::macro_parser;

enum Command {
    Start(i32, HotkeyBinding),
    Stop(i32),
    StopAll,
    Shutdown,
}

pub struct MacroRunner {
    sender: mpsc::Sender<Command>,
    worker: Option<thread::JoinHandle<()>>,
    toggle_states: Arc<Mutex<HashMap<i32, bool>>>,
    active_tasks: Arc<Mutex<HashMap<i32, Arc<AtomicBool>>>>,
    repo: Arc<Mutex<Arc<MacroRepository>>>,
}

impl MacroRunner {
    pub fn new(repo: Arc<MacroRepository>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let toggle_states = Arc::new(Mutex::new(HashMap::new()));
        let active_tasks = Arc::new(Mutex::new(HashMap::new()));
        let repo_shared = Arc::new(Mutex::new(repo));

        let active_tasks_clone = Arc::clone(&active_tasks);
        let repo_clone = Arc::clone(&repo_shared);

        let worker = thread::spawn(move || {
            Self::worker_loop(receiver, repo_clone, active_tasks_clone);
        });

        Self {
            sender,
            worker: Some(worker),
            toggle_states,
            active_tasks,
            repo: repo_shared,
        }
    }

    /// Hot-swap the repository without losing toggle states or active tasks.
    pub fn update_repo(&self, new_repo: Arc<MacroRepository>) {
        *self.repo.lock() = new_repo;
    }

    /// Stop all active macro executions and reset toggle states.
    pub fn stop_all(&self) {
        let _ = self.sender.send(Command::StopAll);
        self.toggle_states.lock().clear();
    }

    pub fn on_hotkey_down(&self, hotkey: &HotkeyBinding) {
        match hotkey.trigger_mode {
            TriggerMode::Once => {
                log::info!("[MacroRunner] Once hotkey {} triggered", hotkey.id);
                let _ = self.sender.send(Command::Start(hotkey.id, hotkey.clone()));
            }
            TriggerMode::Toggle => {
                let mut states = self.toggle_states.lock();
                let is_active = states.get(&hotkey.id).copied().unwrap_or(false);
                log::info!("[MacroRunner] Toggle hotkey {} down, is_active={}", hotkey.id, is_active);
                if is_active {
                    let _ = self.sender.send(Command::Stop(hotkey.id));
                    states.insert(hotkey.id, false);
                } else {
                    let _ = self.sender.send(Command::Start(hotkey.id, hotkey.clone()));
                    states.insert(hotkey.id, true);
                }
            }
            TriggerMode::Hold | TriggerMode::Phased => {
                log::info!("[MacroRunner] Hold/Phased hotkey {} down", hotkey.id);
                let _ = self.sender.send(Command::Start(hotkey.id, hotkey.clone()));
            }
        }
    }

    pub fn on_hotkey_up(&self, hotkey: &HotkeyBinding) {
        match hotkey.trigger_mode {
            TriggerMode::Hold | TriggerMode::Phased => {
                let _ = self.sender.send(Command::Stop(hotkey.id));
            }
            _ => {}
        }
    }

    fn worker_loop(
        receiver: mpsc::Receiver<Command>,
        repo: Arc<Mutex<Arc<MacroRepository>>>,
        active_tasks: Arc<Mutex<HashMap<i32, Arc<AtomicBool>>>>,
    ) {
        while let Ok(cmd) = receiver.recv() {
            match cmd {
                Command::Start(id, hotkey) => {
                    // Cancel any existing task with the same id
                    if let Some(old_flag) = active_tasks.lock().remove(&id) {
                        old_flag.store(true, Ordering::Relaxed);
                    }

                    let cancel_flag = Arc::new(AtomicBool::new(false));
                    active_tasks.lock().insert(id, Arc::clone(&cancel_flag));

                    let repo_snapshot = Arc::clone(&*repo.lock());
                    let cancel_clone = Arc::clone(&cancel_flag);

                    thread::spawn(move || {
                        Self::execute_hotkey(&repo_snapshot, &hotkey, cancel_clone);
                    });
                }
                Command::Stop(id) => {
                    if let Some(cancel_flag) = active_tasks.lock().remove(&id) {
                        cancel_flag.store(true, Ordering::Relaxed);
                    }
                }
                Command::StopAll => {
                    let mut tasks = active_tasks.lock();
                    for cancel_flag in tasks.values() {
                        cancel_flag.store(true, Ordering::Relaxed);
                    }
                    tasks.clear();
                }
                Command::Shutdown => break,
            }
        }
    }

    fn execute_hotkey(repo: &MacroRepository, hotkey: &HotkeyBinding, cancel_flag: Arc<AtomicBool>) {
        let sequence = match hotkey.action {
            HotkeyActionType::PlayMacro => {
                repo.find_macro(&hotkey.macro_name).cloned()
            }
            HotkeyActionType::PlayFile => {
                match macro_parser::load_macro_file(&hotkey.macro_name, &hotkey.target_name, &hotkey.file_path) {
                    Ok(seq) => Some(seq),
                    Err(e) => {
                        log::error!("Failed to load macro file '{}': {}", hotkey.file_path, e);
                        None
                    }
                }
            }
            HotkeyActionType::RecordToggle => {
                None // Handled elsewhere
            }
        };

        let Some(mut sequence) = sequence else {
            log::warn!("Macro not found: {}", hotkey.macro_name);
            return;
        };

        // Apply dispatch override if present
        if hotkey.has_dispatch_override {
            sequence.dispatch_mode = hotkey.dispatch_mode;
        }
        if !hotkey.target_name.is_empty() {
            sequence.target_name = hotkey.target_name.clone();
        }

        // Find target window
        let target_spec = repo.find_target(&sequence.target_name);
        let window_match = if let Some(target) = target_spec {
            window_finder::find_first(target)
        } else {
            None
        };

        let mut dispatcher = Self::create_dispatcher(sequence.dispatch_mode);
        
        // Attach to window if found
        if let Some(w) = window_match {
            dispatcher.attach(HWND(w.handle as _));
        }

        match hotkey.trigger_mode {
            TriggerMode::Once => {
                let steps = if sequence.has_phases {
                    &sequence.on_press_steps
                } else {
                    &sequence.steps
                };
                execute_steps(dispatcher.as_ref(), steps);
            }
            TriggerMode::Toggle | TriggerMode::Hold => {
                let steps = if sequence.has_phases {
                    &sequence.on_hold_steps
                } else {
                    &sequence.steps
                };
                
                while !cancel_flag.load(Ordering::Relaxed) {
                    if !execute_steps(dispatcher.as_ref(), steps) {
                        break;
                    }
                    if hotkey.repeat_delay_ms > 0 {
                        thread::sleep(std::time::Duration::from_millis(hotkey.repeat_delay_ms as u64));
                    } else {
                        // Yield to prevent busy-loop starving the system
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            }
            TriggerMode::Phased => {
                // on_press
                execute_steps(dispatcher.as_ref(), &sequence.on_press_steps);

                // on_hold loop
                while !cancel_flag.load(Ordering::Relaxed) {
                    if !execute_steps(dispatcher.as_ref(), &sequence.on_hold_steps) {
                        break;
                    }
                    if hotkey.repeat_delay_ms > 0 {
                        thread::sleep(std::time::Duration::from_millis(hotkey.repeat_delay_ms as u64));
                    } else {
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                }

                // on_release (uncancellable)
                execute_steps(dispatcher.as_ref(), &sequence.on_release_steps);
            }
        }

        log::info!("[MacroRunner] Hotkey {} execution finished", hotkey.id);
    }

    fn create_dispatcher(mode: DispatchMode) -> Box<dyn InputDispatcher> {
        match mode {
            DispatchMode::WindowMessage => Box::new(MessageDispatcher::new()),
            DispatchMode::Logitech => Box::new(LogitechDispatcher::new()),
            DispatchMode::SendInput => Box::new(SendInputDispatcher::new()),
        }
    }
}

impl Drop for MacroRunner {
    fn drop(&mut self) {
        // Cancel all active execution threads first
        let tasks = self.active_tasks.lock();
        for cancel_flag in tasks.values() {
            cancel_flag.store(true, Ordering::Relaxed);
        }
        drop(tasks);

        // Then shut down the worker loop
        let _ = self.sender.send(Command::Shutdown);
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}
