pub mod draw_batch;
mod path_segement;

use std::{collections::HashMap, fs};

use anyhow::{Context, Result, anyhow, bail};
use boa_engine::{
  Context as JsContext, JsNativeError, JsResult, JsValue,
  object::{JsObject, builtins::JsArray},
};
use rayon::prelude::*;
use skia_safe::{
  AlphaType, BlendMode, Canvas, Color, Color4f, ColorType, Data, FilterMode, Image, ImageInfo,
  Matrix, MipmapMode, Paint, PathFillType, QuickReject, RSXform, Rect, SamplingOptions, Shader,
  TileMode, canvas::SrcRectConstraint, gradient, images,
};
use skia_safe::{image::CachingHint, surface::ContentChangeMode};

use crate::{
  assets_utils::{decode_data_url, resolve_asset_path},
  boa_errors,
  boa_utils::{
    array_len, get_property, optional_bool, optional_f32, optional_string, optional_usize,
    required_f32, required_f32_array, required_string, value_to_object,
  },
  draw_command::path_segement::PathSegment,
  native_bridge::NativeBridge,
  skia_utils,
};

pub(crate) fn register_natives(bridge: &mut NativeBridge) -> Result<()> {
  bridge.register_native("__engine_get_image_data", 7, native_get_image_data)
}

fn native_get_image_data(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let canvas_width = native_usize_arg(args, 0);
  let canvas_height = native_usize_arg(args, 1);
  let commands = args
    .get(2)
    .and_then(JsValue::as_object)
    .ok_or_else(|| js_type_error("getImageData() expected draw commands"))?;
  let x = native_i32_arg(args, 3, js)?;
  let y = native_i32_arg(args, 4, js)?;
  let width = native_i32_arg(args, 5, js)?;
  let height = native_i32_arg(args, 6, js)?;

  let commands = draw_commands_from_array(&commands, js)
    .map_err(|err| js_range_error(format!("getImageData() failed to read commands: {err}")))?;
  let data = read_image_data(canvas_width, canvas_height, &commands, x, y, width, height)
    .map_err(|err| js_range_error(format!("getImageData() failed: {err}")))?;

  Ok(JsValue::from(JsArray::from_iter(
    data.into_iter().map(|byte| JsValue::from(byte as i32)),
    js,
  )))
}

fn native_usize_arg(args: &[JsValue], index: usize) -> usize {
  args
    .get(index)
    .and_then(JsValue::as_number)
    .filter(|value| value.is_finite() && *value >= 0.0)
    .unwrap_or(0.0) as usize
}

fn native_i32_arg(args: &[JsValue], index: usize, js: &mut JsContext) -> JsResult<i32> {
  let value = args.get(index).cloned().unwrap_or_default().to_number(js)?;
  if !value.is_finite() || value < i32::MIN as f64 || value > i32::MAX as f64 {
    return Err(js_type_error(
      "getImageData() coordinates must be finite integers",
    ));
  }
  Ok(value.trunc() as i32)
}

fn js_type_error(message: impl Into<String>) -> boa_engine::JsError {
  JsNativeError::typ().with_message(message.into()).into()
}

fn js_range_error(message: impl Into<String>) -> boa_engine::JsError {
  JsNativeError::range().with_message(message.into()).into()
}

#[derive(Debug)]
pub(crate) enum PaintStyle {
  Color(String),
  LinearGradient {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    stops: Vec<GradientStop>,
  },
  RadialGradient {
    x0: f32,
    y0: f32,
    r0: f32,
    x1: f32,
    y1: f32,
    r1: f32,
    stops: Vec<GradientStop>,
  },
  ConicGradient {
    start_angle: f32,
    x: f32,
    y: f32,
    stops: Vec<GradientStop>,
  },
  Pattern {
    source_kind: String,
    src: String,
    repetition: String,
    width: Option<usize>,
    height: Option<usize>,
    commands: Vec<DrawCommand>,
    transform: PatternTransform,
  },
}

#[derive(Debug)]
pub(crate) struct GradientStop {
  offset: f32,
  color: String,
}

#[derive(Debug)]
pub(crate) struct PatternTransform {
  a: f32,
  b: f32,
  c: f32,
  d: f32,
  e: f32,
  f: f32,
}

#[derive(Debug)]
struct ImageSampling {
  enabled: bool,
  quality: String,
}

impl ImageSampling {
  fn new(enabled: bool, quality: impl Into<String>) -> Self {
    Self {
      enabled,
      quality: quality.into(),
    }
  }
}

impl Default for PatternTransform {
  fn default() -> Self {
    Self {
      a: 1.0,
      b: 0.0,
      c: 0.0,
      d: 1.0,
      e: 0.0,
      f: 0.0,
    }
  }
}

#[derive(Debug)]
pub(crate) enum DrawCommand {
  Exit,
  Save,
  Restore,
  Translate {
    x: f32,
    y: f32,
  },
  Rotate {
    angle: f32,
  },
  Scale {
    x: f32,
    y: f32,
  },
  Transform {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
  },
  SetTransform {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
  },
  ResetTransform,
  ClearRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
  },
  FillRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fill_style: PaintStyle,
    global_alpha: f32,
    global_composite_operation: String,
    image_smoothing_enabled: bool,
    image_smoothing_quality: String,
  },
  StrokeRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    stroke_style: PaintStyle,
    global_alpha: f32,
    global_composite_operation: String,
    image_smoothing_enabled: bool,
    image_smoothing_quality: String,
    line_width: f32,
    line_cap: String,
    line_join: String,
    miter_limit: f32,
  },
  FillPath {
    path: Vec<PathSegment>,
    fill_style: PaintStyle,
    global_alpha: f32,
    global_composite_operation: String,
    image_smoothing_enabled: bool,
    image_smoothing_quality: String,
    rule: String,
  },
  FillCircle {
    x: f32,
    y: f32,
    radius: f32,
    fill_style: PaintStyle,
    global_alpha: f32,
    global_composite_operation: String,
    image_smoothing_enabled: bool,
    image_smoothing_quality: String,
  },
  StrokeLine {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    stroke_style: PaintStyle,
    global_alpha: f32,
    global_composite_operation: String,
    image_smoothing_enabled: bool,
    image_smoothing_quality: String,
    line_width: f32,
    line_cap: String,
    line_join: String,
    miter_limit: f32,
  },
  ClipPath {
    path: Vec<PathSegment>,
    rule: String,
  },
  StrokePath {
    path: Vec<PathSegment>,
    stroke_style: PaintStyle,
    global_alpha: f32,
    global_composite_operation: String,
    image_smoothing_enabled: bool,
    image_smoothing_quality: String,
    line_width: f32,
    line_cap: String,
    line_join: String,
    miter_limit: f32,
  },
  CacheCanvasBitmap {
    src: String,
    width: Option<usize>,
    height: Option<usize>,
    commands: Vec<DrawCommand>,
  },
  DrawImage {
    source_kind: String,
    src: String,
    dx: f32,
    dy: f32,
    dw: Option<f32>,
    dh: Option<f32>,
    sx: Option<f32>,
    sy: Option<f32>,
    sw: Option<f32>,
    sh: Option<f32>,
    global_alpha: f32,
    global_composite_operation: String,
    image_smoothing_enabled: bool,
    image_smoothing_quality: String,
    bitmap_sx: Option<f32>,
    bitmap_sy: Option<f32>,
    bitmap_sw: Option<f32>,
    bitmap_sh: Option<f32>,
    width: Option<usize>,
    height: Option<usize>,
    commands: Vec<DrawCommand>,
  },
  DrawImageBatch {
    source_kind: String,
    src: String,
    global_alpha: f32,
    global_composite_operation: String,
    image_smoothing_enabled: bool,
    image_smoothing_quality: String,
    bitmap_sx: Option<f32>,
    bitmap_sy: Option<f32>,
    bitmap_sw: Option<f32>,
    bitmap_sh: Option<f32>,
    width: Option<usize>,
    height: Option<usize>,
    draws: Vec<f32>,
  },
  PutImageData {
    data: Vec<u8>,
    width: usize,
    height: usize,
    dx: f32,
    dy: f32,
    dirty_x: Option<f32>,
    dirty_y: Option<f32>,
    dirty_width: Option<f32>,
    dirty_height: Option<f32>,
  },
}

fn default_alpha() -> f32 {
  1.0
}

fn default_line_width() -> f32 {
  1.0
}

fn default_line_cap() -> String {
  "butt".to_owned()
}

fn default_line_join() -> String {
  "miter".to_owned()
}

fn default_miter_limit() -> f32 {
  10.0
}

fn default_fill_rule() -> String {
  "nonzero".to_owned()
}

fn default_composite_operation() -> String {
  "source-over".to_owned()
}

fn default_image_source_kind() -> String {
  "image".to_owned()
}

fn default_image_smoothing_enabled() -> bool {
  true
}

fn default_image_smoothing_quality() -> String {
  "low".to_owned()
}

