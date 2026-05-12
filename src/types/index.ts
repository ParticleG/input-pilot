export interface TargetSpec {
  name: string;
  process_name: string;
  window_class: string;
  window_title: string;
  title_match_mode: 'Ignore' | 'Exact' | 'Contains';
  top_level_only: boolean;
  visible_only: boolean;
}

export type MacroStep =
  | { Delay: { milliseconds: number } }
  | { Key: { virtual_key: number; action: 'Tap' | 'Down' | 'Up' } }
  | { MouseMove: { x: number; y: number; coordinate_mode: 'Screen' | 'Client' } }
  | { MouseClick: { button: 'Left' | 'Right' | 'Middle'; action: 'Click' | 'Down' | 'Up' } }
  | { Text: { text: string } };

export interface MacroSequence {
  name: string;
  target_name: string;
  dispatch_mode: 'SendInput' | 'WindowMessage' | 'Logitech';
  source_file: string;
  steps: MacroStep[];
  on_press_steps: MacroStep[];
  on_hold_steps: MacroStep[];
  on_release_steps: MacroStep[];
  has_phases: boolean;
}

export interface HotkeyBinding {
  id: number;
  modifiers: number;
  virtual_key: number;
  action: 'PlayMacro' | 'RecordToggle' | 'PlayFile';
  trigger_mode: 'Once' | 'Toggle' | 'Hold' | 'Phased';
  repeat_delay_ms: number;
  macro_name: string;
  file_path: string;
  target_name: string;
  dispatch_mode: 'SendInput' | 'WindowMessage' | 'Logitech';
  has_dispatch_override: boolean;
  description: string;
}

export interface HotkeyStateEvent {
  id: number;
  active: boolean;
  description: string;
  trigger_mode: string;
}

export interface AppConfig {
  targets: Record<string, TargetSpec>;
  macros: Record<string, MacroSequence>;
  hotkeys: HotkeyBinding[];
  recordings_directory: string;
}

export interface WindowMatch {
  handle: number;
  title: string;
  class_name: string;
  process_name: string;
}
