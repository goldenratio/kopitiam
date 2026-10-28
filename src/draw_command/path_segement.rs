use anyhow::{Result, bail};
use boa_engine::{Context as JsContext, JsObject, JsValue};
use serde::Deserialize;
use skia_safe::{Matrix, Path, PathBuilder, PathDirection, Rect, path::AddPathMode};

use crate::{
  boa_errors,
  boa_utils::{
    array_len, get_property, optional_bool, required_f32, required_string, value_to_object,
  },
};

#[derive(Debug, Deserialize)]
#[serde(tag = "verb", rename_all = "camelCase")]
pub(crate) enum PathSegment {
  ClosePath,
  MoveTo {
    x: f32,
    y: f32,
  },
  LineTo {
    x: f32,
    y: f32,
  },
  BezierCurveTo {
    #[serde(rename = "cp1x")]
    cp1x: f32,
    #[serde(rename = "cp1y")]
    cp1y: f32,
    #[serde(rename = "cp2x")]
    cp2x: f32,
    #[serde(rename = "cp2y")]
    cp2y: f32,
    x: f32,
    y: f32,
  },
  QuadraticCurveTo {
    #[serde(rename = "cpx")]
    cpx: f32,
    #[serde(rename = "cpy")]
    cpy: f32,
    x: f32,
    y: f32,
  },
  Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
  },
  Arc {
    x: f32,
    y: f32,
    radius: f32,
    #[serde(rename = "startAngle")]
    start_angle: f32,
    #[serde(rename = "endAngle")]
    end_angle: f32,
    #[serde(default)]
    counterclockwise: bool,
  },
  ArcTo {
    #[serde(rename = "x1")]
    x1: f32,
    #[serde(rename = "y1")]
    y1: f32,
    #[serde(rename = "x2")]
    x2: f32,
    #[serde(rename = "y2")]
    y2: f32,
    radius: f32,
  },
  Ellipse {
    x: f32,
    y: f32,
    #[serde(rename = "radiusX")]
    radius_x: f32,
    #[serde(rename = "radiusY")]
    radius_y: f32,
    rotation: f32,
    #[serde(rename = "startAngle")]
    start_angle: f32,
    #[serde(rename = "endAngle")]
    end_angle: f32,
    #[serde(default)]
    counterclockwise: bool,
  },
}

fn path_segment_from_value(value: JsValue, js: &mut JsContext) -> Result<PathSegment> {
  let object = value_to_object(value, "path segment")?;
  let verb = required_string(&object, "verb", js)?;

  match verb.as_str() {
    "closePath" => Ok(PathSegment::ClosePath),
    "moveTo" => Ok(PathSegment::MoveTo {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
    }),
    "lineTo" => Ok(PathSegment::LineTo {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
    }),
    "bezierCurveTo" => Ok(PathSegment::BezierCurveTo {
      cp1x: required_f32(&object, "cp1x", js)?,
      cp1y: required_f32(&object, "cp1y", js)?,
      cp2x: required_f32(&object, "cp2x", js)?,
      cp2y: required_f32(&object, "cp2y", js)?,
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
    }),
    "quadraticCurveTo" => Ok(PathSegment::QuadraticCurveTo {
      cpx: required_f32(&object, "cpx", js)?,
      cpy: required_f32(&object, "cpy", js)?,
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
    }),
    "rect" => Ok(PathSegment::Rect {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
      width: required_f32(&object, "width", js)?,
      height: required_f32(&object, "height", js)?,
    }),
    "arc" => Ok(PathSegment::Arc {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
      radius: required_f32(&object, "radius", js)?,
      start_angle: required_f32(&object, "startAngle", js)?,
      end_angle: required_f32(&object, "endAngle", js)?,
      counterclockwise: optional_bool(&object, "counterclockwise", js)?.unwrap_or(false),
    }),
    "arcTo" => Ok(PathSegment::ArcTo {
      x1: required_f32(&object, "x1", js)?,
      y1: required_f32(&object, "y1", js)?,
      x2: required_f32(&object, "x2", js)?,
      y2: required_f32(&object, "y2", js)?,
      radius: required_f32(&object, "radius", js)?,
    }),
    "ellipse" => Ok(PathSegment::Ellipse {
      x: required_f32(&object, "x", js)?,
      y: required_f32(&object, "y", js)?,
      radius_x: required_f32(&object, "radiusX", js)?,
      radius_y: required_f32(&object, "radiusY", js)?,
      rotation: required_f32(&object, "rotation", js)?,
      start_angle: required_f32(&object, "startAngle", js)?,
      end_angle: required_f32(&object, "endAngle", js)?,
      counterclockwise: optional_bool(&object, "counterclockwise", js)?.unwrap_or(false),
    }),
    _ => bail!("unsupported path segment `{verb}`"),
  }
}

