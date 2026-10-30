use std::{fs, str, time::Instant};

use anyhow::{Context as AnyhowContext, Result, anyhow};
use boa_engine::{
  Context as JsContext, JsNativeError, JsResult, JsValue, NativeFunction, Source, js_string,
  native_function::NativeFunctionPointer, string::JsString,
};
use serde::{Deserialize, Serialize};

use crate::{
  assets_utils::{decode_data_url, resolve_asset_path},
  boa_errors::{self, AnyhowJsExt},
  boa_utils::{self, value_to_object},
  canvas_interaction::{self, KeyboardState, PendingKeyEvent, PointerState},
  canvas_scale::CanvasScaleMode,
  draw_command::{self, DrawCommand, draw_batch, draw_commands_from_array},
  engine_options::EngineOptions,
};

const JS_PRELUDE: &str = include_str!("gen/prelude.js");

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyUiEvent {
  #[serde(rename = "type")]
  event_type: String,
  key: String,
  code: String,
  repeat: bool,
  shift_key: bool,
  ctrl_key: bool,
  alt_key: bool,
  meta_key: bool,
}

#[derive(Debug, Deserialize)]
struct ConsoleMessage {
  level: String,
  message: String,
}

pub(crate) struct NativeBridge {
  js: JsContext,
  console: bool,
}

impl NativeBridge {
  pub(crate) fn new(script: &str, options: &EngineOptions) -> Result<NativeBridge> {
    let mut bridge = Self {
      js: JsContext::default(),
      console: options.console,
    };

    register_engine_natives(&mut bridge).context("failed to register JavaScript engine natives")?;
    register_custom_natives(&mut bridge, options)
      .context("failed to register custom JavaScript natives")?;

    let prelude = JS_PRELUDE
      .replace(
        "__ENGINE_CONSOLE_ENABLED__",
        if options.console { "true" } else { "false" },
      )
      .replace(
        "__ENGINE_PROFILE_ENABLED__",
        if crate::profile::is_enabled() {
          "true"
        } else {
          "false"
        },
      )
      .replace(
        "__ENGINE_AUDIO_ENABLED__",
        if cfg!(feature = "audio") {
          "true"
        } else {
          "false"
        },
      )
      .replace(
        "__ENGINE_WINDOW_CONFIG__",
        &serde_json::to_string(&options.window.clone().normalized())
          .context("failed to serialize engine window config")?,
      );

    eval(
      &mut bridge.js,
      &prelude,
      "failed to initialize JavaScript engine prelude",
    )?;

    eval(&mut bridge.js, script, "failed to execute user JavaScript")?;

    Ok(bridge)
  }

  // pub(crate) fn collect_garbage(&mut self) {
  //   boa_engine::gc::force_collect();
  // }

  pub(crate) fn register_native(
    &mut self,
    name: &str,
    length: usize,
    function: NativeFunctionPointer,
  ) -> Result<()> {
    self
      .js
      .register_global_builtin_callable(
        JsString::from(name),
        length,
        NativeFunction::from_fn_ptr(function),
      )
      .map_err(boa_errors::js_error)
  }

  pub(crate) fn has_exit_command(&mut self) -> Result<bool> {
    let global = self.js.global_object();

    let commands_value = global
      .get(js_string!("__engine_draw_commands"), &mut self.js)
      .map_err(boa_errors::js_error)?;

    let commands_array = value_to_object(commands_value, "__engine_draw_commands")?;
    let len = boa_utils::array_len(&commands_array, &mut self.js)?;

    for index in 0..len {
      let value = commands_array
        .get(index as u32, &mut self.js)
        .map_err(boa_errors::js_error)?;
      if value.is_null_or_undefined() {
        continue;
      }

      let object = value_to_object(value, "draw command")?;
      if boa_utils::required_string(&object, "op", &mut self.js)? == "exit" {
        return Ok(true);
      }
    }

    Ok(false)
  }

  pub(crate) fn dispatch_resize(
    &mut self,
    width: usize,
    height: usize,
    resize_canvas: bool,
  ) -> Result<()> {
    call_global_function(
      &mut self.js,
      "__engine_dispatch_resize",
      &[
        JsValue::from(width as u32),
        JsValue::from(height as u32),
        JsValue::from(resize_canvas),
      ],
    )
  }

  pub(crate) fn dispatch_pointer_events(
    &mut self,
    pointer: &mut PointerState,
    scale_mode: CanvasScaleMode,
    canvas_width: usize,
    canvas_height: usize,
    window_width: usize,
    window_height: usize,
  ) -> Result<()> {
    let events = canvas_interaction::take_pointer_events(
      pointer,
      scale_mode,
      canvas_width,
      canvas_height,
      window_width,
      window_height,
    );

    if events.is_empty() {
      return Ok(());
    }

    let json = serde_json::to_string(&events).context("failed to serialize input events")?;
    eval(
      &mut self.js,
      &format!("globalThis.__engine_dispatch_events({json})"),
      "failed to dispatch input events",
    )
  }

  pub(crate) fn dispatch_keyboard_events(
    &mut self,
    keyboard: &KeyboardState,
    pending_key_events: &mut Vec<PendingKeyEvent>,
  ) -> Result<()> {
    let mut events = Vec::new();

    for event in pending_key_events.drain(..) {
      events.push(KeyUiEvent {
        event_type: event.event_type.to_owned(),
        key: event.key.to_owned(),
        code: event.code.to_owned(),
        repeat: event.repeat,
        shift_key: keyboard.shift_key,
        ctrl_key: keyboard.ctrl_key,
        alt_key: keyboard.alt_key,
        meta_key: keyboard.meta_key,
      });
    }

    if events.is_empty() {
      return Ok(());
    }

    let json = serde_json::to_string(&events).context("failed to serialize keyboard events")?;
    eval(
      &mut self.js,
      &format!("globalThis.__engine_dispatch_key_events({json})"),
      "failed to dispatch keyboard events",
    )
  }

