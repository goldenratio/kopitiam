use std::{
  env,
  sync::Arc,
  time::{Duration, Instant},
};

use anyhow::{Context as AnyhowContext, Result};
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, PhysicalSize},
  event::{ElementState, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow},
  keyboard::PhysicalKey,
  window::{Fullscreen, Window as WinitWindow, WindowAttributes, WindowId},
};

use crate::{
  canvas_interaction::{self, KeyboardState, PendingKeyEvent, PointerState, key_name_from_code},
  canvas_scale::CanvasScaleMode,
  draw_command,
  native_bridge::NativeBridge,
  profile::{FrameProfile, FrameProfileSample},
  software_renderer::SoftwareRenderer,
  window_config::WindowConfig,
};

pub struct EngineApp {
  bridge: Option<NativeBridge>,
  config: WindowConfig,
  scale_mode: CanvasScaleMode,
  window: Option<Arc<WinitWindow>>,
  renderer: Option<SoftwareRenderer>,
  canvas_width: usize,
  canvas_height: usize,
  window_width: usize,
  window_height: usize,
  pointer: PointerState,
  keyboard: KeyboardState,
  pending_key_events: Vec<PendingKeyEvent>,
  started: Instant,
  next_frame_at: Instant,
  profile: bool,
  profile_stats: FrameProfile,
  // last_gc: Instant,
  pub(crate) result: Result<()>,
}

impl EngineApp {
  pub fn new(bridge: NativeBridge, config: WindowConfig) -> Result<Self> {
    let scale_mode = CanvasScaleMode::from_config(config.scale_mode.as_deref())?;
    let canvas_width = config.width;
    let canvas_height = config.height;

    Ok(Self {
      bridge: Some(bridge),
      config,
      scale_mode,
      window: None,
      renderer: None,
      canvas_width,
      canvas_height,
      window_width: canvas_width,
      window_height: canvas_height,
      pointer: PointerState::default(),
      keyboard: KeyboardState::default(),
      pending_key_events: Vec::new(),
      started: Instant::now(),
      next_frame_at: Instant::now(),
      profile: env::var("CANVAS_PROFILE").is_ok_and(|value| value != "0"),
      profile_stats: FrameProfile::default(),
      // last_gc: Instant::now(),
      result: Ok(()),
    })
  }

  fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
    if self.window.is_some() {
      return Ok(());
    }

    let attributes = WindowAttributes::default()
      .with_title(self.config.title.clone())
      .with_inner_size(LogicalSize::new(
        self.config.width as f64,
        self.config.height as f64,
      ))
      .with_resizable(self.config.resizable || self.config.fullscreen)
      .with_decorations(!self.config.fullscreen)
      .with_fullscreen(if self.config.fullscreen {
        Some(Fullscreen::Borderless(None))
      } else {
        None
      });

    let window = Arc::new(
      event_loop
        .create_window(attributes)
        .context("failed to create desktop window")?,
    );

    let size = window.inner_size();
    self.resize_to_window(size)?;

    let renderer = SoftwareRenderer::new(Arc::clone(&window), size)?;

