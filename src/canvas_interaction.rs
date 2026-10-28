use serde::Serialize;
use winit::{
  event::{ElementState, MouseButton, MouseScrollDelta},
  keyboard::KeyCode,
};

use crate::canvas_scale::CanvasScaleMode;

#[derive(Debug, Default)]
pub(crate) struct PointerState {
  pub(crate) window_position: Option<(f32, f32)>,
  pub(crate) stage_position: Option<(f32, f32)>,
  pub(crate) scroll_delta: (f32, f32),
  buttons: u16,
  previous_buttons: u16,
}

#[derive(Debug, Default)]
pub(crate) struct KeyboardState {
  pub(crate) shift_key: bool,
  pub(crate) ctrl_key: bool,
  pub(crate) alt_key: bool,
  pub(crate) meta_key: bool,
}

#[derive(Debug)]
pub(crate) struct PendingKeyEvent {
  pub(crate) event_type: &'static str,
  pub(crate) key: &'static str,
  pub(crate) code: &'static str,
  pub(crate) repeat: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UiEvent {
  #[serde(rename = "type")]
  event_type: String,
  pointer_id: u32,
  pointer_type: &'static str,
  is_primary: bool,
  pressure: f32,
  tangential_pressure: f32,
  width: f32,
  height: f32,
  tilt_x: i32,
  tilt_y: i32,
  twist: i32,
  button: i32,
  buttons: u16,
  x: f32,
  y: f32,
  page_x: f32,
  page_y: f32,
  delta_x: f32,
  delta_y: f32,
  shift_key: bool,
  ctrl_key: bool,
  alt_key: bool,
  meta_key: bool,
  coalesced: bool,
}

pub(crate) fn take_pointer_events(
  pointer: &mut PointerState,
  scale_mode: CanvasScaleMode,
  canvas_width: usize,
  canvas_height: usize,
  window_width: usize,
  window_height: usize,
) -> Vec<UiEvent> {
  let position = stage_mouse_position(
    pointer.window_position,
    scale_mode,
    canvas_width,
    canvas_height,
    window_width,
    window_height,
  );
  let buttons = pointer.buttons;
  let mut events = Vec::new();

  match (pointer.stage_position, position) {
    (None, Some((x, y))) => {
      events.push(pointer_event("pointerover", x, y, -1, buttons, false));
      events.push(pointer_event("pointerenter", x, y, -1, buttons, false));
      events.push(mouse_event("mouseenter", x, y, -1, buttons));
    }
    (Some((x, y)), None) => {
      events.push(pointer_event("pointerout", x, y, -1, buttons, false));
      events.push(pointer_event("pointerleave", x, y, -1, buttons, false));
      events.push(mouse_event("mouseleave", x, y, -1, buttons));
    }
    (Some((previous_x, previous_y)), Some((x, y))) if previous_x != x || previous_y != y => {
      events.push(pointer_event("pointermove", x, y, -1, buttons, true));
      events.push(mouse_event("mousemove", x, y, -1, buttons));
    }
    _ => {}
  }

  if let Some((x, y)) = position {
    for (button, mask) in [(0, 1), (1, 4), (2, 2)] {
      let was_down = pointer.previous_buttons & mask != 0;
      let is_down = buttons & mask != 0;

      if !was_down && is_down {
        events.push(pointer_event("pointerdown", x, y, button, buttons, true));
        events.push(mouse_event("mousedown", x, y, button, buttons));
      } else if was_down && !is_down {
        events.push(pointer_event("pointerup", x, y, button, buttons, true));
        events.push(mouse_event("mouseup", x, y, button, buttons));
      }
    }

    let (delta_x, delta_y) = pointer.scroll_delta;
    if delta_x != 0.0 || delta_y != 0.0 {
      events.push(wheel_event(x, y, delta_x, delta_y, buttons));
    }
  }

  pointer.stage_position = position;
  pointer.previous_buttons = buttons;
  pointer.scroll_delta = (0.0, 0.0);

  events
}

fn pointer_event(
  event_type: &str,
  x: f32,
  y: f32,
  button: i32,
  buttons: u16,
  coalesced: bool,
) -> UiEvent {
  UiEvent {
    event_type: event_type.to_owned(),
    pointer_id: 1,
    pointer_type: "mouse",
    is_primary: true,
    pressure: if buttons == 0 { 0.0 } else { 0.5 },
    tangential_pressure: 0.0,
    width: 1.0,
    height: 1.0,
    tilt_x: 0,
    tilt_y: 0,
    twist: 0,
    button,
    buttons,
    x,
    y,
    page_x: x,
    page_y: y,
    delta_x: 0.0,
    delta_y: 0.0,
    shift_key: false,
    ctrl_key: false,
    alt_key: false,
    meta_key: false,
    coalesced,
  }
}

pub(crate) fn key_name_from_code(key: KeyCode) -> Option<(&'static str, &'static str)> {
  match key {
    KeyCode::Digit0 => Some(("0", "Digit0")),
    KeyCode::Digit1 => Some(("1", "Digit1")),
    KeyCode::Digit2 => Some(("2", "Digit2")),
    KeyCode::Digit3 => Some(("3", "Digit3")),
    KeyCode::Digit4 => Some(("4", "Digit4")),
    KeyCode::Digit5 => Some(("5", "Digit5")),
    KeyCode::Digit6 => Some(("6", "Digit6")),
    KeyCode::Digit7 => Some(("7", "Digit7")),
    KeyCode::Digit8 => Some(("8", "Digit8")),
    KeyCode::Digit9 => Some(("9", "Digit9")),
    KeyCode::KeyA => Some(("a", "KeyA")),
    KeyCode::KeyB => Some(("b", "KeyB")),
    KeyCode::KeyC => Some(("c", "KeyC")),
    KeyCode::KeyD => Some(("d", "KeyD")),
    KeyCode::KeyE => Some(("e", "KeyE")),
    KeyCode::KeyF => Some(("f", "KeyF")),
    KeyCode::KeyG => Some(("g", "KeyG")),
    KeyCode::KeyH => Some(("h", "KeyH")),
    KeyCode::KeyI => Some(("i", "KeyI")),
    KeyCode::KeyJ => Some(("j", "KeyJ")),
    KeyCode::KeyK => Some(("k", "KeyK")),
    KeyCode::KeyL => Some(("l", "KeyL")),
    KeyCode::KeyM => Some(("m", "KeyM")),
    KeyCode::KeyN => Some(("n", "KeyN")),
    KeyCode::KeyO => Some(("o", "KeyO")),
    KeyCode::KeyP => Some(("p", "KeyP")),
    KeyCode::KeyQ => Some(("q", "KeyQ")),
    KeyCode::KeyR => Some(("r", "KeyR")),
    KeyCode::KeyS => Some(("s", "KeyS")),
    KeyCode::KeyT => Some(("t", "KeyT")),
    KeyCode::KeyU => Some(("u", "KeyU")),
    KeyCode::KeyV => Some(("v", "KeyV")),
    KeyCode::KeyW => Some(("w", "KeyW")),
    KeyCode::KeyX => Some(("x", "KeyX")),
    KeyCode::KeyY => Some(("y", "KeyY")),
    KeyCode::KeyZ => Some(("z", "KeyZ")),
    KeyCode::F1 => Some(("F1", "F1")),
    KeyCode::F2 => Some(("F2", "F2")),
    KeyCode::F3 => Some(("F3", "F3")),
    KeyCode::F4 => Some(("F4", "F4")),
    KeyCode::F5 => Some(("F5", "F5")),
    KeyCode::F6 => Some(("F6", "F6")),
    KeyCode::F7 => Some(("F7", "F7")),
    KeyCode::F8 => Some(("F8", "F8")),
    KeyCode::F9 => Some(("F9", "F9")),
    KeyCode::F10 => Some(("F10", "F10")),
    KeyCode::F11 => Some(("F11", "F11")),
    KeyCode::F12 => Some(("F12", "F12")),
    KeyCode::F13 => Some(("F13", "F13")),
    KeyCode::F14 => Some(("F14", "F14")),
    KeyCode::F15 => Some(("F15", "F15")),
    KeyCode::ArrowLeft => Some(("ArrowLeft", "ArrowLeft")),
    KeyCode::ArrowRight => Some(("ArrowRight", "ArrowRight")),
    KeyCode::ArrowUp => Some(("ArrowUp", "ArrowUp")),
    KeyCode::ArrowDown => Some(("ArrowDown", "ArrowDown")),
    KeyCode::Quote => Some(("'", "Quote")),
    KeyCode::Backquote => Some(("`", "Backquote")),
    KeyCode::Backslash => Some(("\\", "Backslash")),
    KeyCode::Comma => Some((",", "Comma")),
    KeyCode::Equal => Some(("=", "Equal")),
    KeyCode::BracketLeft => Some(("[", "BracketLeft")),
    KeyCode::Minus => Some(("-", "Minus")),
    KeyCode::Period => Some((".", "Period")),
    KeyCode::BracketRight => Some(("]", "BracketRight")),
    KeyCode::Semicolon => Some((";", "Semicolon")),
    KeyCode::Slash => Some(("/", "Slash")),
    KeyCode::Backspace => Some(("Backspace", "Backspace")),
    KeyCode::Delete => Some(("Delete", "Delete")),
    KeyCode::End => Some(("End", "End")),
    KeyCode::Enter => Some(("Enter", "Enter")),
    KeyCode::Escape => Some(("Escape", "Escape")),
    KeyCode::Home => Some(("Home", "Home")),
    KeyCode::Insert => Some(("Insert", "Insert")),
    KeyCode::ContextMenu => Some(("ContextMenu", "ContextMenu")),
    KeyCode::PageDown => Some(("PageDown", "PageDown")),
    KeyCode::PageUp => Some(("PageUp", "PageUp")),
    KeyCode::Pause => Some(("Pause", "Pause")),
    KeyCode::Space => Some((" ", "Space")),
    KeyCode::Tab => Some(("Tab", "Tab")),
    KeyCode::NumLock => Some(("NumLock", "NumLock")),
    KeyCode::CapsLock => Some(("CapsLock", "CapsLock")),
    KeyCode::ScrollLock => Some(("ScrollLock", "ScrollLock")),
    KeyCode::ShiftLeft => Some(("Shift", "ShiftLeft")),
    KeyCode::ShiftRight => Some(("Shift", "ShiftRight")),
    KeyCode::ControlLeft => Some(("Control", "ControlLeft")),
    KeyCode::ControlRight => Some(("Control", "ControlRight")),
    KeyCode::AltLeft => Some(("Alt", "AltLeft")),
    KeyCode::AltRight => Some(("Alt", "AltRight")),
    KeyCode::SuperLeft => Some(("Meta", "MetaLeft")),
    KeyCode::SuperRight => Some(("Meta", "MetaRight")),
    KeyCode::Numpad0 => Some(("0", "Numpad0")),
    KeyCode::Numpad1 => Some(("1", "Numpad1")),
    KeyCode::Numpad2 => Some(("2", "Numpad2")),
    KeyCode::Numpad3 => Some(("3", "Numpad3")),
    KeyCode::Numpad4 => Some(("4", "Numpad4")),
    KeyCode::Numpad5 => Some(("5", "Numpad5")),
    KeyCode::Numpad6 => Some(("6", "Numpad6")),
    KeyCode::Numpad7 => Some(("7", "Numpad7")),
    KeyCode::Numpad8 => Some(("8", "Numpad8")),
    KeyCode::Numpad9 => Some(("9", "Numpad9")),
    KeyCode::NumpadDecimal => Some((".", "NumpadDecimal")),
    KeyCode::NumpadDivide => Some(("/", "NumpadDivide")),
    KeyCode::NumpadMultiply => Some(("*", "NumpadMultiply")),
    KeyCode::NumpadSubtract => Some(("-", "NumpadSubtract")),
    KeyCode::NumpadAdd => Some(("+", "NumpadAdd")),
    KeyCode::NumpadEnter => Some(("Enter", "NumpadEnter")),
    _ => None,
  }
}

pub(crate) fn update_mouse_buttons(
  pointer: &mut PointerState,
  button: MouseButton,
  state: ElementState,
) {
  let mask = match button {
    MouseButton::Left => 1,
    MouseButton::Right => 2,
    MouseButton::Middle => 4,
    _ => return,
  };

  match state {
    ElementState::Pressed => pointer.buttons |= mask,
    ElementState::Released => pointer.buttons &= !mask,
  }
}

pub(crate) fn scroll_delta(delta: MouseScrollDelta) -> (f32, f32) {
  match delta {
    MouseScrollDelta::LineDelta(x, y) => (x, y),
    MouseScrollDelta::PixelDelta(position) => (position.x as f32, position.y as f32),
  }
}

fn mouse_event(event_type: &str, x: f32, y: f32, button: i32, buttons: u16) -> UiEvent {
  UiEvent {
    event_type: event_type.to_owned(),
    button,
    buttons,
    x,
    y,
    page_x: x,
    page_y: y,
    ..pointer_event(event_type, x, y, button, buttons, false)
  }
}

fn wheel_event(x: f32, y: f32, delta_x: f32, delta_y: f32, buttons: u16) -> UiEvent {
  UiEvent {
    event_type: "wheel".to_owned(),
    delta_x,
    delta_y,
    ..mouse_event("wheel", x, y, -1, buttons)
  }
}

fn stage_mouse_position(
  position: Option<(f32, f32)>,
  scale_mode: CanvasScaleMode,
  canvas_width: usize,
  canvas_height: usize,
  window_width: usize,
  window_height: usize,
) -> Option<(f32, f32)> {
  let (x, y) = position?;
  if scale_mode.resizes_canvas() {
    return Some((x, y));
  }

  map_window_to_stage(
    x,
    y,
    window_width as f32,
    window_height as f32,
    canvas_width as f32,
    canvas_height as f32,
    scale_mode,
  )
}

fn map_window_to_stage(
  x: f32,
  y: f32,
  window_width: f32,
  window_height: f32,
  canvas_width: f32,
  canvas_height: f32,
  scale_mode: CanvasScaleMode,
) -> Option<(f32, f32)> {
  let (scale_x, scale_y, offset_x, offset_y) = match scale_mode {
    CanvasScaleMode::Responsive => return Some((x, y)),
    CanvasScaleMode::ExactFit => (
      window_width / canvas_width,
      window_height / canvas_height,
      0.0,
      0.0,
    ),
    CanvasScaleMode::ShowAll => {
      let scale = (window_width / canvas_width).min(window_height / canvas_height);
      (
        scale,
        scale,
        (window_width - canvas_width * scale) * 0.5,
        (window_height - canvas_height * scale) * 0.5,
      )
    }
    CanvasScaleMode::NoBorder => {
      let scale = (window_width / canvas_width).max(window_height / canvas_height);
      (
        scale,
        scale,
        (window_width - canvas_width * scale) * 0.5,
        (window_height - canvas_height * scale) * 0.5,
      )
    }
    CanvasScaleMode::NoScale => (
      1.0,
      1.0,
      (window_width - canvas_width) * 0.5,
      (window_height - canvas_height) * 0.5,
    ),
  };

  let stage_x = (x - offset_x) / scale_x;
  let stage_y = (y - offset_y) / scale_y;
  if stage_x < 0.0 || stage_y < 0.0 || stage_x >= canvas_width || stage_y >= canvas_height {
    None
  } else {
    Some((stage_x, stage_y))
  }
}
