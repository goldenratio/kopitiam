use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasScaleMode {
  Responsive,
  ExactFit,
  NoBorder,
  NoScale,
  ShowAll,
}

impl CanvasScaleMode {
  pub fn from_config(value: Option<&str>) -> Result<Self> {
    let Some(value) = value else {
      return Ok(Self::ShowAll);
    };

    match value {
      "responsive" => Ok(Self::Responsive),
      "exactFit" => Ok(Self::ExactFit),
      "noBorder" => Ok(Self::NoBorder),
      "noScale" => Ok(Self::NoScale),
      "showAll" => Ok(Self::ShowAll),
      _ => {
        bail!(
          "unsupported scaleMode `{value}`; expected responsive, exactFit, noBorder, noScale, or showAll"
        )
      }
    }
  }

  pub fn resizes_canvas(self) -> bool {
    self == Self::Responsive
  }
}

pub fn blit_aspect_fit(
  source: &[u32],
  source_width: usize,
  source_height: usize,
  destination: &mut [u32],
  destination_width: usize,
  destination_height: usize,
) {
  let scale = (destination_width as f32 / source_width as f32)
    .min(destination_height as f32 / source_height as f32);
  let scaled_width = source_width as f32 * scale;
  let scaled_height = source_height as f32 * scale;
  let offset_x = (destination_width as f32 - scaled_width) * 0.5;
  let offset_y = (destination_height as f32 - scaled_height) * 0.5;

  let start_x = offset_x.ceil().max(0.0) as usize;
  let start_y = offset_y.ceil().max(0.0) as usize;
  let end_x = (offset_x + scaled_width)
    .floor()
    .min(destination_width as f32) as usize;
  let end_y = (offset_y + scaled_height)
    .floor()
    .min(destination_height as f32) as usize;

  if start_y > 0 {
    destination[..start_y * destination_width].fill(0);
  }
  if end_y < destination_height {
    destination[end_y * destination_width..].fill(0);
  }

  let inv_scale = 1.0 / scale;
  let mut x_map = Vec::with_capacity(end_x.saturating_sub(start_x));
  for x in start_x..end_x {
    let source_x = (((x as f32 - offset_x) * inv_scale).floor() as usize).min(source_width - 1);
    x_map.push(source_x);
  }

  for y in start_y..end_y {
    if start_x > 0 {
      destination[y * destination_width..y * destination_width + start_x].fill(0);
    }
    if end_x < destination_width {
      destination[y * destination_width + end_x..(y + 1) * destination_width].fill(0);
    }

    let source_y = (((y as f32 - offset_y) * inv_scale).floor() as usize).min(source_height - 1);
    let source_row = source_y * source_width;
    let dst_row = y * destination_width + start_x;

    for (i, &source_x) in x_map.iter().enumerate() {
      destination[dst_row + i] = source[source_row + source_x];
    }
  }
}

pub fn blit_aspect_fill(
  source: &[u32],
  source_width: usize,
  source_height: usize,
  destination: &mut [u32],
  destination_width: usize,
  destination_height: usize,
) {
  let scale = (destination_width as f32 / source_width as f32)
    .max(destination_height as f32 / source_height as f32);
  let scaled_width = source_width as f32 * scale;
  let scaled_height = source_height as f32 * scale;
  let offset_x = (destination_width as f32 - scaled_width) * 0.5;
  let offset_y = (destination_height as f32 - scaled_height) * 0.5;

  let inv_scale = 1.0 / scale;
  let mut x_map = Vec::with_capacity(destination_width);
  for x in 0..destination_width {
    let source_x = (((x as f32 - offset_x) * inv_scale).floor() as usize).min(source_width - 1);
    x_map.push(source_x);
  }

  for y in 0..destination_height {
    let source_y = (((y as f32 - offset_y) * inv_scale).floor() as usize).min(source_height - 1);
    let source_row = source_y * source_width;
    let destination_row = y * destination_width;

    for (x, &source_x) in x_map.iter().enumerate() {
      destination[destination_row + x] = source[source_row + source_x];
    }
  }
}

pub fn blit_exact_fit(
  source: &[u32],
  source_width: usize,
  source_height: usize,
  destination: &mut [u32],
  destination_width: usize,
  destination_height: usize,
) {
  let mut x_map = Vec::with_capacity(destination_width);
  for x in 0..destination_width {
    x_map.push((x * source_width / destination_width).min(source_width - 1));
  }

  for y in 0..destination_height {
    let source_y = (y * source_height / destination_height).min(source_height - 1);
    let source_row = source_y * source_width;
    let destination_row = y * destination_width;

    for (x, &source_x) in x_map.iter().enumerate() {
      destination[destination_row + x] = source[source_row + source_x];
    }
  }
}

pub fn blit_center(
  source: &[u32],
  source_width: usize,
  source_height: usize,
  destination: &mut [u32],
  destination_width: usize,
  destination_height: usize,
) {
  destination.fill(0);
  let copy_width = source_width.min(destination_width);
  let copy_height = source_height.min(destination_height);
  let source_x = (source_width - copy_width) / 2;
  let source_y = (source_height - copy_height) / 2;
  let destination_x = (destination_width - copy_width) / 2;
  let destination_y = (destination_height - copy_height) / 2;

  for y in 0..copy_height {
    let source_start = (source_y + y) * source_width + source_x;
    let destination_start = (destination_y + y) * destination_width + destination_x;
    destination[destination_start..destination_start + copy_width]
      .copy_from_slice(&source[source_start..source_start + copy_width]);
  }
}