pub(crate) fn contains_exit_command(commands: &[DrawCommand]) -> bool {
  commands
    .iter()
    .any(|command| matches!(command, DrawCommand::Exit))
}

pub(crate) fn draw_commands_from_array(
  array: &JsObject,
  js: &mut JsContext,
) -> Result<Vec<DrawCommand>> {
  let len = array_len(array, js)?;
  let mut commands = Vec::with_capacity(len);

  for index in 0..len {
    let value = array.get(index as u32, js).map_err(boa_errors::js_error)?;
    if value.is_null_or_undefined() {
      continue;
    }
    commands.push(draw_command_from_value(value, js)?);
  }

  Ok(commands)
}

pub(crate) fn optional_commands(
  object: &JsObject,
  key: &'static str,
  js: &mut JsContext,
) -> Result<Option<Vec<DrawCommand>>> {
  let value = get_property(object, key, js)?;
  if value.is_null_or_undefined() {
    return Ok(None);
  }
  let array = value_to_object(value, key)?;
  draw_commands_from_array(&array, js).map(Some)
}

pub(crate) fn batch_draw_count(commands: &[DrawCommand]) -> usize {
  commands
    .iter()
    .map(|command| match command {
      DrawCommand::DrawImageBatch { draws, .. } => draws.len() / 4,
      _ => 0,
    })
    .sum()
}

fn required_draw_batch(object: &JsObject, js: &mut JsContext) -> Result<Vec<f32>> {
  if let Some(id) = optional_usize(object, "drawBatchId", js)? {
    return draw_batch::take_draw_batch(id).ok_or_else(|| anyhow!("invalid draw batch id `{id}`"));
  }

  required_f32_array(object, "draws", js)
}

fn required_u8_array(object: &JsObject, key: &'static str, js: &mut JsContext) -> Result<Vec<u8>> {
  let value = get_property(object, key, js)?;
  let array = value_to_object(value, key)?;
  let len = array_len(&array, js)?;
  let mut values = Vec::with_capacity(len);

  for index in 0..len {
    let value = array.get(index as u32, js).map_err(boa_errors::js_error)?;
    let number = value.to_number(js).map_err(boa_errors::js_error)?;
    values.push(number.clamp(0.0, 255.0) as u8);
  }

  Ok(values)
}

fn paint_style_property(
  object: &JsObject,
  key: &'static str,
  js: &mut JsContext,
) -> Result<PaintStyle> {
  let value = get_property(object, key, js)?;
  paint_style_from_value(value, key, js)
}

fn paint_style_from_value(
  value: JsValue,
  key: &'static str,
  js: &mut JsContext,
) -> Result<PaintStyle> {
  if value.is_null_or_undefined() {
    return Ok(PaintStyle::Color("#000000".to_owned()));
  }

  if let Some(text) = value.as_string() {
    return Ok(PaintStyle::Color(text.to_std_string_escaped()));
  }

  let object = value_to_object(value, key)?;
  let style_type = required_string(&object, "type", js)?;
  match style_type.as_str() {
    "linearGradient" => Ok(PaintStyle::LinearGradient {
      x0: required_f32(&object, "x0", js)?,
      y0: required_f32(&object, "y0", js)?,
      x1: required_f32(&object, "x1", js)?,
      y1: required_f32(&object, "y1", js)?,
      stops: gradient_stops(&object, js)?,
    }),
    "radialGradient" => Ok(PaintStyle::RadialGradient {
      x0: required_f32(&object, "x0", js)?,
      y0: required_f32(&object, "y0", js)?,
      r0: required_f32(&object, "r0", js)?,
      x1: required_f32(&object, "x1", js)?,
      y1: required_f32(&object, "y1", js)?,
      r1: required_f32(&object, "r1", js)?,
      stops: gradient_stops(&object, js)?,
    }),
    "conicGradient" => Ok(PaintStyle::ConicGradient {
      start_angle: required_f32(&object, "startAngle", js)?,
      x: required_f32(&object, "x0", js)?,
      y: required_f32(&object, "y0", js)?,
      stops: gradient_stops(&object, js)?,
    }),
    "pattern" => Ok(PaintStyle::Pattern {
      source_kind: optional_string(&object, "sourceKind", js)?
        .unwrap_or_else(default_image_source_kind),
      src: required_string(&object, "src", js)?,
      repetition: optional_string(&object, "repetition", js)?
        .unwrap_or_else(|| "repeat".to_owned()),
      width: optional_usize(&object, "width", js)?,
      height: optional_usize(&object, "height", js)?,
      commands: optional_commands(&object, "commands", js)?.unwrap_or_default(),
      transform: pattern_transform(&object, js)?,
    }),
    _ => bail!("unsupported paint style `{style_type}`"),
  }
}

fn gradient_stops(object: &JsObject, js: &mut JsContext) -> Result<Vec<GradientStop>> {
  let stops_value = get_property(object, "stops", js)?;
  let stops_array = value_to_object(stops_value, "stops")?;
  let len = array_len(&stops_array, js)?;
  let mut stops = Vec::with_capacity(len);

  for index in 0..len {
    let stop_value = stops_array
      .get(index as u32, js)
      .map_err(boa_errors::js_error)?;
    if stop_value.is_null_or_undefined() {
      continue;
    }
    let stop = value_to_object(stop_value, "gradient stop")?;
    stops.push(GradientStop {
      offset: required_f32(&stop, "offset", js)?.clamp(0.0, 1.0),
      color: required_string(&stop, "color", js)?,
    });
  }

  Ok(stops)
}

fn pattern_transform(object: &JsObject, js: &mut JsContext) -> Result<PatternTransform> {
  let value = get_property(object, "transform", js)?;
  if value.is_null_or_undefined() {
    return Ok(PatternTransform::default());
  }

  let transform = value_to_object(value, "transform")?;
  Ok(PatternTransform {
    a: optional_f32(&transform, "a", js)?.unwrap_or(1.0),
    b: optional_f32(&transform, "b", js)?.unwrap_or(0.0),
    c: optional_f32(&transform, "c", js)?.unwrap_or(0.0),
    d: optional_f32(&transform, "d", js)?.unwrap_or(1.0),
    e: optional_f32(&transform, "e", js)?.unwrap_or(0.0),
    f: optional_f32(&transform, "f", js)?.unwrap_or(0.0),
  })
}

