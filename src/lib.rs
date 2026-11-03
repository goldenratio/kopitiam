use anyhow::{Context as AnyhowContext, Result, anyhow};
use winit::event_loop::EventLoop;

pub use boa_engine::{Context as JsContext, JsNativeError, JsResult, JsValue};
pub use engine_options::EngineOptions;
pub use native_function_binding::{NativeFunction, NativeFunctionBinding};

mod assets_utils;
#[cfg(feature = "audio")]
mod audio;
mod boa_errors;
mod boa_utils;
mod canvas_interaction;
mod canvas_scale;
mod draw_command;
mod engine_app;
mod engine_options;
mod native_bridge;
mod native_function_binding;
mod profile;
mod skia_utils;
mod software_renderer;
mod window_config;

use crate::{engine_app::EngineApp, native_bridge::NativeBridge, window_config::WindowConfig};

pub fn run_script(script: &str, options: EngineOptions) -> Result<()> {
  let config = options.window.clone().normalized();
  let mut bridge = NativeBridge::new(script, &options)?;
  if bridge.has_exit_command()? {
    return Ok(());
  }

  bridge.drain_console_messages()?;
  run_engine(bridge, config)
}

fn run_engine(bridge: NativeBridge, config: WindowConfig) -> Result<()> {
  let event_loop = EventLoop::new().context("failed to create window event loop")?;
  let mut app = EngineApp::new(bridge, config)?;
  event_loop
    .run_app(&mut app)
    .map_err(|err| anyhow!("window event loop failed: {err}"))?;
  app.result
}