  pub(crate) fn dispatch_tick(&mut self, timestamp_ms: f64) -> Result<()> {
    call_global_function(
      &mut self.js,
      "__engine_dispatch_tick",
      &[JsValue::from(timestamp_ms)],
    )?;
    self.drain_console_messages()
  }

  pub(crate) fn take_draw_commands(&mut self) -> Result<Vec<DrawCommand>> {
    let global = self.js.global_object();

    let commands_value = global
      .get(js_string!("__engine_draw_commands"), &mut self.js)
      .map_err(boa_errors::js_error)?;

    let commands_array = value_to_object(commands_value, "__engine_draw_commands")?;

    let commands = draw_commands_from_array(&commands_array, &mut self.js)
      .context("JavaScript produced invalid draw commands")?;

    commands_array
      .set(js_string!("length"), 0, true, &mut self.js)
      .map_err(boa_errors::js_error)?;

    draw_batch::clear_draw_batches();

    eval(
      &mut self.js,
      "globalThis.__engine_invalidate_draw_batches?.()",
      "failed to invalidate JavaScript draw batches",
    )?;

    Ok(commands)
  }

  pub(crate) fn drain_console_messages(&mut self) -> Result<()> {
    let started = Instant::now();
    if !self.console {
      crate::profile::record_console_drain(started.elapsed());
      return Ok(());
    }

    let global = self.js.global_object();
    let dirty = global
      .get(js_string!("__engine_console_dirty"), &mut self.js)
      .map_err(boa_errors::js_error)?
      .to_boolean();
    if !dirty {
      crate::profile::record_console_drain(started.elapsed());
      return Ok(());
    }

    let json = eval_to_string(
      &mut self.js,
      "globalThis.__engine_console_dirty = false; JSON.stringify(globalThis.__engine_console_messages.splice(0))",
      "failed to drain JavaScript console messages",
    )?;

    let messages: Vec<ConsoleMessage> =
      serde_json::from_str(&json).context("JavaScript produced invalid console messages")?;

    for ConsoleMessage { level, message } in messages {
      match level.as_str() {
        "warn" => eprintln!("[warn] {message}"),
        "error" => eprintln!("[error] {message}"),
        "debug" => println!("[debug] {message}"),
        "info" => println!("[info] {message}"),
        _ => println!("{message}"),
      }
    }

    crate::profile::record_console_drain(started.elapsed());
    Ok(())
  }
}

fn register_engine_natives(bridge: &mut NativeBridge) -> Result<()> {
  bridge.register_native("__engine_fetch", 1, native_fetch)?;
  crate::profile::register_natives(bridge)?;
  draw_command::draw_batch::register_natives(bridge)?;
  draw_command::register_natives(bridge)?;
  #[cfg(feature = "audio")]
  crate::audio::register_natives(bridge)?;
  Ok(())
}

fn native_fetch(_this: &JsValue, args: &[JsValue], js: &mut JsContext) -> JsResult<JsValue> {
  let url = args
    .first()
    .cloned()
    .unwrap_or_default()
    .to_string(js)?
    .to_std_string_escaped();

  if url.starts_with("http://") || url.starts_with("https://") {
    return Err(js_type_error(format!(
      "fetch() only supports local files and data URLs, got `{url}`"
    )));
  }

  let bytes = if let Some(bytes) = decode_data_url(&url).map_err(anyhow_to_js_error)? {
    bytes
  } else {
    let path = resolve_asset_path(url.strip_prefix("file://").unwrap_or(&url));
    fs::read(&path)
      .map_err(|err| js_range_error(format!("failed to fetch `{}`: {err}", path.display())))?
  };

  let text = str::from_utf8(&bytes)
    .map_err(|err| js_type_error(format!("fetch() response is not valid UTF-8: {err}")))?;
  Ok(JsValue::from(JsString::from(text)))
}

fn js_type_error(message: impl Into<String>) -> boa_engine::JsError {
  JsNativeError::typ().with_message(message.into()).into()
}

fn js_range_error(message: impl Into<String>) -> boa_engine::JsError {
  JsNativeError::range().with_message(message.into()).into()
}

fn anyhow_to_js_error(err: anyhow::Error) -> boa_engine::JsError {
  js_type_error(err.to_string())
}

fn register_custom_natives(bridge: &mut NativeBridge, options: &EngineOptions) -> Result<()> {
  for native in &options.native_functions {
    bridge.register_native(native.name(), native.length(), native.function())?;
  }
  Ok(())
}

fn eval(js: &mut JsContext, source: &str, message: &'static str) -> Result<()> {
  js.eval(Source::from_bytes(source))
    .context_js(js, message)?;
  js.run_jobs().context_js(js, message)?;
  Ok(())
}

fn eval_to_string(js: &mut JsContext, source: &str, message: &'static str) -> Result<String> {
  let value = js
    .eval(Source::from_bytes(source))
    .context_js(js, message)?;
  let text = value.to_string(js).context_js(js, message)?;

  Ok(text.to_std_string_escaped())
}

fn call_global_function(js: &mut JsContext, name: &str, args: &[JsValue]) -> Result<()> {
  let global = js.global_object();

  let callback = global
    .get(js_string!(name), js)
    .context_js(js, "failed to load JavaScript function")?
    .as_object()
    .ok_or_else(|| anyhow!("globalThis.{name} is not callable"))?;

  callback
    .call(&JsValue::undefined(), args, js)
    .context_js(js, "failed to call JavaScript function")?;

  js.run_jobs()
    .context_js(js, "failed to run JavaScript jobs")?;

  Ok(())
}
