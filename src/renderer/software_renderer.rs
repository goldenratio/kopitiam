use std::{sync::Arc, time::Instant};

use anyhow::{Context as AnyhowContext, Result, anyhow, bail};
use skia_safe::Image;
use winit::{dpi::PhysicalSize, window::Window as WinitWindow};

use crate::{
  canvas_scale::{self, CanvasScaleMode},
  draw_command,
  profile::RenderTimings,
  skia_utils,
};

pub struct SoftwareRenderer {
  window_buffer: softbuffer::Surface<Arc<WinitWindow>, Arc<WinitWindow>>,
  skia: skia_safe::Surface,
  canvas_pixels: Vec<u32>,
  image_cache: std::collections::HashMap<String, Image>,
  /// Size of `skia_surface` and `frame`; follows the canvas, not the window.
  canvas_size: (usize, usize),
  /// Size of `softbuffer_surface`; follows the window.
  window_size: (usize, usize),
}

impl SoftwareRenderer {
  pub fn new(window: Arc<WinitWindow>, size: PhysicalSize<u32>) -> Result<Self> {
    let context = softbuffer::Context::new(window.clone())
      .map_err(|err| anyhow!("failed to create softbuffer context: {err:?}"))?;

    let mut window_buffer_surface = softbuffer::Surface::new(&context, window)
      .map_err(|err| anyhow!("failed to create softbuffer surface: {err:?}"))?;

    // Provisional sizes; `render` resizes both surfaces to the actual canvas/window sizes.
    let width = size.width.max(1) as usize;
    let height = size.height.max(1) as usize;

    resize_window_buffer(&mut window_buffer_surface, width, height)?;

    Ok(Self {
      window_buffer: window_buffer_surface,
      skia: skia_utils::create_surface(width, height)?,
      canvas_pixels: vec![0_u32; width * height],
      image_cache: std::collections::HashMap::new(),
      canvas_size: (width, height),
      window_size: (width, height),
    })
  }

  /// Keeps the Skia surface sized to the canvas and the softbuffer surface sized to the window.
  fn ensure_sizes(
    &mut self,
    canvas_size: (usize, usize),
    window_size: (usize, usize),
  ) -> Result<()> {
    if self.window_size != window_size {
      resize_window_buffer(&mut self.window_buffer, window_size.0, window_size.1)?;
      self.window_size = window_size;
    }

    if self.canvas_size != canvas_size {
      self.skia = skia_utils::create_surface(canvas_size.0, canvas_size.1)?;
      self.canvas_pixels.resize(canvas_size.0 * canvas_size.1, 0);
      self.canvas_size = canvas_size;
    }

    Ok(())
  }

  pub fn render(
    &mut self,
    scale_mode: CanvasScaleMode,
    commands: &[draw_command::DrawCommand],
    canvas_width: usize,
    canvas_height: usize,
    window_width: usize,
    window_height: usize,
  ) -> Result<RenderTimings> {
    self.ensure_sizes((canvas_width, canvas_height), (window_width, window_height))?;

    let render_started = Instant::now();
    if !draw_command::render_software_sprite_batch(&mut self.skia, commands, &mut self.image_cache)?
    {
      draw_command::render_commands(&mut self.skia, commands, &mut self.image_cache)?;
    }
    let render = render_started.elapsed();

    let present_started = Instant::now();
    let mut window_buffer = self
      .window_buffer
      .buffer_mut()
      .map_err(|err| anyhow!("failed to get frame buffer: {err:?}"))?;

    let readback = if canvas_width == window_width && canvas_height == window_height {
      let readback_started = Instant::now();
      copy_skia_to_window_buffer(
        &mut self.skia,
        &mut window_buffer,
        canvas_width,
        canvas_height,
      )?;
      readback_started.elapsed()
    } else {
      let readback_started = Instant::now();
      copy_skia_to_window_buffer(
        &mut self.skia,
        &mut self.canvas_pixels,
        canvas_width,
        canvas_height,
      )?;
      let elapsed = readback_started.elapsed();
      present_frame_for_scale_mode(
        scale_mode,
        &mut window_buffer,
        &self.canvas_pixels,
        canvas_width,
        canvas_height,
        window_width,
        window_height,
      );
      elapsed
    };

    window_buffer
      .present()
      .map_err(|err| anyhow!("failed to present frame: {err:?}"))?;

    let present = present_started.elapsed().saturating_sub(readback);

    Ok(RenderTimings {
      render,
      readback,
      present,
    })
  }
}

fn resize_window_buffer(
  dst_window_buffer: &mut softbuffer::Surface<Arc<WinitWindow>, Arc<WinitWindow>>,
  width: usize,
  height: usize,
) -> Result<()> {
  let width = u32::try_from(width.max(1))
    .ok()
    .and_then(std::num::NonZeroU32::new)
    .context("window width was out of range")?;
  let height = u32::try_from(height.max(1))
    .ok()
    .and_then(std::num::NonZeroU32::new)
    .context("window height was out of range")?;

  dst_window_buffer
    .resize(width, height)
    .map_err(|err| anyhow!("failed to resize softbuffer surface: {err:?}"))
}

fn present_frame_for_scale_mode(
  scale_mode: CanvasScaleMode,
  dst_window_buffer: &mut [u32],
  canvas_pixels: &[u32],
  canvas_width: usize,
  canvas_height: usize,
  window_width: usize,
  window_height: usize,
) {
  match scale_mode {
    CanvasScaleMode::Responsive => {
      let len = (canvas_width * canvas_height)
        .min(dst_window_buffer.len())
        .min(canvas_pixels.len());

      dst_window_buffer[..len].copy_from_slice(&canvas_pixels[..len]);
    }
    CanvasScaleMode::ExactFit => {
      canvas_scale::blit_exact_fit(
        canvas_pixels,
        canvas_width,
        canvas_height,
        dst_window_buffer,
        window_width,
        window_height,
      );
    }
    CanvasScaleMode::ShowAll => {
      canvas_scale::blit_aspect_fit(
        canvas_pixels,
        canvas_width,
        canvas_height,
        dst_window_buffer,
        window_width,
        window_height,
      );
    }
    CanvasScaleMode::NoBorder => {
      canvas_scale::blit_aspect_fill(
        canvas_pixels,
        canvas_width,
        canvas_height,
        dst_window_buffer,
        window_width,
        window_height,
      );
    }
    CanvasScaleMode::NoScale => {
      canvas_scale::blit_center(
        canvas_pixels,
        canvas_width,
        canvas_height,
        dst_window_buffer,
        window_width,
        window_height,
      );
    }
  }
}

fn copy_skia_to_window_buffer(
  skia: &mut skia_safe::Surface,
  dst_window_buffer: &mut [u32],
  dst_width: usize,
  dst_height: usize,
) -> Result<()> {
  let image_info = skia_safe::ImageInfo::new(
    (dst_width as i32, dst_height as i32),
    skia_safe::ColorType::BGRA8888,
    skia_safe::AlphaType::Premul,
    None,
  );

  let pixel_count = dst_width * dst_height;
  if dst_window_buffer.len() < pixel_count {
    bail!("frame buffer is smaller than the Skia surface");
  }

  // Skia writes bytes, while the window presenter consumes the same backing buffer as u32 pixels.
  let frame_bytes: &mut [u8] = bytemuck::cast_slice_mut(&mut dst_window_buffer[..pixel_count]);

  if !skia.read_pixels(&image_info, frame_bytes, dst_width * 4, (0, 0)) {
    bail!("failed to read pixels from Skia surface");
  }

  Ok(())
}
