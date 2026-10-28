use std::{cell::RefCell, collections::HashMap};

use anyhow::{Result, anyhow, bail};
use boa_engine::{Context as JsContext, JsValue, js_string, object::JsObject, string::JsString};

use crate::boa_errors;

thread_local! {
  static KEY_CACHE: RefCell<HashMap<&'static str, JsString>> = RefCell::new({
    let mut map = HashMap::new();
    for &key in &[
      "op", "x", "y", "width", "height", "fillStyle", "strokeStyle",
      "globalAlpha", "globalCompositeOperation", "imageSmoothingEnabled",
      "imageSmoothingQuality", "lineWidth", "lineCap", "lineJoin",
      "miterLimit", "path", "rule", "verb", "radius", "startAngle",
      "endAngle", "counterclockwise", "x0", "y0", "x1", "y1", "x2", "y2",
      "cpx", "cpy", "cp1x", "cp1y", "cp2x", "cp2y", "radiusX", "radiusY",
      "rotation", "dx", "dy", "dw", "dh", "sx", "sy", "sw", "sh",
      "bitmapSx", "bitmapSy", "bitmapSw", "bitmapSh", "sourceKind", "src",
      "draws", "commands", "transform", "a", "b", "c", "d", "e", "f",
      "color", "offset", "stops", "length"
    ] {
      map.insert(key, JsString::from(key));
    }
    map
  });
}

pub fn array_len(array: &JsObject, js: &mut JsContext) -> Result<usize> {
  let length = array
    .get(js_string!("length"), js)
    .map_err(boa_errors::js_error)?
    .to_number(js)
    .map_err(boa_errors::js_error)?;
  if !length.is_finite() || length < 0.0 {
    bail!("expected finite non-negative array length");
  }
  Ok(length as usize)
}

pub fn required_string(object: &JsObject, key: &'static str, js: &mut JsContext) -> Result<String> {
  let value = get_property(object, key, js)?;
  if value.is_null_or_undefined() {
    bail!("missing field `{key}`");
  }
  value_to_string(value, js)
}

pub fn optional_string(
  object: &JsObject,
  key: &'static str,
  js: &mut JsContext,
) -> Result<Option<String>> {
  let value = get_property(object, key, js)?;
  if value.is_null_or_undefined() {
    return Ok(None);
  }
  value_to_string(value, js).map(Some)
}

pub fn required_f32(object: &JsObject, key: &'static str, js: &mut JsContext) -> Result<f32> {
  let value = get_property(object, key, js)?;
  if value.is_null_or_undefined() {
    bail!("missing field `{key}`");
  }
  value_to_f32(value, key, js)
}

pub fn optional_f32(
  object: &JsObject,
  key: &'static str,
  js: &mut JsContext,
) -> Result<Option<f32>> {
  let value = get_property(object, key, js)?;
  if value.is_null_or_undefined() {
    return Ok(None);
  }
  value_to_f32(value, key, js).map(Some)
}

pub fn optional_usize(
  object: &JsObject,
  key: &'static str,
  js: &mut JsContext,
) -> Result<Option<usize>> {
  let Some(value) = optional_f32(object, key, js)? else {
    return Ok(None);
  };
  if value < 0.0 {
    bail!("field `{key}` must be non-negative");
  }
  Ok(Some(value as usize))
}

pub fn optional_bool(
  object: &JsObject,
  key: &'static str,
  js: &mut JsContext,
) -> Result<Option<bool>> {
  let value = get_property(object, key, js)?;
  if value.is_null_or_undefined() {
    return Ok(None);
  }
  if let Some(b) = value.as_boolean() {
    Ok(Some(b))
  } else {
    Ok(Some(value.to_boolean()))
  }
}

pub fn value_to_object(value: JsValue, what: &'static str) -> Result<JsObject> {
  value
    .as_object()
    .ok_or_else(|| anyhow!("expected `{what}` to be an object"))
}

pub fn get_property(object: &JsObject, key: &'static str, js: &mut JsContext) -> Result<JsValue> {
  let js_key = KEY_CACHE.with(|cache| {
    let mut map = cache.borrow_mut();
    map
      .entry(key)
      .or_insert_with(|| JsString::from(key))
      .clone()
  });
  object.get(js_key, js).map_err(boa_errors::js_error)
}

pub fn value_to_string(value: JsValue, js: &mut JsContext) -> Result<String> {
  if let Some(s) = value.as_string() {
    Ok(s.to_std_string_escaped())
  } else {
    Ok(
      value
        .to_string(js)
        .map_err(boa_errors::js_error)?
        .to_std_string_escaped(),
    )
  }
}

pub fn value_to_f32(value: JsValue, key: &'static str, js: &mut JsContext) -> Result<f32> {
  let number = if let Some(n) = value.as_number() {
    n
  } else {
    value.to_number(js).map_err(boa_errors::js_error)?
  };
  if !number.is_finite() {
    bail!("field `{key}` must be a finite number");
  }
  Ok(number as f32)
}

pub fn required_f32_array(
  object: &JsObject,
  key: &'static str,
  js: &mut JsContext,
) -> Result<Vec<f32>> {
  let value = get_property(object, key, js)?;
  let array = value_to_object(value, key)?;
  let len = array_len(&array, js)?;
  let mut values = Vec::with_capacity(len);

  for index in 0..len {
    let value = array.get(index as u32, js).map_err(boa_errors::js_error)?;
    values.push(value_to_f32(value, key, js)?);
  }

  Ok(values)
}
