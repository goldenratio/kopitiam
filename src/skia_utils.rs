use anyhow::{Result, anyhow};
use skia_safe::surfaces;

pub fn create_surface(width: usize, height: usize) -> Result<skia_safe::Surface> {
  surfaces::raster_n32_premul((width.max(1) as i32, height.max(1) as i32))
    .ok_or_else(|| anyhow!("failed to create Skia raster surface"))
}