fn draw_command_from_value(value: JsValue, js: &mut JsContext) -> Result<DrawCommand> {
  let object = value_to_object(value, "draw command")?;
  let op = required_string(&object, "op", js)?;

  match op.as_str() {
    "exit" => Ok(DrawCommand::Exit),
    "save" => Ok(DrawCommand::Save),
    "restore" => Ok(DrawCommand::Restore),
    "translate" => Ok(DrawCommand::Translate {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
    }),
    "rotate" => Ok(DrawCommand::Rotate {
      angle: required_f32(&object, "angle", js)?,
    }),
    "scale" => Ok(DrawCommand::Scale {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
    }),
    "transform" => Ok(DrawCommand::Transform {
      a: required_f32(&object, "a", js)?,
      b: required_f32(&object, "b", js)?,
      c: required_f32(&object, "c", js)?,
      d: required_f32(&object, "d", js)?,
      e: required_f32(&object, "e", js)?,
      f: required_f32(&object, "f", js)?,
    }),
    "setTransform" => Ok(DrawCommand::SetTransform {
      a: required_f32(&object, "a", js)?,
      b: required_f32(&object, "b", js)?,
      c: required_f32(&object, "c", js)?,
      d: required_f32(&object, "d", js)?,
      e: required_f32(&object, "e", js)?,
      f: required_f32(&object, "f", js)?,
    }),
    "resetTransform" => Ok(DrawCommand::ResetTransform),
    "clearRect" => Ok(DrawCommand::ClearRect {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
      width: required_f32(&object, "width", js)?,
      height: required_f32(&object, "height", js)?,
    }),
    "fillRect" => Ok(DrawCommand::FillRect {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
      width: required_f32(&object, "width", js)?,
      height: required_f32(&object, "height", js)?,
      fill_style: paint_style_property(&object, "fillStyle", js)?,
      global_alpha: optional_f32(&object, "globalAlpha", js)?.unwrap_or_else(default_alpha),
      global_composite_operation: optional_string(&object, "globalCompositeOperation", js)?
        .unwrap_or_else(default_composite_operation),
      image_smoothing_enabled: optional_bool(&object, "imageSmoothingEnabled", js)?
        .unwrap_or_else(default_image_smoothing_enabled),
      image_smoothing_quality: optional_string(&object, "imageSmoothingQuality", js)?
        .unwrap_or_else(default_image_smoothing_quality),
    }),
    "strokeRect" => Ok(DrawCommand::StrokeRect {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
      width: required_f32(&object, "width", js)?,
      height: required_f32(&object, "height", js)?,
      stroke_style: paint_style_property(&object, "strokeStyle", js)?,
      global_alpha: optional_f32(&object, "globalAlpha", js)?.unwrap_or_else(default_alpha),
      global_composite_operation: optional_string(&object, "globalCompositeOperation", js)?
        .unwrap_or_else(default_composite_operation),
      image_smoothing_enabled: optional_bool(&object, "imageSmoothingEnabled", js)?
        .unwrap_or_else(default_image_smoothing_enabled),
      image_smoothing_quality: optional_string(&object, "imageSmoothingQuality", js)?
        .unwrap_or_else(default_image_smoothing_quality),
      line_width: optional_f32(&object, "lineWidth", js)?.unwrap_or_else(default_line_width),
      line_cap: optional_string(&object, "lineCap", js)?.unwrap_or_else(default_line_cap),
      line_join: optional_string(&object, "lineJoin", js)?.unwrap_or_else(default_line_join),
      miter_limit: optional_f32(&object, "miterLimit", js)?.unwrap_or_else(default_miter_limit),
    }),
    "fillPath" => Ok(DrawCommand::FillPath {
      path: path_segement::required_path_segments(&object, "path", js)?,
      fill_style: paint_style_property(&object, "fillStyle", js)?,
      global_alpha: optional_f32(&object, "globalAlpha", js)?.unwrap_or_else(default_alpha),
      global_composite_operation: optional_string(&object, "globalCompositeOperation", js)?
        .unwrap_or_else(default_composite_operation),
      image_smoothing_enabled: optional_bool(&object, "imageSmoothingEnabled", js)?
        .unwrap_or_else(default_image_smoothing_enabled),
      image_smoothing_quality: optional_string(&object, "imageSmoothingQuality", js)?
        .unwrap_or_else(default_image_smoothing_quality),
      rule: optional_string(&object, "rule", js)?.unwrap_or_else(default_fill_rule),
    }),
    "clipPath" => Ok(DrawCommand::ClipPath {
      path: path_segement::required_path_segments(&object, "path", js)?,
      rule: optional_string(&object, "rule", js)?.unwrap_or_else(default_fill_rule),
    }),
    "strokePath" => Ok(DrawCommand::StrokePath {
      path: path_segement::required_path_segments(&object, "path", js)?,
      stroke_style: paint_style_property(&object, "strokeStyle", js)?,
      global_alpha: optional_f32(&object, "globalAlpha", js)?.unwrap_or_else(default_alpha),
      global_composite_operation: optional_string(&object, "globalCompositeOperation", js)?
        .unwrap_or_else(default_composite_operation),
      image_smoothing_enabled: optional_bool(&object, "imageSmoothingEnabled", js)?
        .unwrap_or_else(default_image_smoothing_enabled),
      image_smoothing_quality: optional_string(&object, "imageSmoothingQuality", js)?
        .unwrap_or_else(default_image_smoothing_quality),
      line_width: optional_f32(&object, "lineWidth", js)?.unwrap_or_else(default_line_width),
      line_cap: optional_string(&object, "lineCap", js)?.unwrap_or_else(default_line_cap),
      line_join: optional_string(&object, "lineJoin", js)?.unwrap_or_else(default_line_join),
      miter_limit: optional_f32(&object, "miterLimit", js)?.unwrap_or_else(default_miter_limit),
    }),
    "fillCircle" => Ok(DrawCommand::FillCircle {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
      radius: required_f32(&object, "radius", js)?,
      fill_style: paint_style_property(&object, "fillStyle", js)?,
      global_alpha: optional_f32(&object, "globalAlpha", js)?.unwrap_or_else(default_alpha),
      global_composite_operation: optional_string(&object, "globalCompositeOperation", js)?
        .unwrap_or_else(default_composite_operation),
      image_smoothing_enabled: optional_bool(&object, "imageSmoothingEnabled", js)?
        .unwrap_or_else(default_image_smoothing_enabled),
      image_smoothing_quality: optional_string(&object, "imageSmoothingQuality", js)?
        .unwrap_or_else(default_image_smoothing_quality),
    }),
    "strokeLine" => Ok(DrawCommand::StrokeLine {
      x0: required_f32(&object, "x0", js)?,
      y0: required_f32(&object, "y0", js)?,
      x1: required_f32(&object, "x1", js)?,
      y1: required_f32(&object, "y1", js)?,
      stroke_style: paint_style_property(&object, "strokeStyle", js)?,
      global_alpha: optional_f32(&object, "globalAlpha", js)?.unwrap_or_else(default_alpha),
      global_composite_operation: optional_string(&object, "globalCompositeOperation", js)?
        .unwrap_or_else(default_composite_operation),
      image_smoothing_enabled: optional_bool(&object, "imageSmoothingEnabled", js)?
        .unwrap_or_else(default_image_smoothing_enabled),
      image_smoothing_quality: optional_string(&object, "imageSmoothingQuality", js)?
        .unwrap_or_else(default_image_smoothing_quality),
      line_width: optional_f32(&object, "lineWidth", js)?.unwrap_or_else(default_line_width),
      line_cap: optional_string(&object, "lineCap", js)?.unwrap_or_else(default_line_cap),
      line_join: optional_string(&object, "lineJoin", js)?.unwrap_or_else(default_line_join),
      miter_limit: optional_f32(&object, "miterLimit", js)?.unwrap_or_else(default_miter_limit),
    }),
    "cacheCanvasBitmap" => Ok(DrawCommand::CacheCanvasBitmap {
      src: required_string(&object, "src", js)?,
      width: optional_usize(&object, "width", js)?,
      height: optional_usize(&object, "height", js)?,
      commands: optional_commands(&object, "commands", js)?.unwrap_or_default(),
    }),
    "drawImage" => Ok(DrawCommand::DrawImage {
      source_kind: optional_string(&object, "sourceKind", js)?
        .unwrap_or_else(default_image_source_kind),
      src: required_string(&object, "src", js)?,
      dx: required_f32(&object, "dx", js)?,
      dy: required_f32(&object, "dy", js)?,
      dw: optional_f32(&object, "dw", js)?,
      dh: optional_f32(&object, "dh", js)?,
      sx: optional_f32(&object, "sx", js)?,
      sy: optional_f32(&object, "sy", js)?,
      sw: optional_f32(&object, "sw", js)?,
      sh: optional_f32(&object, "sh", js)?,
      global_alpha: optional_f32(&object, "globalAlpha", js)?.unwrap_or_else(default_alpha),
      global_composite_operation: optional_string(&object, "globalCompositeOperation", js)?
        .unwrap_or_else(default_composite_operation),
      image_smoothing_enabled: optional_bool(&object, "imageSmoothingEnabled", js)?
        .unwrap_or_else(default_image_smoothing_enabled),
      image_smoothing_quality: optional_string(&object, "imageSmoothingQuality", js)?
        .unwrap_or_else(default_image_smoothing_quality),
      bitmap_sx: optional_f32(&object, "bitmapSx", js)?,
      bitmap_sy: optional_f32(&object, "bitmapSy", js)?,
      bitmap_sw: optional_f32(&object, "bitmapSw", js)?,
      bitmap_sh: optional_f32(&object, "bitmapSh", js)?,
      width: optional_usize(&object, "width", js)?,
      height: optional_usize(&object, "height", js)?,
      commands: optional_commands(&object, "commands", js)?.unwrap_or_default(),
    }),
    "drawImageBatch" => Ok(DrawCommand::DrawImageBatch {
      source_kind: optional_string(&object, "sourceKind", js)?
        .unwrap_or_else(default_image_source_kind),
      src: required_string(&object, "src", js)?,
      global_alpha: optional_f32(&object, "globalAlpha", js)?.unwrap_or_else(default_alpha),
      global_composite_operation: optional_string(&object, "globalCompositeOperation", js)?
        .unwrap_or_else(default_composite_operation),
      image_smoothing_enabled: optional_bool(&object, "imageSmoothingEnabled", js)?
        .unwrap_or_else(default_image_smoothing_enabled),
      image_smoothing_quality: optional_string(&object, "imageSmoothingQuality", js)?
        .unwrap_or_else(default_image_smoothing_quality),
      bitmap_sx: optional_f32(&object, "bitmapSx", js)?,
      bitmap_sy: optional_f32(&object, "bitmapSy", js)?,
      bitmap_sw: optional_f32(&object, "bitmapSw", js)?,
      bitmap_sh: optional_f32(&object, "bitmapSh", js)?,
      width: optional_usize(&object, "width", js)?,
      height: optional_usize(&object, "height", js)?,
      draws: required_draw_batch(&object, js)?,
    }),
    "putImageData" => Ok(DrawCommand::PutImageData {
      data: required_u8_array(&object, "data", js)?,
      width: optional_usize(&object, "width", js)?.unwrap_or(0),
      height: optional_usize(&object, "height", js)?.unwrap_or(0),
      dx: required_f32(&object, "dx", js)?,
      dy: required_f32(&object, "dy", js)?,
      dirty_x: optional_f32(&object, "dirtyX", js)?,
      dirty_y: optional_f32(&object, "dirtyY", js)?,
      dirty_width: optional_f32(&object, "dirtyWidth", js)?,
      dirty_height: optional_f32(&object, "dirtyHeight", js)?,
    }),
    _ => bail!("unsupported draw command `{op}`"),
  }
}

