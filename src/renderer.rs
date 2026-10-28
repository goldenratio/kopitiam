use std::sync::Arc;

use anyhow::Result;
use winit::{dpi::PhysicalSize, window::Window as WinitWindow};

mod software_renderer;
use crate::{
  canvas_scale::CanvasScaleMode, draw_command, profile::RenderTimings,
  renderer::software_renderer::SoftwareRenderer,
};

pub(crate) struct RendererFactory();

impl RendererFactory {
  pub(crate) fn create(window: Arc<WinitWindow>, size: PhysicalSize<u32>) -> Result<Renderer> {
    Ok(Renderer::Software(SoftwareRenderer::new(window, size)?))
  }
}

pub(crate) enum Renderer {
  Software(SoftwareRenderer),
}

impl Renderer {
  pub(crate) fn name(&self) -> &'static str {
    match self {
      Self::Software(_) => "software",
    }
  }

  pub(crate) fn render(
    &mut self,
    scale_mode: CanvasScaleMode,
    commands: &[draw_command::DrawCommand],
    canvas_width: usize,
    canvas_height: usize,
    window_width: usize,
    window_height: usize,
  ) -> Result<RenderTimings> {
    match self {
      Self::Software(renderer) => renderer.render(
        scale_mode,
        commands,
        canvas_width,
        canvas_height,
        window_width,
        window_height,
      ),
    }
  }
}
