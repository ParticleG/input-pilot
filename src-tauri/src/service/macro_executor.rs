use anyhow::{bail, Result};
use windows::Win32::Foundation::HWND;
use crate::model::dispatch_mode::DispatchMode;
use crate::model::macro_sequence::MacroSequence;
use crate::model::macro_step::*;
use crate::win32::input_dispatcher::*;
use crate::win32::send_input_dispatcher::SendInputDispatcher;
use crate::win32::message_dispatcher::MessageDispatcher;
use crate::win32::logitech_dispatcher::LogitechDispatcher;
use crate::win32::window_finder;
use super::macro_repository::MacroRepository;

fn create_dispatcher(mode: DispatchMode) -> Box<dyn InputDispatcher> {
    match mode {
        DispatchMode::WindowMessage => Box::new(MessageDispatcher::new()),
        DispatchMode::Logitech => Box::new(LogitechDispatcher::new()),
        DispatchMode::SendInput => Box::new(SendInputDispatcher::new()),
    }
}

pub fn execute(repo: &MacroRepository, sequence: &MacroSequence) -> Result<bool> {
    // Find target window
    let target_spec = repo.find_target(&sequence.target_name);
    if target_spec.is_none() {
        bail!("Target '{}' not found in config", sequence.target_name);
    }
    let target = target_spec.unwrap();

    let window_match = window_finder::find_first(target);

    if window_match.is_none() && sequence.dispatch_mode == DispatchMode::WindowMessage {
        bail!("Target window not found for WindowMessage dispatch");
    }

    // Create dispatcher
    let mut dispatcher = create_dispatcher(sequence.dispatch_mode);
    
    // Attach to window if found
    if let Some(w) = window_match {
        dispatcher.attach(HWND(w.handle as _));
    }

    // Execute steps
    let steps = if sequence.has_phases {
        &sequence.on_press_steps
    } else {
        &sequence.steps
    };

    let success = execute_steps(dispatcher.as_ref(), steps);
    Ok(success)
}

pub fn execute_steps(dispatcher: &dyn InputDispatcher, steps: &[MacroStep]) -> bool {
    for step in steps {
        let ok = match step {
            MacroStep::Delay { milliseconds } => {
                std::thread::sleep(std::time::Duration::from_millis(*milliseconds as u64));
                true
            }
            MacroStep::Key { virtual_key, action } => {
                dispatcher.send_key(&KeyStep { virtual_key: *virtual_key, action: *action })
            }
            MacroStep::MouseMove { x, y, coordinate_mode } => {
                dispatcher.move_mouse(&MouseMoveStep { x: *x, y: *y, coordinate_mode: *coordinate_mode })
            }
            MacroStep::MouseClick { button, action } => {
                dispatcher.click_mouse(&MouseClickStep { button: *button, action: *action })
            }
            MacroStep::Text { text } => {
                dispatcher.send_text(&TextStep { text: text.clone() })
            }
        };
        if !ok {
            log::warn!("Step failed: {:?}", step);
            return false;
        }
    }
    true
}