fn parse_color(input: &str, global_alpha: f32) -> Result<Color> {
  let alpha_factor = global_alpha.clamp(0.0, 1.0);
  let named = match input.to_ascii_lowercase().as_str() {
    "black" => Some("#000000"),
    "white" => Some("#ffffff"),
    "red" => Some("#ff0000"),
    "green" => Some("#008000"),
    "blue" => Some("#0000ff"),
    "yellow" => Some("#ffff00"),
    "cyan" => Some("#00ffff"),
    "magenta" => Some("#ff00ff"),
    "transparent" => Some("#00000000"),
    _ => None,
  };
  let input = named.unwrap_or(input);
  let hex = input.strip_prefix('#').ok_or_else(|| {
    anyhow!("color `{input}` must use a basic name, #RGB, #RGBA, #RRGGBB, or #RRGGBBAA")
  })?;

  match hex.len() {
    3 => {
      let rgb = u32::from_str_radix(hex, 16).with_context(|| format!("invalid color `{input}`"))?;
      Ok(Color::from_argb(
        (255.0 * alpha_factor).round() as u8,
        (((rgb >> 8) & 0xf) * 17) as u8,
        (((rgb >> 4) & 0xf) * 17) as u8,
        ((rgb & 0xf) * 17) as u8,
      ))
    }
    4 => {
      let rgba =
        u32::from_str_radix(hex, 16).with_context(|| format!("invalid color `{input}`"))?;
      let alpha = ((rgba & 0xf) * 17) as f32 * alpha_factor;
      Ok(Color::from_argb(
        alpha.round() as u8,
        (((rgba >> 12) & 0xf) * 17) as u8,
        (((rgba >> 8) & 0xf) * 17) as u8,
        (((rgba >> 4) & 0xf) * 17) as u8,
      ))
    }
    6 => {
      let rgb = u32::from_str_radix(hex, 16).with_context(|| format!("invalid color `{input}`"))?;
      Ok(Color::from_argb(
        (255.0 * alpha_factor).round() as u8,
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
      ))
    }
    8 => {
      let rgba =
        u32::from_str_radix(hex, 16).with_context(|| format!("invalid color `{input}`"))?;
      let alpha = (rgba & 0xff) as f32 * alpha_factor;
      Ok(Color::from_argb(
        alpha.round() as u8,
        ((rgba >> 24) & 0xff) as u8,
        ((rgba >> 16) & 0xff) as u8,
        ((rgba >> 8) & 0xff) as u8,
      ))
    }
    _ => bail!("color `{input}` must use a basic name, #RGB, #RGBA, #RRGGBB, or #RRGGBBAA"),
  }
}

fn blend_mode(composite_operation: &str) -> BlendMode {
  match composite_operation {
    "source-over" => BlendMode::SrcOver,
    "source-in" => BlendMode::SrcIn,
    "source-out" => BlendMode::SrcOut,
    "source-atop" => BlendMode::SrcATop,
    "destination-over" => BlendMode::DstOver,
    "destination-in" => BlendMode::DstIn,
    "destination-out" => BlendMode::DstOut,
    "destination-atop" => BlendMode::DstATop,
    "lighter" => BlendMode::Plus,
    "copy" => BlendMode::Src,
    "xor" => BlendMode::Xor,
    "multiply" => BlendMode::Multiply,
    "screen" => BlendMode::Screen,
    "overlay" => BlendMode::Overlay,
    "darken" => BlendMode::Darken,
    "lighten" => BlendMode::Lighten,
    "color-dodge" => BlendMode::ColorDodge,
    "color-burn" => BlendMode::ColorBurn,
    "hard-light" => BlendMode::HardLight,
    "soft-light" => BlendMode::SoftLight,
    "difference" => BlendMode::Difference,
    "exclusion" => BlendMode::Exclusion,
    "hue" => BlendMode::Hue,
    "saturation" => BlendMode::Saturation,
    "color" => BlendMode::Color,
    "luminosity" => BlendMode::Luminosity,
    _ => BlendMode::SrcOver,
  }
}

fn fill_paint(
  style: &PaintStyle,
  global_alpha: f32,
  composite_operation: &str,
  image_sampling: &ImageSampling,
  image_cache: &mut HashMap<String, Image>,
) -> Result<Paint> {
  let mut paint = Paint::default();
  paint.set_anti_alias(true);
  apply_paint_style(&mut paint, style, global_alpha, image_sampling, image_cache)?;
  paint.set_blend_mode(blend_mode(composite_operation));
  Ok(paint)
}

#[allow(clippy::too_many_arguments)]
fn stroke_paint(
  style: &PaintStyle,
  global_alpha: f32,
  composite_operation: &str,
  line_width: f32,
  line_cap: &str,
  line_join: &str,
  miter_limit: f32,
  image_sampling: &ImageSampling,
  image_cache: &mut HashMap<String, Image>,
) -> Result<Paint> {
  let mut paint = fill_paint(
    style,
    global_alpha,
    composite_operation,
    image_sampling,
    image_cache,
  )?;
  paint.set_style(skia_safe::paint::Style::Stroke);
  paint.set_stroke_width(line_width.max(0.0));
  paint.set_stroke_cap(match line_cap {
    "round" => skia_safe::paint::Cap::Round,
    "square" => skia_safe::paint::Cap::Square,
    _ => skia_safe::paint::Cap::Butt,
  });
  paint.set_stroke_join(match line_join {
    "round" => skia_safe::paint::Join::Round,
    "bevel" => skia_safe::paint::Join::Bevel,
    _ => skia_safe::paint::Join::Miter,
  });
  paint.set_stroke_miter(miter_limit.max(0.0));
  Ok(paint)
}

fn apply_paint_style(
  paint: &mut Paint,
  style: &PaintStyle,
  global_alpha: f32,
  image_sampling: &ImageSampling,
  image_cache: &mut HashMap<String, Image>,
) -> Result<()> {
  match style {
    PaintStyle::Color(color) => {
      paint.set_color(parse_color(color, global_alpha)?);
    }
    PaintStyle::LinearGradient {
      x0,
      y0,
      x1,
      y1,
      stops,
    } => {
      if let Some(shader) = gradient_shader_from_stops(stops, global_alpha, |colors, positions| {
        let colors = gradient::Colors::new(colors, positions, TileMode::Clamp, None);
        let gradient = gradient::Gradient::new(colors, gradient::Interpolation::default());
        gradient::shaders::linear_gradient(((*x0, *y0), (*x1, *y1)), &gradient, None)
      })? {
        paint.set_shader(shader);
      }
    }
    PaintStyle::RadialGradient {
      x0,
      y0,
      r0,
      x1,
      y1,
      r1,
      stops,
    } => {
      if let Some(shader) = gradient_shader_from_stops(stops, global_alpha, |colors, positions| {
        let colors = gradient::Colors::new(colors, positions, TileMode::Clamp, None);
        let gradient = gradient::Gradient::new(colors, gradient::Interpolation::default());
        gradient::shaders::two_point_conical_gradient(
          ((*x0, *y0), *r0),
          ((*x1, *y1), *r1),
          &gradient,
          None,
        )
      })? {
        paint.set_shader(shader);
      }
    }
    PaintStyle::ConicGradient {
      start_angle,
      x,
      y,
      stops,
    } => {
      let start = start_angle.to_degrees();
      if let Some(shader) = gradient_shader_from_stops(stops, global_alpha, |colors, positions| {
        let colors = gradient::Colors::new(colors, positions, TileMode::Clamp, None);
        let gradient = gradient::Gradient::new(colors, gradient::Interpolation::default());
        gradient::shaders::sweep_gradient((*x, *y), (start, start + 360.0), &gradient, None)
      })? {
        paint.set_shader(shader);
      }
    }
    PaintStyle::Pattern {
      source_kind,
      src,
      repetition,
      width,
      height,
      commands,
      transform,
    } => {
      let image = pattern_image(source_kind, src, *width, *height, commands, image_cache)?;
      let tile_modes = pattern_tile_modes(repetition);
      let matrix = matrix_from_transform(
        transform.a,
        transform.b,
        transform.c,
        transform.d,
        transform.e,
        transform.f,
      );
      let shader = image
        .to_shader(
          tile_modes,
          skia_image_sampling(image_sampling),
          Some(&matrix),
        )
        .ok_or_else(|| anyhow!("failed to create image pattern shader"))?;
      paint.set_alpha_f(global_alpha.clamp(0.0, 1.0));
      paint.set_shader(shader);
    }
  }

  Ok(())
}

