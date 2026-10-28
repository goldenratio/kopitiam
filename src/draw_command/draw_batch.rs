use std::cell::RefCell;

use anyhow::Result;
use boa_engine::{Context as JsContext, JsNativeError, JsResult, JsValue};

use crate::native_bridge::NativeBridge;

thread_local! {
  static DRAW_BATCHES: RefCell<Vec<Vec<f32>>> = const { RefCell::new(Vec::new()) };
  static ACTIVE_DRAW_BATCH: RefCell<Option<usize>> = const { RefCell::new(None) };
}

pub(crate) fn register_natives(bridge: &mut NativeBridge) -> Result<()> {
  bridge.register_native("__engine_create_draw_batch", 4, native_create_draw_batch)?;
  bridge.register_native("__engine_append_draw_batch", 5, native_append_draw_batch)?;
  bridge.register_native(
    "__engine_append_active_draw_batch",
    4,
    native_append_active_draw_batch,
  )
}

pub(crate) fn take_draw_batch(id: usize) -> Option<Vec<f32>> {
  DRAW_BATCHES.with(|batches| {
    let mut batches = batches.borrow_mut();
    if id >= batches.len() {
      None
    } else {
      Some(std::mem::take(&mut batches[id]))
    }
  })
}

pub(crate) fn clear_draw_batches() {
  DRAW_BATCHES.with(|batches| batches.borrow_mut().clear());
  ACTIVE_DRAW_BATCH.with(|active| *active.borrow_mut() = None);
}

fn native_create_draw_batch(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let draw = [
    native_number_arg(args, 0, js),
    native_number_arg(args, 1, js),
    native_number_arg(args, 2, js),
    native_number_arg(args, 3, js),
  ];

  let id = DRAW_BATCHES.with(|batches| {
    let mut batches = batches.borrow_mut();
    let id = batches.len();
    batches.push(Vec::from(draw));
    id
  });
  ACTIVE_DRAW_BATCH.with(|active| *active.borrow_mut() = Some(id));

  Ok(JsValue::from(id as i32))
}

fn native_append_draw_batch(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let started = crate::profile::is_enabled().then(std::time::Instant::now);
  let id = native_number_arg(args, 0, js) as usize;
  let draw = [
    native_number_arg(args, 1, js),
    native_number_arg(args, 2, js),
    native_number_arg(args, 3, js),
    native_number_arg(args, 4, js),
  ];

  DRAW_BATCHES.with(|batches| {
    let result = if let Some(batch) = batches.borrow_mut().get_mut(id) {
      batch.extend_from_slice(&draw);
      Ok(JsValue::undefined())
    } else {
      Err(
        JsNativeError::range()
          .with_message("invalid draw batch id")
          .into(),
      )
    };
    if let Some(started) = started {
      crate::profile::record_native_batch_append(started.elapsed());
    }
    result
  })
}

fn native_append_active_draw_batch(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let started = crate::profile::is_enabled().then(std::time::Instant::now);
  let draw = [
    native_number_arg(args, 0, js),
    native_number_arg(args, 1, js),
    native_number_arg(args, 2, js),
    native_number_arg(args, 3, js),
  ];

  let Some(id) = ACTIVE_DRAW_BATCH.with(|active| *active.borrow()) else {
    return Err(
      JsNativeError::range()
        .with_message("no active draw batch")
        .into(),
    );
  };

  DRAW_BATCHES.with(|batches| {
    let result = if let Some(batch) = batches.borrow_mut().get_mut(id) {
      batch.extend_from_slice(&draw);
      Ok(JsValue::undefined())
    } else {
      Err(
        JsNativeError::range()
          .with_message("invalid active draw batch")
          .into(),
      )
    };
    if let Some(started) = started {
      crate::profile::record_native_batch_append(started.elapsed());
    }
    result
  })
}

fn native_number_arg(args: &[JsValue], index: usize, js: &mut JsContext) -> f32 {
  let number = args
    .get(index)
    .and_then(JsValue::as_number)
    .or_else(|| args.get(index).and_then(|value| value.to_number(js).ok()))
    .unwrap_or(0.0);
  if number.is_finite() {
    number as f32
  } else {
    0.0
  }
}