    self.window = Some(window);
    self.renderer = Some(renderer);
    Ok(())
  }

  fn resize_to_window(&mut self, size: PhysicalSize<u32>) -> Result<()> {
    let width = (size.width.max(1) as usize).clamp(1, 7680);
    let height = (size.height.max(1) as usize).clamp(1, 4320);

    if width == self.window_width && height == self.window_height {
      return Ok(());
    }

    self.window_width = width;
    self.window_height = height;

    if self.scale_mode.resizes_canvas() {
      self.canvas_width = width;
      self.canvas_height = height;
    }

    if let Some(bridge) = &mut self.bridge {
      bridge.dispatch_resize(width, height, self.scale_mode.resizes_canvas())?;
    }

    Ok(())
  }

  fn redraw(&mut self) -> Result<bool> {
    const FRAME_INTERVAL: Duration = Duration::from_nanos(16_666_667);

    // Run garbage collection every 30 seconds
    // if self.last_gc.elapsed() >= Duration::from_secs(30) {
    //   if let Some(bridge) = &mut self.bridge {
    //     bridge.collect_garbage();
    //   }
    //   self.last_gc = Instant::now();
    // }

    let frame_started = Instant::now();
    let timestamp_ms = self.started.elapsed().as_secs_f64() * 1000.0;

    if let Some(bridge) = &mut self.bridge {
      bridge.dispatch_pointer_events(
        &mut self.pointer,
        self.scale_mode,
        self.canvas_width,
        self.canvas_height,
        self.window_width,
        self.window_height,
      )?;

      bridge.dispatch_keyboard_events(&self.keyboard, &mut self.pending_key_events)?;

      let input_take_started = Instant::now();
      let mut commands = bridge.take_draw_commands()?;
      let mut take_elapsed = input_take_started.elapsed();

      if draw_command::contains_exit_command(&commands) {
        return Ok(true);
      }

      let js_started = Instant::now();
      bridge.dispatch_tick(timestamp_ms)?;
      let js_elapsed = js_started.elapsed();

      let take_started = Instant::now();
      commands.extend(bridge.take_draw_commands()?);
      take_elapsed += take_started.elapsed();

      if draw_command::contains_exit_command(&commands) {
        return Ok(true);
      }

      let js_profile = crate::profile::take_js_profile_sample();
      let command_count = commands.len();
      let batch_draw_count = draw_command::batch_draw_count(&commands);

      let renderer = self
        .renderer
        .as_mut()
        .context("renderer was not initialized")?;

      let timings = renderer.render(
        self.scale_mode,
        &commands,
        self.canvas_width,
        self.canvas_height,
        self.window_width,
        self.window_height,
      )?;

      if self.profile {
        self.profile_stats.record(FrameProfileSample {
          frame: frame_started.elapsed(),
          js: js_elapsed,
          take: take_elapsed,
          render: timings.render,
          readback: timings.readback,
          present: timings.present,
          commands: command_count,
          batch_draws: batch_draw_count,
          js_profile,
        });
      }
    }

    self.next_frame_at = frame_started + FRAME_INTERVAL;

    Ok(false)
  }

  fn fail(&mut self, event_loop: &ActiveEventLoop, err: anyhow::Error) {
    self.result = Err(err);
    event_loop.exit();
  }
}

impl ApplicationHandler for EngineApp {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_at));
    if let Err(err) = self.create_window(event_loop) {
      self.fail(event_loop, err);
    }
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let Some(window) = &self.window else {
      return;
    };
    if window_id != window.id() {
      return;
    }

    match event {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::Resized(size) => {
        if let Err(err) = self.resize_to_window(size) {
          self.fail(event_loop, err);
        }
      }
      WindowEvent::CursorMoved { position, .. } => {
        self.pointer.window_position = Some((position.x as f32, position.y as f32));
      }
      WindowEvent::CursorLeft { .. } => {
        self.pointer.window_position = None;
      }
      WindowEvent::MouseInput { state, button, .. } => {
        canvas_interaction::update_mouse_buttons(&mut self.pointer, button, state);
      }
      WindowEvent::MouseWheel { delta, .. } => {
        let (delta_x, delta_y) = canvas_interaction::scroll_delta(delta);
        self.pointer.scroll_delta.0 += delta_x;
        self.pointer.scroll_delta.1 += delta_y;
      }
      WindowEvent::ModifiersChanged(modifiers) => {
        let state = modifiers.state();
        self.keyboard.shift_key = state.shift_key();
        self.keyboard.ctrl_key = state.control_key();
        self.keyboard.alt_key = state.alt_key();
        self.keyboard.meta_key = state.super_key();
      }
      WindowEvent::KeyboardInput { event, .. } => {
        if let PhysicalKey::Code(code) = event.physical_key
          && let Some((key, code)) = key_name_from_code(code)
        {
          self.pending_key_events.push(PendingKeyEvent {
            event_type: match event.state {
              ElementState::Pressed => "keydown",
              ElementState::Released => "keyup",
            },
            key,
            code,
            repeat: event.repeat,
          });

          self.next_frame_at = Instant::now();
          window.request_redraw();
        }
      }
      WindowEvent::RedrawRequested => match self.redraw() {
        Ok(true) => event_loop.exit(),
        Ok(false) => {}
        Err(err) => self.fail(event_loop, err),
      },
      _ => {}
    }
  }

  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    let now = Instant::now();

    if now >= self.next_frame_at
      && let Some(window) = &self.window
    {
      window.request_redraw();
    }

    event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_at));
  }
}