fn gradient_shader_from_stops(
  stops: &[GradientStop],
  global_alpha: f32,
  make_shader: impl FnOnce(&[Color4f], Option<&[f32]>) -> Option<Shader>,
) -> Result<Option<Shader>> {
  match stops {
    [] => {
      return Ok(None);
    }
    [stop] => {
      return Ok(Some(skia_safe::shaders::color(parse_color(
        &stop.color,
        global_alpha,
      )?)));
    }
    _ => {}
  }

  let mut colors = Vec::with_capacity(stops.len());
  let mut positions = Vec::with_capacity(stops.len());
  for stop in stops {
    colors.push(Color4f::from(parse_color(&stop.color, global_alpha)?));
    positions.push(stop.offset);
  }

  Ok(make_shader(&colors, Some(&positions)))
}

fn pattern_image(
  source_kind: &str,
  src: &str,
  width: Option<usize>,
  height: Option<usize>,
  commands: &[DrawCommand],
  image_cache: &mut HashMap<String, Image>,
) -> Result<Image> {
  if source_kind == "canvas" {
    render_offscreen_image(width, height, commands, image_cache)
  } else if source_kind == "canvasBitmap" {
    load_canvas_bitmap(src, width, height, commands, image_cache)
  } else {
    load_image(src, image_cache)
  }
}

fn pattern_tile_modes(repetition: &str) -> (TileMode, TileMode) {
  match repetition {
    "repeat-x" => (TileMode::Repeat, TileMode::Decal),
    "repeat-y" => (TileMode::Decal, TileMode::Repeat),
    "no-repeat" => (TileMode::Decal, TileMode::Decal),
    _ => (TileMode::Repeat, TileMode::Repeat),
  }
}

fn load_canvas_bitmap(
  src: &str,
  width: Option<usize>,
  height: Option<usize>,
  commands: &[DrawCommand],
  image_cache: &mut HashMap<String, Image>,
) -> Result<Image> {
  let cache_key = format!("canvasBitmap:{src}");
  if let Some(image) = image_cache.get(&cache_key) {
    return Ok(image.clone());
  }

  let image = render_offscreen_image(width, height, commands, image_cache)?;
  image_cache.insert(cache_key, image.clone());
  Ok(image)
}

fn render_offscreen_image(
  width: Option<usize>,
  height: Option<usize>,
  commands: &[DrawCommand],
  image_cache: &mut HashMap<String, Image>,
) -> Result<Image> {
  let width = width.unwrap_or(0).clamp(1, 7680);
  let height = height.unwrap_or(0).clamp(1, 4320);

  let mut surface = skia_utils::create_surface(width, height)?;
  surface.canvas().clear(Color::TRANSPARENT);

  render_commands(&mut surface, commands, image_cache)?;

  Ok(surface.image_snapshot())
}

fn load_image(src: &str, image_cache: &mut HashMap<String, Image>) -> Result<Image> {
  if let Some(image) = image_cache.get(src) {
    return Ok(image.clone());
  }

  let bytes = if let Some(bytes) = decode_data_url(src)? {
    bytes
  } else {
    let path = resolve_asset_path(src);
    fs::read(&path).with_context(|| format!("failed to read image `{}`", path.display()))?
  };
  let data = Data::new_copy(&bytes);
  let image = Image::from_encoded(data).ok_or_else(|| anyhow!("failed to decode image"))?;
  // Keep the decoded pixels in the cache.  A software frame can draw the same
  // image thousands of times; retaining the raster image avoids making Skia
  // resolve the encoded image generator while processing every batch.
  let image = image
    .make_raster_image(None, Some(CachingHint::Allow))
    .unwrap_or(image);

  image_cache.insert(src.to_owned(), image.clone());
  Ok(image)
}

fn image_data_info(width: usize, height: usize) -> ImageInfo {
  ImageInfo::new(
    (width as i32, height as i32),
    ColorType::RGBA8888,
    AlphaType::Unpremul,
    None,
  )
}

fn image_from_image_data(data: &[u8], width: usize, height: usize) -> Result<Image> {
  if width == 0 || height == 0 {
    bail!("ImageData dimensions must be non-zero");
  }
  let expected_len = width
    .checked_mul(height)
    .and_then(|pixels| pixels.checked_mul(4))
    .ok_or_else(|| anyhow!("ImageData dimensions are too large"))?;
  if data.len() != expected_len {
    bail!("ImageData dimensions do not match data length");
  }

  let info = image_data_info(width, height);
  let pixels = Data::new_copy(data);
  images::raster_from_data(&info, pixels, width * 4)
    .ok_or_else(|| anyhow!("failed to create ImageData raster image"))
}

fn sorted_rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
  let left = x.min(x + width);
  let top = y.min(y + height);
  let right = x.max(x + width);
  let bottom = y.max(y + height);
  Rect::new(left, top, right, bottom)
}

#[allow(clippy::too_many_arguments)]
fn put_image_data(
  canvas: &Canvas,
  data: &[u8],
  width: usize,
  height: usize,
  dx: f32,
  dy: f32,
  dirty_x: Option<f32>,
  dirty_y: Option<f32>,
  dirty_width: Option<f32>,
  dirty_height: Option<f32>,
) -> Result<()> {
  let image = image_from_image_data(data, width, height)?;
  let src_rect = match (dirty_x, dirty_y, dirty_width, dirty_height) {
    (Some(x), Some(y), Some(width), Some(height)) => sorted_rect(x, y, width, height),
    _ => Rect::from_xywh(0.0, 0.0, width as f32, height as f32),
  };
  if src_rect.width() <= 0.0 || src_rect.height() <= 0.0 {
    return Ok(());
  }
  let dst_rect = src_rect.with_offset((dx, dy));
  let mut paint = Paint::default();
  paint.set_blend_mode(BlendMode::Src);
  canvas.draw_image_rect_with_sampling_options(
    &image,
    Some((&src_rect, SrcRectConstraint::Fast)),
    dst_rect,
    SamplingOptions::new(FilterMode::Nearest, MipmapMode::None),
    &paint,
  );
  Ok(())
}

fn read_image_data(
  canvas_width: usize,
  canvas_height: usize,
  commands: &[DrawCommand],
  sx: i32,
  sy: i32,
  sw: i32,
  sh: i32,
) -> Result<Vec<u8>> {
  if sw == 0 || sh == 0 {
    bail!("getImageData() width and height must be non-zero");
  }

  let canvas_width = canvas_width.clamp(1, 7680);
  let canvas_height = canvas_height.clamp(1, 4320);
  let read_width = sw.unsigned_abs() as usize;
  let read_height = sh.unsigned_abs() as usize;
  let left = sx.min(sx.saturating_add(sw));
  let top = sy.min(sy.saturating_add(sh));

  let mut surface = skia_utils::create_surface(canvas_width, canvas_height)?;
  surface.canvas().clear(Color::TRANSPARENT);

  let mut image_cache = HashMap::new();
  render_commands(&mut surface, commands, &mut image_cache)?;

  let mut canvas_pixels = vec![0_u8; canvas_width * canvas_height * 4];
  if !surface.read_pixels(
    &image_data_info(canvas_width, canvas_height),
    canvas_pixels.as_mut_slice(),
    canvas_width * 4,
    (0, 0),
  ) {
    bail!("failed to read ImageData pixels");
  }

  let mut output = vec![0_u8; read_width * read_height * 4];
  for row in 0..read_height {
    let source_y = top + row as i32;
    if source_y < 0 || source_y >= canvas_height as i32 {
      continue;
    }
    for column in 0..read_width {
      let source_x = left + column as i32;
      if source_x < 0 || source_x >= canvas_width as i32 {
        continue;
      }
      let source_index = ((source_y as usize * canvas_width) + source_x as usize) * 4;
      let dest_index = ((row * read_width) + column) * 4;
      output[dest_index..dest_index + 4]
        .copy_from_slice(&canvas_pixels[source_index..source_index + 4]);
    }
  }

  Ok(output)
}

fn skia_image_sampling(image_sampling: &ImageSampling) -> SamplingOptions {
  if !image_sampling.enabled {
    return SamplingOptions::new(FilterMode::Nearest, MipmapMode::None);
  }

  match image_sampling.quality.as_str() {
    "medium" | "high" => SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear),
    _ => SamplingOptions::new(FilterMode::Linear, MipmapMode::Nearest),
  }
}

