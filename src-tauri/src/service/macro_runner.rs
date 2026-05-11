use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::collections::HashMap;
use parking_lot::Mutex;
use windows::Win32::Foundation::HWND;

use crate::model::hotkey_binding::*;
use crate::model::macro_step::*;
use crate::model::macro_sequence::MacroSequence;
use crate::model::dispatch_mode::DispatchMode;
use crate::win32::input_dispatcher::*;
use crate::win32::send_input_dispatcher::SendInputDispatcher;
use crate::win32::message_dispatcher::MessageDispatcher;
use crate::win32::window_finder;
use super::macro_repository::MacroRepository;
use super::macro_executor::execute_steps;

enum Command {
    Start(i32, HotkeyBinding),
    Stop(i32),
    Shutdown,
}

pub struct MacroRunner {
    sender: mpsc::Sender<Command>,
    worker: Option<thread::JoinHandle<()>>,
    toggle_states: Arc<Mutex<HashMap<i32, bool>>>,
    active_tasks: Arc<Mutex<HashMap<i32, Arc<AtomicBool>>>>,
}

impl MacroRunner {
    pub fn new(repo: Arc<MacroRepository>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let toggle_states = Arc::new(Mutex::new(HashMap::new()));
        let active_tasks = Arc::new(Mutex::new(HashMap::new()));

        let active_tasks_clone = Arc::clone(&active_tasks);

        let worker = thread::spawn(move || {
            Self::worker_loop(receiver, repo, active_tasks_clone);
        });

        Self {
            sender,
            worker: Some(worker),
            toggle_states,
            active_tasks,
        }
    }

    pub fn on_hotkey_down(&self, hotkey: &HotkeyBinding) {
        match hotkey.trigger_mode {
            TriggerMode::Once => {
                let _ = self.sender.send(Command::Start(hotkey.id, hotkey.clone()));
            }
            TriggerMode::Toggle => {
                let mut states = self.toggle_states.lock();
                let is_active = states.get(&hotkey.id).copied().unwrap_or(false);
                if is_active {
                    let _ = self.sender.send(Command::Stop(hotkey.id));
                    states.insert(hotkey.id, false);
                } else {
                    let _ = self.sender.send(Command::Start(hotkey.id, hotkey.clone()));
                    states.insert(hotkey.id, true);
                }
            }
            TriggerMode::Hold | TriggerMode::Phased => {
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
        repo: Arc<MacroRepository>,
        active_tasks: Arc<Mutex<HashMap<i32, Arc<AtomicBool>>>>,
    ) {
        while let Ok(cmd) = receiver.recv() {
            match cmd {
                Command::Start(id, hotkey) => {
                    let cancel_flag = Arc::new(AtomicBool::new(false));
                    active_tasks.lock().insert(id, Arc::clone(&cancel_flag));

                    let repo_clone = Arc::clone(&repo);
                    let cancel_clone = Arc::clone(&cancel_flag);

                    thread::spawn(move || {
                        Self::execute_hotkey(&repo_clone, &hotkey, cancel_clone);
                    });
                }
                Command::Stop(id) => {
                    if let Some(cancel_flag) = active_tasks.lock().remove(&id) {
                        cancel_flag.store(true, Ordering::Relaxed);
                    }
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
                // Load from file_path
                None // TODO: implement file loading
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
                    }
                }

                // on_release (uncancellable)
                execute_steps(dispatcher.as_ref(), &sequence.on_release_steps);
            }
        }
    }

    fn create_dispatcher(mode: DispatchMode) -> Box<dyn InputDispatcher> {
        match mode {
            DispatchMode::WindowMessage => Box::new(MessageDispatcher::new()),
            DispatchMode::Logitech => Box::new(SendInputDispatcher::new()),
            DispatchMode::SendInput => Box::new(SendInputDispatcher::new()),
        }
    }
}

impl Drop for MacroRunner {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}