fn arc_sweep_degrees(start_angle: f32, end_angle: f32, counterclockwise: bool) -> f32 {
  let mut sweep_degrees = (end_angle - start_angle).to_degrees();
  if counterclockwise && sweep_degrees > 0.0 {
    sweep_degrees -= 360.0;
  } else if !counterclockwise && sweep_degrees < 0.0 {
    sweep_degrees += 360.0;
  }
  sweep_degrees
}

fn ellipse_matrix(x: f32, y: f32, radius_x: f32, radius_y: f32, rotation: f32) -> Matrix {
  let (sin, cos) = rotation.sin_cos();
  Matrix::new_all(
    radius_x * cos,
    -radius_y * sin,
    x,
    radius_x * sin,
    radius_y * cos,
    y,
    0.0,
    0.0,
    1.0,
  )
}

pub(crate) fn required_path_segments(
  object: &JsObject,
  key: &'static str,
  js: &mut JsContext,
) -> Result<Vec<PathSegment>> {
  let value = get_property(object, key, js)?;
  let array = value_to_object(value, key)?;
  let len = array_len(&array, js)?;
  let mut segments = Vec::with_capacity(len);

  for index in 0..len {
    let value = array.get(index as u32, js).map_err(boa_errors::js_error)?;
    segments.push(path_segment_from_value(value, js)?);
  }

  Ok(segments)
}

pub(crate) fn build_path_from_segment(segments: &[PathSegment]) -> Result<Path> {
  let mut path = PathBuilder::new();

  for segment in segments {
    match segment {
      PathSegment::ClosePath => {
        path.close();
      }
      PathSegment::MoveTo { x, y } => {
        path.move_to((*x, *y));
      }
      PathSegment::LineTo { x, y } => {
        path.line_to((*x, *y));
      }
      PathSegment::BezierCurveTo {
        cp1x,
        cp1y,
        cp2x,
        cp2y,
        x,
        y,
      } => {
        path.cubic_to((*cp1x, *cp1y), (*cp2x, *cp2y), (*x, *y));
      }
      PathSegment::QuadraticCurveTo { cpx, cpy, x, y } => {
        path.quad_to((*cpx, *cpy), (*x, *y));
      }
      PathSegment::Rect {
        x,
        y,
        width,
        height,
      } => {
        path.add_rect(Rect::from_xywh(*x, *y, *width, *height), None, None);
      }
      PathSegment::Arc {
        x,
        y,
        radius,
        start_angle,
        end_angle,
        counterclockwise,
      } => {
        if *radius < 0.0 {
          bail!("arc radius must be non-negative");
        }

        let oval = Rect::from_xywh(*x - *radius, *y - *radius, *radius * 2.0, *radius * 2.0);
        let start_degrees = start_angle.to_degrees();
        let sweep_degrees = arc_sweep_degrees(*start_angle, *end_angle, *counterclockwise);

        if sweep_degrees.abs() >= 360.0 {
          path.add_circle((*x, *y), *radius, None);
        } else {
          path.arc_to(oval, start_degrees, sweep_degrees, false);
        }
      }
      PathSegment::ArcTo {
        x1,
        y1,
        x2,
        y2,
        radius,
      } => {
        if *radius < 0.0 {
          bail!("arcTo radius must be non-negative");
        }
        path.arc_to_tangent((*x1, *y1), (*x2, *y2), *radius);
      }
      PathSegment::Ellipse {
        x,
        y,
        radius_x,
        radius_y,
        rotation,
        start_angle,
        end_angle,
        counterclockwise,
      } => {
        if *radius_x < 0.0 || *radius_y < 0.0 {
          bail!("ellipse radius must be non-negative");
        }

        let matrix = ellipse_matrix(*x, *y, *radius_x, *radius_y, *rotation);
        let sweep_degrees = arc_sweep_degrees(*start_angle, *end_angle, *counterclockwise);
        let mut ellipse = PathBuilder::new();
        if sweep_degrees.abs() >= 360.0 {
          ellipse.add_oval(
            Rect::from_xywh(-1.0, -1.0, 2.0, 2.0),
            if *counterclockwise {
              PathDirection::CCW
            } else {
              PathDirection::CW
            },
            0,
          );
          path.add_path_with_transform(&ellipse.detach(), &matrix, AddPathMode::Append);
        } else {
          ellipse.add_arc(
            Rect::from_xywh(-1.0, -1.0, 2.0, 2.0),
            start_angle.to_degrees(),
            sweep_degrees,
          );
          path.add_path_with_transform(&ellipse.detach(), &matrix, AddPathMode::Extend);
        }
      }
    }
  }

  Ok(path.detach())
}