fn matrix_from_transform(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Matrix {
  Matrix::new_all(a, c, e, b, d, f, 0.0, 0.0, 1.0)
}

pub(crate) fn render_commands(
  surface: &mut skia_safe::Surface,
  commands: &[DrawCommand],
  image_cache: &mut HashMap<String, Image>,
) -> Result<()> {
  let canvas = surface.canvas();
  let base_matrix = canvas.local_to_device_as_3x3();

  for command in commands {
    match command {
      DrawCommand::Exit => {}
      DrawCommand::Save => {
        canvas.save();
      }
      DrawCommand::Restore => {
        canvas.restore();
      }
      DrawCommand::Translate { x, y } => {
        canvas.translate((*x, *y));
      }
      DrawCommand::Rotate { angle } => {
        canvas.rotate(angle.to_degrees(), None);
      }
      DrawCommand::Scale { x, y } => {
        canvas.scale((*x, *y));
      }
      DrawCommand::Transform { a, b, c, d, e, f } => {
        canvas.concat(&matrix_from_transform(*a, *b, *c, *d, *e, *f));
      }
      DrawCommand::SetTransform { a, b, c, d, e, f } => {
        canvas.reset_matrix();
        canvas.concat(&base_matrix);
        canvas.concat(&matrix_from_transform(*a, *b, *c, *d, *e, *f));
      }
      DrawCommand::ResetTransform => {
        canvas.reset_matrix();
        canvas.concat(&base_matrix);
      }
      DrawCommand::ClearRect {
        x,
        y,
        width,
        height,
      } => {
        let mut paint = Paint::default();
        paint.set_blend_mode(BlendMode::Clear);
        canvas.draw_rect(Rect::from_xywh(*x, *y, *width, *height), &paint);
      }
      DrawCommand::FillRect {
        x,
        y,
        width,
        height,
        fill_style,
        global_alpha,
        global_composite_operation,
        image_smoothing_enabled,
        image_smoothing_quality,
      } => {
        let image_sampling =
          ImageSampling::new(*image_smoothing_enabled, image_smoothing_quality.as_str());
        let paint = fill_paint(
          fill_style,
          *global_alpha,
          global_composite_operation,
          &image_sampling,
          image_cache,
        )?;
        canvas.draw_rect(Rect::from_xywh(*x, *y, *width, *height), &paint);
      }
      DrawCommand::StrokeRect {
        x,
        y,
        width,
        height,
        stroke_style,
        global_alpha,
        global_composite_operation,
        image_smoothing_enabled,
        image_smoothing_quality,
        line_width,
        line_cap,
        line_join,
        miter_limit,
      } => {
        let image_sampling =
          ImageSampling::new(*image_smoothing_enabled, image_smoothing_quality.as_str());
        let paint = stroke_paint(
          stroke_style,
          *global_alpha,
          global_composite_operation,
          *line_width,
          line_cap,
          line_join,
          *miter_limit,
          &image_sampling,
          image_cache,
        )?;
        canvas.draw_rect(Rect::from_xywh(*x, *y, *width, *height), &paint);
      }
      DrawCommand::FillPath {
        path,
        fill_style,
        global_alpha,
        global_composite_operation,
        image_smoothing_enabled,
        image_smoothing_quality,
        rule,
      } => {
        let mut path = path_segement::build_path_from_segment(path)?;
        if rule == "evenodd" {
          path.set_fill_type(PathFillType::EvenOdd);
        }
        let image_sampling =
          ImageSampling::new(*image_smoothing_enabled, image_smoothing_quality.as_str());
        let paint = fill_paint(
          fill_style,
          *global_alpha,
          global_composite_operation,
          &image_sampling,
          image_cache,
        )?;
        canvas.draw_path(&path, &paint);
      }
      DrawCommand::ClipPath { path, rule } => {
        let mut path = path_segement::build_path_from_segment(path)?;
        if rule == "evenodd" {
          path.set_fill_type(PathFillType::EvenOdd);
        }
        canvas.clip_path(&path, skia_safe::ClipOp::Intersect, true);
      }
      DrawCommand::StrokePath {
        path,
        stroke_style,
        global_alpha,
        global_composite_operation,
        image_smoothing_enabled,
        image_smoothing_quality,
        line_width,
        line_cap,
        line_join,
        miter_limit,
      } => {
        let path = path_segement::build_path_from_segment(path)?;
        let image_sampling =
          ImageSampling::new(*image_smoothing_enabled, image_smoothing_quality.as_str());
        let paint = stroke_paint(
          stroke_style,
          *global_alpha,
          global_composite_operation,
          *line_width,
          line_cap,
          line_join,
          *miter_limit,
          &image_sampling,
          image_cache,
        )?;
        canvas.draw_path(&path, &paint);
      }
      DrawCommand::FillCircle {
        x,
        y,
        radius,
        fill_style,
        global_alpha,
        global_composite_operation,
        image_smoothing_enabled,
        image_smoothing_quality,
      } => {
        let image_sampling =
          ImageSampling::new(*image_smoothing_enabled, image_smoothing_quality.as_str());
        let paint = fill_paint(
          fill_style,
          *global_alpha,
          global_composite_operation,
          &image_sampling,
          image_cache,
        )?;
        canvas.draw_circle((*x, *y), *radius, &paint);
      }
      DrawCommand::StrokeLine {
        x0,
        y0,
        x1,
        y1,
        stroke_style,
        global_alpha,
        global_composite_operation,
        image_smoothing_enabled,
        image_smoothing_quality,
        line_width,
        line_cap,
        line_join,
        miter_limit,
      } => {
        let image_sampling =
          ImageSampling::new(*image_smoothing_enabled, image_smoothing_quality.as_str());
        let paint = stroke_paint(
          stroke_style,
          *global_alpha,
          global_composite_operation,
          *line_width,
          line_cap,
          line_join,
          *miter_limit,
          &image_sampling,
          image_cache,
        )?;
        canvas.draw_line((*x0, *y0), (*x1, *y1), &paint);
      }
      DrawCommand::CacheCanvasBitmap {
        src,
        width,
        height,
        commands,
      } => {
        load_canvas_bitmap(src, *width, *height, commands, image_cache)?;
      }
      DrawCommand::DrawImage {
        source_kind,
        src,
        dx,
        dy,
        dw,
        dh,
        sx,
        sy,
        sw,
        sh,
        global_alpha,
        global_composite_operation,
        image_smoothing_enabled,
        image_smoothing_quality,
        bitmap_sx,
        bitmap_sy,
        bitmap_sw,
        bitmap_sh,
        width,
        height,
        commands,
      } => {
        let bitmap_rect = match (bitmap_sx, bitmap_sy, bitmap_sw, bitmap_sh) {
          (Some(sx), Some(sy), Some(sw), Some(sh)) => Some(Rect::from_xywh(*sx, *sy, *sw, *sh)),
          _ => None,
        };
        let reject_rect = match (dw, dh, bitmap_rect) {
          (Some(dw), Some(dh), _) => Some(Rect::from_xywh(*dx, *dy, *dw, *dh)),
          (_, _, Some(src_rect)) => Some(Rect::from_xywh(
            *dx,
            *dy,
            src_rect.width(),
            src_rect.height(),
          )),
          (_, _, None) => match (width, height) {
            (Some(width), Some(height)) => {
              Some(Rect::from_xywh(*dx, *dy, *width as f32, *height as f32))
            }
            _ => None,
          },
        };

        if let Some(rect) = reject_rect
          && canvas.quick_reject(&rect)
        {
          continue;
        }

        let image = if source_kind == "canvas" {
          render_offscreen_image(*width, *height, commands, image_cache)?
        } else if source_kind == "canvasBitmap" {
          load_canvas_bitmap(src, *width, *height, commands, image_cache)?
        } else {
          load_image(src, image_cache)?
        };
        let mut paint = Paint::default();
        paint.set_anti_alias(false);
        paint.set_alpha_f(global_alpha.clamp(0.0, 1.0));
        paint.set_blend_mode(blend_mode(global_composite_operation));
        let image_sampling =
          ImageSampling::new(*image_smoothing_enabled, image_smoothing_quality.as_str());
        let sampling = skia_image_sampling(&image_sampling);

        match (dw, dh, sx, sy, sw, sh) {
          (Some(dw), Some(dh), Some(sx), Some(sy), Some(sw), Some(sh)) => {
            let src_rect = match bitmap_rect {
              Some(bitmap_rect) => {
                Rect::from_xywh(bitmap_rect.left + *sx, bitmap_rect.top + *sy, *sw, *sh)
              }
              None => Rect::from_xywh(*sx, *sy, *sw, *sh),
            };
            let dst_rect = Rect::from_xywh(*dx, *dy, *dw, *dh);
            canvas.draw_image_rect_with_sampling_options(
              &image,
              Some((&src_rect, SrcRectConstraint::Fast)),
              dst_rect,
              sampling,
              &paint,
            );
          }
          (Some(dw), Some(dh), _, _, _, _) => {
            let dst_rect = Rect::from_xywh(*dx, *dy, *dw, *dh);
            match bitmap_rect {
              Some(src_rect) => {
                canvas.draw_image_rect_with_sampling_options(
                  &image,
                  Some((&src_rect, SrcRectConstraint::Fast)),
                  dst_rect,
                  sampling,
                  &paint,
                );
              }
              None => {
                canvas
                  .draw_image_rect_with_sampling_options(&image, None, dst_rect, sampling, &paint);
              }
            }
          }
          _ => match bitmap_rect {
            Some(src_rect) => {
              let dst_rect = Rect::from_xywh(*dx, *dy, src_rect.width(), src_rect.height());
              canvas.draw_image_rect_with_sampling_options(
                &image,
                Some((&src_rect, SrcRectConstraint::Fast)),
                dst_rect,
                sampling,
                &paint,
              );
            }
            None => {
              canvas.draw_image_with_sampling_options(&image, (*dx, *dy), sampling, Some(&paint));
            }
          },
        }
      }
      DrawCommand::DrawImageBatch {
        source_kind,
        src,
        global_alpha,
        global_composite_operation,
        image_smoothing_enabled,
        image_smoothing_quality,
        bitmap_sx,
        bitmap_sy,
        bitmap_sw,
        bitmap_sh,
        width,
        height,
        draws,
      } => {
        let image = if source_kind == "canvasBitmap" {
          load_canvas_bitmap(src, *width, *height, &[], image_cache)?
        } else {
          load_image(src, image_cache)?
        };
        let mut paint = Paint::default();
        paint.set_anti_alias(false);
        paint.set_alpha_f(global_alpha.clamp(0.0, 1.0));
        paint.set_blend_mode(blend_mode(global_composite_operation));
        let image_sampling =
          ImageSampling::new(*image_smoothing_enabled, image_smoothing_quality.as_str());
        let sampling = skia_image_sampling(&image_sampling);
        let src_rect = match (bitmap_sx, bitmap_sy, bitmap_sw, bitmap_sh) {
          (Some(sx), Some(sy), Some(sw), Some(sh)) => Some(Rect::from_xywh(*sx, *sy, *sw, *sh)),
          _ => None,
        };

        // A batch is normally made up of many axis-aligned copies of the same
        // image.  Sending those copies through draw_atlas keeps the whole batch
        // in one Skia call.  Atlas uses a uniform scale per sprite, so retain
        // the old path for non-uniform or otherwise invalid rectangles.
        let atlas_src = src_rect.unwrap_or_else(|| {
          Rect::from_xywh(0.0, 0.0, image.width() as f32, image.height() as f32)
        });
        if atlas_src.width() > 0.0
          && atlas_src.height() > 0.0
          && let Some((xforms, tex)) = atlas_batch_geometry(draws, atlas_src)
        {
          canvas.draw_atlas(
            &image,
            &xforms,
            &tex,
            None,
            BlendMode::SrcOver,
            sampling,
            None,
            Some(&paint),
          );
          continue;
        }

        for draw in draws.chunks_exact(4) {
          let dst_rect = Rect::from_xywh(draw[0], draw[1], draw[2], draw[3]);

          match src_rect {
            Some(src_rect) => canvas.draw_image_rect_with_sampling_options(
              &image,
              Some((&src_rect, SrcRectConstraint::Fast)),
              dst_rect,
              sampling,
              &paint,
            ),
            None => {
              canvas.draw_image_rect_with_sampling_options(&image, None, dst_rect, sampling, &paint)
            }
          };
        }
      }
      DrawCommand::PutImageData {
        data,
        width,
        height,
        dx,
        dy,
        dirty_x,
        dirty_y,
        dirty_width,
        dirty_height,
      } => {
        let save_count = canvas.save();
        canvas.reset_matrix();
        put_image_data(
          canvas,
          data,
          *width,
          *height,
          *dx,
          *dy,
          *dirty_x,
          *dirty_y,
          *dirty_width,
          *dirty_height,
        )?;
        canvas.restore_to_count(save_count);
      }
    }
  }

  Ok(())
}

/// Renders the common software-game frame without routing every sprite through
/// Skia's general image pipeline. This is deliberately narrow: callers retain
/// Skia for all other command combinations and for canvas semantics that cannot
/// be represented by this blitter.
pub(crate) fn render_software_sprite_batch(
  surface: &mut skia_safe::Surface,
  commands: &[DrawCommand],
  image_cache: &mut HashMap<String, Image>,
) -> Result<bool> {
  let [
    background,
    DrawCommand::DrawImageBatch {
      source_kind,
      src,
      global_alpha,
      global_composite_operation,
      image_smoothing_enabled,
      bitmap_sx,
      bitmap_sy,
      bitmap_sw,
      bitmap_sh,
      draws,
      ..
    },
  ] = commands
  else {
    return Ok(false);
  };

  let DrawCommand::FillRect {
    x,
    y,
    width,
    height,
    fill_style: PaintStyle::Color(_),
    global_alpha: background_alpha,
    global_composite_operation: background_composite,
    ..
  } = background
  else {
    return Ok(false);
  };

  if source_kind != "image"
    || *global_alpha != 1.0
    || global_composite_operation != "source-over"
    || *background_alpha != 1.0
    || background_composite != "source-over"
    || *x != 0.0
    || *y != 0.0
    || *width != surface.width() as f32
    || *height != surface.height() as f32
    || draws.len() % 4 != 0
    || bitmap_sx.is_some()
    || bitmap_sy.is_some()
    || bitmap_sw.is_some()
    || bitmap_sh.is_some()
  {
    return Ok(false);
  }

  let image = load_image(src, image_cache)?;
  let Some(source_pixels) = image.peek_pixels() else {
    return Ok(false);
  };
  let source_bgra = match source_pixels.color_type() {
    ColorType::BGRA8888 => true,
    ColorType::RGBA8888 => false,
    _ => return Ok(false),
  };
  if source_pixels.alpha_type() != AlphaType::Premul {
    return Ok(false);
  }
  // Draw the background with Skia so color parsing remains identical, then
  // blend the sprites directly into the raster surface.
  render_commands(surface, std::slice::from_ref(background), image_cache)?;
  let Some(destination_pixels) = surface.peek_pixels() else {
    return Ok(false);
  };
  let destination_bgra = match destination_pixels.color_type() {
    ColorType::BGRA8888 => true,
    ColorType::RGBA8888 => false,
    _ => return Ok(false),
  };
  if destination_pixels.alpha_type() != AlphaType::Premul
    || destination_pixels.alpha_type() != AlphaType::Premul
  {
    return Ok(false);
  }

  let source_width = source_pixels.width() as usize;
  let source_height = source_pixels.height() as usize;
  let destination_width = destination_pixels.width() as usize;
  let destination_height = destination_pixels.height() as usize;
  let source = unsafe {
    std::slice::from_raw_parts(
      source_pixels.addr() as *const u8,
      source_pixels.compute_byte_size(),
    )
  };
  let destination = unsafe {
    std::slice::from_raw_parts_mut(
      destination_pixels.addr() as *mut u8,
      destination_pixels.compute_byte_size(),
    )
  };

  let exact_size = draws
    .chunks_exact(4)
    .all(|draw| draw[2] == source_width as f32 && draw[3] == source_height as f32);
  if exact_size && draws.len() / 4 >= 256 && destination_height >= 128 {
    blit_sprite_exact_parallel(
      source,
      source_pixels.row_bytes(),
      source_width,
      source_height,
      source_bgra,
      destination,
      destination_pixels.row_bytes(),
      destination_width,
      destination_height,
      destination_bgra,
      draws,
    );
  } else {
    for draw in draws.chunks_exact(4) {
      if exact_size {
        blit_sprite_exact(
          source,
          source_pixels.row_bytes(),
          source_width,
          source_height,
          source_bgra,
          destination,
          destination_pixels.row_bytes(),
          destination_width,
          destination_height,
          destination_bgra,
          draw[0],
          draw[1],
        );
      } else {
        blit_sprite(
          source,
          source_pixels.row_bytes(),
          source_width,
          source_height,
          source_bgra,
          destination,
          destination_pixels.row_bytes(),
          destination_width,
          destination_height,
          destination_bgra,
          draw[0],
          draw[1],
          draw[2],
          draw[3],
          *image_smoothing_enabled,
        );
      }
    }
  }
  surface.notify_content_will_change(ContentChangeMode::Retain);
  Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn blit_sprite_exact_parallel(
  source: &[u8],
  source_row_bytes: usize,
  source_width: usize,
  source_height: usize,
  source_bgra: bool,
  destination: &mut [u8],
  destination_row_bytes: usize,
  destination_width: usize,
  destination_height: usize,
  destination_bgra: bool,
  draws: &[f32],
) {
  const TILE_HEIGHT: usize = 64;
  destination
    .par_chunks_mut(destination_row_bytes * TILE_HEIGHT)
    .enumerate()
    .for_each(|(tile_index, tile)| {
      let y_start = tile_index * TILE_HEIGHT;
      let y_end = (y_start + TILE_HEIGHT).min(destination_height);
      for draw in draws.chunks_exact(4) {
        blit_sprite_exact_rows(
          source,
          source_row_bytes,
          source_width,
          source_height,
          source_bgra,
          tile,
          destination_row_bytes,
          destination_width,
          y_start,
          y_end,
          destination_bgra,
          draw[0],
          draw[1],
        );
      }
    });
}

#[allow(clippy::too_many_arguments)]
fn blit_sprite_exact_rows(
  source: &[u8],
  source_row_bytes: usize,
  source_width: usize,
  source_height: usize,
  source_bgra: bool,
  destination: &mut [u8],
  destination_row_bytes: usize,
  destination_width: usize,
  y_start: usize,
  y_end: usize,
  destination_bgra: bool,
  dx: f32,
  dy: f32,
) {
  let dx = dx.floor() as isize;
  let dy = dy.floor() as isize;
  let (red, green, blue) = if destination_bgra {
    (2, 1, 0)
  } else {
    (0, 1, 2)
  };
  for sy in 0..source_height {
    let y = dy + sy as isize;
    if y < y_start as isize || y >= y_end as isize {
      continue;
    }
    for sx in 0..source_width {
      let x = dx + sx as isize;
      if !(0..destination_width as isize).contains(&x) {
        continue;
      }
      let source_offset = sy * source_row_bytes + sx * 4;
      let (r, g, b, a) = if source_bgra {
        (
          source[source_offset + 2],
          source[source_offset + 1],
          source[source_offset],
          source[source_offset + 3],
        )
      } else {
        (
          source[source_offset],
          source[source_offset + 1],
          source[source_offset + 2],
          source[source_offset + 3],
        )
      };
      if a == 0 {
        continue;
      }
      let destination_offset = (y as usize - y_start) * destination_row_bytes + x as usize * 4;
      let inverse = 255_u16 - a as u16;
      destination[destination_offset + red] =
        (r as u16 + destination[destination_offset + red] as u16 * inverse / 255) as u8;
      destination[destination_offset + green] =
        (g as u16 + destination[destination_offset + green] as u16 * inverse / 255) as u8;
      destination[destination_offset + blue] =
        (b as u16 + destination[destination_offset + blue] as u16 * inverse / 255) as u8;
      destination[destination_offset + 3] =
        (a as u16 + destination[destination_offset + 3] as u16 * inverse / 255) as u8;
    }
  }
}

#[allow(clippy::too_many_arguments)]
fn blit_sprite_exact(
  source: &[u8],
  source_row_bytes: usize,
  source_width: usize,
  source_height: usize,
  source_bgra: bool,
  destination: &mut [u8],
  destination_row_bytes: usize,
  destination_width: usize,
  destination_height: usize,
  destination_bgra: bool,
  dx: f32,
  dy: f32,
) {
  let dx = dx.floor() as isize;
  let dy = dy.floor() as isize;
  let (red, green, blue) = if destination_bgra {
    (2, 1, 0)
  } else {
    (0, 1, 2)
  };
  for sy in 0..source_height {
    let y = dy + sy as isize;
    if !(0..destination_height as isize).contains(&y) {
      continue;
    }
    for sx in 0..source_width {
      let x = dx + sx as isize;
      if !(0..destination_width as isize).contains(&x) {
        continue;
      }
      let source_offset = sy * source_row_bytes + sx * 4;
      let (r, g, b, a) = if source_bgra {
        (
          source[source_offset + 2],
          source[source_offset + 1],
          source[source_offset],
          source[source_offset + 3],
        )
      } else {
        (
          source[source_offset],
          source[source_offset + 1],
          source[source_offset + 2],
          source[source_offset + 3],
        )
      };
      if a == 0 {
        continue;
      }
      let destination_offset = y as usize * destination_row_bytes + x as usize * 4;
      let inverse = 255_u16 - a as u16;
      destination[destination_offset + red] =
        (r as u16 + destination[destination_offset + red] as u16 * inverse / 255) as u8;
      destination[destination_offset + green] =
        (g as u16 + destination[destination_offset + green] as u16 * inverse / 255) as u8;
      destination[destination_offset + blue] =
        (b as u16 + destination[destination_offset + blue] as u16 * inverse / 255) as u8;
      destination[destination_offset + 3] =
        (a as u16 + destination[destination_offset + 3] as u16 * inverse / 255) as u8;
    }
  }
}

#[allow(clippy::too_many_arguments)]
fn blit_sprite(
  source: &[u8],
  source_row_bytes: usize,
  source_width: usize,
  source_height: usize,
  source_bgra: bool,
  destination: &mut [u8],
  destination_row_bytes: usize,
  destination_width: usize,
  destination_height: usize,
  destination_bgra: bool,
  dx: f32,
  dy: f32,
  dw: f32,
  dh: f32,
  smooth: bool,
) {
  if !dx.is_finite()
    || !dy.is_finite()
    || !dw.is_finite()
    || !dh.is_finite()
    || dw <= 0.0
    || dh <= 0.0
  {
    return;
  }
  let left = (dx.floor() as isize).max(0) as usize;
  let top = (dy.floor() as isize).max(0) as usize;
  let right = ((dx + dw).ceil() as isize)
    .min(destination_width as isize)
    .max(0) as usize;
  let bottom = ((dy + dh).ceil() as isize)
    .min(destination_height as isize)
    .max(0) as usize;

  for y in top..bottom {
    for x in left..right {
      let u = ((x as f32 + 0.5 - dx) / dw) * source_width as f32 - 0.5;
      let v = ((y as f32 + 0.5 - dy) / dh) * source_height as f32 - 0.5;
      let (r, g, b, a) = if smooth {
        sample_bilinear(
          source,
          source_row_bytes,
          source_width,
          source_height,
          u,
          v,
          source_bgra,
        )
      } else {
        sample_nearest(
          source,
          source_row_bytes,
          source_width,
          source_height,
          u,
          v,
          source_bgra,
        )
      };
      if a == 0 {
        continue;
      }
      let offset = y * destination_row_bytes + x * 4;
      let inverse = 255_u16 - a as u16;
      let (red, green, blue) = if destination_bgra {
        (2, 1, 0)
      } else {
        (0, 1, 2)
      };
      destination[offset + red] =
        (r as u16 + destination[offset + red] as u16 * inverse / 255) as u8;
      destination[offset + green] =
        (g as u16 + destination[offset + green] as u16 * inverse / 255) as u8;
      destination[offset + blue] =
        (b as u16 + destination[offset + blue] as u16 * inverse / 255) as u8;
      destination[offset + 3] = (a as u16 + destination[offset + 3] as u16 * inverse / 255) as u8;
    }
  }
}

fn sample_nearest(
  source: &[u8],
  row_bytes: usize,
  width: usize,
  height: usize,
  x: f32,
  y: f32,
  bgra: bool,
) -> (u8, u8, u8, u8) {
  let x = x.round().clamp(0.0, (width - 1) as f32) as usize;
  let y = y.round().clamp(0.0, (height - 1) as f32) as usize;
  let offset = y * row_bytes + x * 4;
  if bgra {
    (
      source[offset + 2],
      source[offset + 1],
      source[offset],
      source[offset + 3],
    )
  } else {
    (
      source[offset],
      source[offset + 1],
      source[offset + 2],
      source[offset + 3],
    )
  }
}

fn sample_bilinear(
  source: &[u8],
  row_bytes: usize,
  width: usize,
  height: usize,
  x: f32,
  y: f32,
  bgra: bool,
) -> (u8, u8, u8, u8) {
  let x0 = x.floor();
  let y0 = y.floor();
  let fx = x - x0;
  let fy = y - y0;
  let x0 = x0.clamp(0.0, (width - 1) as f32) as usize;
  let y0 = y0.clamp(0.0, (height - 1) as f32) as usize;
  let x1 = (x0 + 1).min(width - 1);
  let y1 = (y0 + 1).min(height - 1);
  let p = |x: usize, y: usize| {
    let offset = y * row_bytes + x * 4;
    if bgra {
      [
        source[offset + 2],
        source[offset + 1],
        source[offset],
        source[offset + 3],
      ]
    } else {
      [
        source[offset],
        source[offset + 1],
        source[offset + 2],
        source[offset + 3],
      ]
    }
  };
  let p00 = p(x0, y0);
  let p10 = p(x1, y0);
  let p01 = p(x0, y1);
  let p11 = p(x1, y1);
  let mix = |channel: usize| {
    let top = p00[channel] as f32 + (p10[channel] as f32 - p00[channel] as f32) * fx;
    let bottom = p01[channel] as f32 + (p11[channel] as f32 - p01[channel] as f32) * fx;
    (top + (bottom - top) * fy).round().clamp(0.0, 255.0) as u8
  };
  (mix(0), mix(1), mix(2), mix(3))
}

fn atlas_batch_geometry(draws: &[f32], src: Rect) -> Option<(Vec<RSXform>, Vec<Rect>)> {
  let mut xforms = Vec::with_capacity(draws.len() / 4);
  let mut tex = Vec::with_capacity(draws.len() / 4);

  for draw in draws.chunks_exact(4) {
    let (x, y, width, height) = (draw[0], draw[1], draw[2], draw[3]);
    let scale_x = width / src.width();
    let scale_y = height / src.height();
    if !x.is_finite()
      || !y.is_finite()
      || !width.is_finite()
      || !height.is_finite()
      || width <= 0.0
      || height <= 0.0
      || !scale_x.is_finite()
      || !scale_y.is_finite()
      || (scale_x - scale_y).abs() > 0.0001
    {
      return None;
    }

    // RSXform maps the atlas rect into destination space.  Offset by the
    // source rect's origin because the transform is applied to image-space
    // coordinates, not to a rect-local coordinate.
    xforms.push(RSXform::new(
      scale_x,
      0.0,
      (x - scale_x * src.left, y - scale_x * src.top),
    ));
    tex.push(src);
  }

  Some((xforms, tex))
}
