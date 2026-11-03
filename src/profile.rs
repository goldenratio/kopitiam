use std::{
  cell::RefCell,
  env,
  sync::OnceLock,
  time::{Duration, Instant},
};

use anyhow::Result;
use boa_engine::{Context as JsContext, JsResult, JsValue};

use crate::native_bridge::NativeBridge;

#[derive(Debug, Clone, Copy)]
pub struct RenderTimings {
  pub render: Duration,
  pub readback: Duration,
  pub present: Duration,
}

#[derive(Default, Clone, Copy)]
pub struct JsProfileSample {
  pub tick_dispatch: Duration,
  pub tick_listeners: Duration,
  pub native_batch: Duration,
  pub native_batch_calls: usize,
  pub console_drain: Duration,
}

#[derive(Default)]
struct JsProfileStats {
  tick_dispatch: Duration,
  tick_dispatch_started: Option<Instant>,
  tick_listeners: Duration,
  tick_listener_started: Option<Instant>,
  native_batch: Duration,
  native_batch_calls: usize,
  console_drain: Duration,
}

thread_local! {
  static JS_PROFILE: RefCell<JsProfileStats> = RefCell::new(JsProfileStats::default());
}

pub(crate) fn is_enabled() -> bool {
  static ENABLED: OnceLock<bool> = OnceLock::new();
  *ENABLED.get_or_init(|| env::var("CANVAS_PROFILE").is_ok_and(|value| value != "0"))
}

#[derive(Default)]
pub struct FrameProfile {
  frames: usize,
  commands: usize,
  batch_draws: usize,
  frame: Duration,
  js: Duration,
  take: Duration,
  render: Duration,
  readback: Duration,
  present: Duration,
  tick_dispatch: Duration,
  tick_listeners: Duration,
  tick_overhead: Duration,
  native_batch: Duration,
  native_batch_calls: usize,
  console_drain: Duration,
  window_started: Option<Instant>,
}

pub struct FrameProfileSample {
  pub frame: Duration,
  pub js: Duration,
  pub take: Duration,
  pub render: Duration,
  pub readback: Duration,
  pub present: Duration,
  pub commands: usize,
  pub batch_draws: usize,
  pub js_profile: JsProfileSample,
}

impl FrameProfile {
  pub fn record(&mut self, sample: FrameProfileSample) {
    let now = Instant::now();
    let window_started = *self.window_started.get_or_insert(now);
    self.frames += 1;
    self.commands += sample.commands;
    self.batch_draws += sample.batch_draws;
    self.frame += sample.frame;
    self.js += sample.js;
    self.take += sample.take;
    self.render += sample.render;
    self.readback += sample.readback;
    self.present += sample.present;
    self.tick_dispatch += sample.js_profile.tick_dispatch;
    self.tick_listeners += sample.js_profile.tick_listeners;
    self.tick_overhead += sample
      .js_profile
      .tick_dispatch
      .saturating_sub(sample.js_profile.tick_listeners);
    self.native_batch += sample.js_profile.native_batch;
    self.native_batch_calls += sample.js_profile.native_batch_calls;
    self.console_drain += sample.js_profile.console_drain;

    if now.duration_since(window_started) >= Duration::from_secs(1) {
      let frames = self.frames as f64;
      eprintln!(
        "[profile] fps={:.1} commands/frame={:.1} batchDraws/frame={:.1} frame={:.2}ms js={:.2}ms tickDispatch={:.2}ms tickListeners={:.2}ms tickOverhead={:.2}ms nativeBatch={:.2}ms/{} console={:.2}ms take={:.2}ms render={:.2}ms readback={:.2}ms present={:.2}ms",
        frames / now.duration_since(window_started).as_secs_f64(),
        self.commands as f64 / frames,
        self.batch_draws as f64 / frames,
        duration_ms(self.frame) / frames,
        duration_ms(self.js) / frames,
        duration_ms(self.tick_dispatch) / frames,
        duration_ms(self.tick_listeners) / frames,
        duration_ms(self.tick_overhead) / frames,
        duration_ms(self.native_batch) / frames,
        self.native_batch_calls / self.frames,
        duration_ms(self.console_drain) / frames,
        duration_ms(self.take) / frames,
        duration_ms(self.render) / frames,
        duration_ms(self.readback) / frames,
        duration_ms(self.present) / frames,
      );
      *self = Self {
        window_started: Some(now),
        ..Self::default()
      };
    }
  }
}

fn duration_ms(duration: Duration) -> f64 {
  duration.as_secs_f64() * 1000.0
}

pub(crate) fn register_natives(bridge: &mut NativeBridge) -> Result<()> {
  bridge.register_native("__engine_profile_begin", 1, native_profile_begin)?;
  bridge.register_native("__engine_profile_end", 1, native_profile_end)
}

pub(crate) fn record_native_batch_append(duration: Duration) {
  if !is_enabled() {
    return;
  }
  JS_PROFILE.with(|profile| {
    let mut profile = profile.borrow_mut();
    profile.native_batch += duration;
    profile.native_batch_calls += 1;
  });
}

pub(crate) fn record_console_drain(duration: Duration) {
  if !is_enabled() {
    return;
  }
  JS_PROFILE.with(|profile| profile.borrow_mut().console_drain += duration);
}

pub(crate) fn take_js_profile_sample() -> JsProfileSample {
  if !is_enabled() {
    return JsProfileSample::default();
  }
  JS_PROFILE.with(|profile| {
    let mut profile = profile.borrow_mut();
    JsProfileSample {
      tick_dispatch: std::mem::take(&mut profile.tick_dispatch),
      tick_listeners: std::mem::take(&mut profile.tick_listeners),
      native_batch: std::mem::take(&mut profile.native_batch),
      native_batch_calls: std::mem::take(&mut profile.native_batch_calls),
      console_drain: std::mem::take(&mut profile.console_drain),
    }
  })
}

fn native_profile_begin(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  if !is_enabled() {
    return Ok(JsValue::undefined());
  }
  let name = profile_name(args, js)?;
  JS_PROFILE.with(|profile| {
    let mut profile = profile.borrow_mut();
    match name.as_str() {
      "tickDispatch" => profile.tick_dispatch_started = Some(Instant::now()),
      "tickListener" => profile.tick_listener_started = Some(Instant::now()),
      _ => {}
    }
  });
  Ok(JsValue::undefined())
}

fn native_profile_end(_this: &JsValue, args: &[JsValue], js: &mut JsContext) -> JsResult<JsValue> {
  if !is_enabled() {
    return Ok(JsValue::undefined());
  }
  let name = profile_name(args, js)?;
  JS_PROFILE.with(|profile| {
    let mut profile = profile.borrow_mut();
    match name.as_str() {
      "tickDispatch" => {
        if let Some(started) = profile.tick_dispatch_started.take() {
          profile.tick_dispatch += started.elapsed();
        }
      }
      "tickListener" => {
        if let Some(started) = profile.tick_listener_started.take() {
          profile.tick_listeners += started.elapsed();
        }
      }
      _ => {}
    }
  });
  Ok(JsValue::undefined())
}

fn profile_name(args: &[JsValue], js: &mut JsContext) -> JsResult<String> {
  Ok(
    args
      .first()
      .cloned()
      .unwrap_or_default()
      .to_string(js)?
      .to_std_string_escaped(),
  )
}
