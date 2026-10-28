use std::{cell::RefCell, fs, io::Cursor};

use anyhow::Result;
use boa_engine::{
  Context as JsContext, JsNativeError, JsResult, JsValue, js_string,
  object::{
    ObjectInitializer,
    builtins::{JsArray, JsArrayBuffer},
  },
  property::Attribute,
  string::JsString,
};
use web_audio_api::{
  AudioBuffer as WebAudioBuffer, AudioParam,
  context::{
    AudioContext as WebAudioContext, AudioContextOptions, AudioContextState, BaseAudioContext,
  },
  node::{
    AudioBufferSourceNode, AudioNode as WebAudioNode, AudioScheduledSourceNode, BiquadFilterNode,
    BiquadFilterType, GainNode, OscillatorNode, OscillatorType,
  },
};

use crate::{
  assets_utils::{decode_data_url, resolve_asset_path},
  boa_utils::array_len,
  native_bridge::NativeBridge,
};

thread_local! {
  static AUDIO_ENGINE: RefCell<AudioEngine> = RefCell::new(AudioEngine::default());
}

#[derive(Default)]
struct AudioEngine {
  contexts: Vec<Option<WebAudioContext>>,
  buffers: Vec<Option<WebAudioBuffer>>,
  nodes: Vec<Option<AudioNodeHandle>>,
}

enum AudioNodeHandle {
  BufferSource(AudioBufferSourceNode),
  Gain(GainNode),
  Oscillator(OscillatorNode),
  BiquadFilter(BiquadFilterNode),
}

pub fn register_natives(bridge: &mut NativeBridge) -> Result<()> {
  bridge.register_native(
    "__engine_audio_context_create",
    1,
    native_audio_context_create,
  )?;
  bridge.register_native(
    "__engine_audio_context_current_time",
    1,
    native_audio_context_current_time,
  )?;
  bridge.register_native(
    "__engine_audio_context_sample_rate",
    1,
    native_audio_context_sample_rate,
  )?;
  bridge.register_native(
    "__engine_audio_context_state",
    1,
    native_audio_context_state,
  )?;
  bridge.register_native(
    "__engine_audio_context_resume",
    1,
    native_audio_context_resume,
  )?;
  bridge.register_native(
    "__engine_audio_context_suspend",
    1,
    native_audio_context_suspend,
  )?;
  bridge.register_native(
    "__engine_audio_context_close",
    1,
    native_audio_context_close,
  )?;
  bridge.register_native("__engine_audio_create_gain", 1, native_audio_create_gain)?;
  bridge.register_native(
    "__engine_audio_create_buffer_source",
    1,
    native_audio_create_buffer_source,
  )?;
  bridge.register_native(
    "__engine_audio_create_oscillator",
    1,
    native_audio_create_oscillator,
  )?;
  bridge.register_native(
    "__engine_audio_create_biquad_filter",
    1,
    native_audio_create_biquad_filter,
  )?;
  bridge.register_native("__engine_audio_connect", 3, native_audio_connect)?;
  bridge.register_native("__engine_audio_disconnect", 1, native_audio_disconnect)?;
  bridge.register_native("__engine_audio_param_get", 2, native_audio_param_get)?;
  bridge.register_native("__engine_audio_param_set", 3, native_audio_param_set)?;
  bridge.register_native(
    "__engine_audio_param_set_at_time",
    4,
    native_audio_param_set_at_time,
  )?;
  bridge.register_native(
    "__engine_audio_param_linear_ramp_to_value_at_time",
    4,
    native_audio_param_linear_ramp_to_value_at_time,
  )?;
  bridge.register_native(
    "__engine_audio_param_exponential_ramp_to_value_at_time",
    4,
    native_audio_param_exponential_ramp_to_value_at_time,
  )?;
  bridge.register_native(
    "__engine_audio_buffer_source_set_buffer",
    2,
    native_audio_buffer_source_set_buffer,
  )?;
  bridge.register_native("__engine_audio_source_start", 2, native_audio_source_start)?;
  bridge.register_native(
    "__engine_audio_source_start_at_with_offset",
    3,
    native_audio_source_start_at_with_offset,
  )?;
  bridge.register_native(
    "__engine_audio_source_start_at_with_offset_and_duration",
    4,
    native_audio_source_start_at_with_offset_and_duration,
  )?;
  bridge.register_native("__engine_audio_source_stop", 2, native_audio_source_stop)?;
  bridge.register_native(
    "__engine_audio_oscillator_get_type",
    1,
    native_audio_oscillator_get_type,
  )?;
  bridge.register_native(
    "__engine_audio_oscillator_set_type",
    2,
    native_audio_oscillator_set_type,
  )?;
  bridge.register_native(
    "__engine_audio_biquad_get_type",
    1,
    native_audio_biquad_get_type,
  )?;
  bridge.register_native(
    "__engine_audio_biquad_set_type",
    2,
    native_audio_biquad_set_type,
  )?;
  bridge.register_native(
    "__engine_audio_decode_audio_data",
    2,
    native_audio_decode_audio_data,
  )?;
  bridge.register_native("__engine_audio_buffer_info", 1, native_audio_buffer_info)?;
  bridge.register_native(
    "__engine_audio_buffer_number_of_channels",
    1,
    native_audio_buffer_number_of_channels,
  )?;
  bridge.register_native(
    "__engine_audio_buffer_to_channels",
    1,
    native_audio_buffer_to_channels,
  )?;
  bridge.register_native(
    "__engine_audio_create_buffer_from_channels",
    3,
    native_audio_create_buffer_from_channels,
  )?;
  Ok(())
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

fn native_audio_context_create(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  let sample_rate = args
    .first()
    .and_then(JsValue::as_number)
    .filter(|value| value.is_finite() && *value > 0.0)
    .map(|value| value as f32);
  let context = WebAudioContext::try_new(AudioContextOptions {
    sample_rate,
    ..AudioContextOptions::default()
  })
  .map_err(|err| js_range_error(format!("failed to create AudioContext: {err}")))?;
  let id = AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    let id = engine.contexts.len();
    engine.contexts.push(Some(context));
    id
  });
  Ok(JsValue::from(id as i32))
}

fn native_audio_context_current_time(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  with_audio_context(number_arg_usize(args, 0), |context| {
    Ok(JsValue::from(context.current_time()))
  })
}

fn native_audio_context_sample_rate(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  with_audio_context(number_arg_usize(args, 0), |context| {
    Ok(JsValue::from(context.sample_rate() as f64))
  })
}

fn native_audio_context_state(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  with_audio_context(number_arg_usize(args, 0), |context| {
    Ok(JsValue::from(JsString::from(match context.state() {
      AudioContextState::Suspended => "suspended",
      AudioContextState::Running => "running",
      AudioContextState::Closed => "closed",
    })))
  })
}

fn native_audio_context_resume(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  with_audio_context(number_arg_usize(args, 0), |context| {
    context.resume_sync();
    Ok(JsValue::undefined())
  })
}

fn native_audio_context_suspend(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  with_audio_context(number_arg_usize(args, 0), |context| {
    context.suspend_sync();
    Ok(JsValue::undefined())
  })
}

fn native_audio_context_close(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  with_audio_context(number_arg_usize(args, 0), |context| {
    context.close_sync();
    Ok(JsValue::undefined())
  })
}

fn native_audio_create_gain(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  create_audio_node(number_arg_usize(args, 0), |context| {
    AudioNodeHandle::Gain(context.create_gain())
  })
}

fn native_audio_create_buffer_source(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  create_audio_node(number_arg_usize(args, 0), |context| {
    AudioNodeHandle::BufferSource(context.create_buffer_source())
  })
}

fn native_audio_create_oscillator(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  create_audio_node(number_arg_usize(args, 0), |context| {
    AudioNodeHandle::Oscillator(context.create_oscillator())
  })
}

fn native_audio_create_biquad_filter(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  create_audio_node(number_arg_usize(args, 0), |context| {
    AudioNodeHandle::BiquadFilter(context.create_biquad_filter())
  })
}

fn native_audio_connect(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  let context_id = number_arg_usize(args, 0);
  let source_id = number_arg_usize(args, 1);
  let dest_id = number_arg_usize(args, 2);

  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    let source = get_audio_node_handle(&engine, source_id)?;
    if dest_id == 0 {
      let destination = get_audio_context_ref(&engine, context_id)?.destination();
      connect_audio_node(source, &destination);
    } else {
      let destination = get_audio_node_handle(&engine, dest_id)?;
      with_audio_node_ref(destination, |destination| {
        connect_audio_node(source, destination)
      });
    }
    Ok(JsValue::undefined())
  })
}

fn native_audio_disconnect(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  let node_id = number_arg_usize(args, 0);
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    with_audio_node_ref(get_audio_node_handle(&engine, node_id)?, |node| {
      node.disconnect();
    });
    Ok(JsValue::undefined())
  })
}

fn native_audio_param_get(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let node_id = number_arg_usize(args, 0);
  let name = string_arg(args, 1, js)?;
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    let param = get_audio_param(get_audio_node_handle(&engine, node_id)?, &name)?;
    Ok(JsValue::from(param.value() as f64))
  })
}

fn native_audio_param_set(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let node_id = number_arg_usize(args, 0);
  let name = string_arg(args, 1, js)?;
  let value = native_number_arg(args, 2, js);
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    get_audio_param(get_audio_node_handle(&engine, node_id)?, &name)?.set_value(value);
    Ok(JsValue::undefined())
  })
}

fn native_audio_param_set_at_time(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  audio_param_event(args, js, |param, value, time| {
    param.set_value_at_time(value, time);
  })
}

fn native_audio_param_linear_ramp_to_value_at_time(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  audio_param_event(args, js, |param, value, time| {
    param.linear_ramp_to_value_at_time(value, time);
  })
}

fn native_audio_param_exponential_ramp_to_value_at_time(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  audio_param_event(args, js, |param, value, time| {
    param.exponential_ramp_to_value_at_time(value.max(0.000_001), time);
  })
}

fn native_audio_buffer_source_set_buffer(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  let node_id = number_arg_usize(args, 0);
  let buffer_id = number_arg_usize(args, 1);
  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    let buffer = get_audio_buffer_ref(&engine, buffer_id)?.clone();
    match get_audio_node_handle_mut(&mut engine, node_id)? {
      AudioNodeHandle::BufferSource(source) => source.set_buffer(buffer),
      _ => return Err(js_type_error("node is not an AudioBufferSourceNode")),
    }
    Ok(JsValue::undefined())
  })
}

fn native_audio_source_start(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let node_id = number_arg_usize(args, 0);
  let when = native_number_arg(args, 1, js) as f64;
  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    match get_audio_node_handle_mut(&mut engine, node_id)? {
      AudioNodeHandle::BufferSource(source) => {
        if when <= 0.0 {
          source.start();
        } else {
          source.start_at(when);
        }
      }
      AudioNodeHandle::Oscillator(source) => {
        if when <= 0.0 {
          source.start();
        } else {
          source.start_at(when);
        }
      }
      _ => return Err(js_type_error("node is not an AudioScheduledSourceNode")),
    }
    Ok(JsValue::undefined())
  })
}

fn native_audio_source_start_at_with_offset(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let node_id = number_arg_usize(args, 0);
  let when = native_number_arg(args, 1, js) as f64;
  let offset = native_number_arg(args, 2, js) as f64;
  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    match get_audio_node_handle_mut(&mut engine, node_id)? {
      AudioNodeHandle::BufferSource(source) => source.start_at_with_offset(when, offset),
      _ => return Err(js_type_error("node is not an AudioBufferSourceNode")),
    }
    Ok(JsValue::undefined())
  })
}

fn native_audio_source_start_at_with_offset_and_duration(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let node_id = number_arg_usize(args, 0);
  let when = native_number_arg(args, 1, js) as f64;
  let offset = native_number_arg(args, 2, js) as f64;
  let duration = native_number_arg(args, 3, js) as f64;
  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    match get_audio_node_handle_mut(&mut engine, node_id)? {
      AudioNodeHandle::BufferSource(source) => {
        source.start_at_with_offset_and_duration(when, offset, duration)
      }
      _ => return Err(js_type_error("node is not an AudioBufferSourceNode")),
    }
    Ok(JsValue::undefined())
  })
}

fn native_audio_source_stop(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let node_id = number_arg_usize(args, 0);
  let when = native_number_arg(args, 1, js) as f64;
  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    match get_audio_node_handle_mut(&mut engine, node_id)? {
      AudioNodeHandle::BufferSource(source) => {
        if when <= 0.0 {
          source.stop();
        } else {
          source.stop_at(when);
        }
      }
      AudioNodeHandle::Oscillator(source) => {
        if when <= 0.0 {
          source.stop();
        } else {
          source.stop_at(when);
        }
      }
      _ => return Err(js_type_error("node is not an AudioScheduledSourceNode")),
    }
    Ok(JsValue::undefined())
  })
}

fn native_audio_oscillator_get_type(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    match get_audio_node_handle(&engine, number_arg_usize(args, 0))? {
      AudioNodeHandle::Oscillator(oscillator) => Ok(JsValue::from(JsString::from(
        oscillator_type_name(oscillator.type_()),
      ))),
      _ => Err(js_type_error("node is not an OscillatorNode")),
    }
  })
}

fn native_audio_oscillator_set_type(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let value = string_arg(args, 1, js)?;
  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    match get_audio_node_handle_mut(&mut engine, number_arg_usize(args, 0))? {
      AudioNodeHandle::Oscillator(oscillator) => {
        oscillator.set_type(parse_oscillator_type(&value)?)
      }
      _ => return Err(js_type_error("node is not an OscillatorNode")),
    }
    Ok(JsValue::undefined())
  })
}

fn native_audio_biquad_get_type(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    match get_audio_node_handle(&engine, number_arg_usize(args, 0))? {
      AudioNodeHandle::BiquadFilter(filter) => Ok(JsValue::from(JsString::from(biquad_type_name(
        filter.type_(),
      )))),
      _ => Err(js_type_error("node is not a BiquadFilterNode")),
    }
  })
}

fn native_audio_biquad_set_type(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let value = string_arg(args, 1, js)?;
  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    match get_audio_node_handle_mut(&mut engine, number_arg_usize(args, 0))? {
      AudioNodeHandle::BiquadFilter(filter) => filter.set_type(parse_biquad_type(&value)?),
      _ => return Err(js_type_error("node is not a BiquadFilterNode")),
    }
    Ok(JsValue::undefined())
  })
}

fn native_audio_decode_audio_data(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let context_id = number_arg_usize(args, 0);
  let input = audio_decode_input(args.get(1).unwrap_or(&JsValue::undefined()), js)?;
  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    let buffer = get_audio_context_ref(&engine, context_id)?
      .decode_audio_data_sync(Cursor::new(input))
      .map_err(|err| js_range_error(format!("failed to decode audio data: {err}")))?;
    let id = engine.buffers.len();
    engine.buffers.push(Some(buffer));
    Ok(JsValue::from(id as i32))
  })
}

fn native_audio_buffer_info(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    let buffer = get_audio_buffer_ref(&engine, number_arg_usize(args, 0))?;
    let object = ObjectInitializer::new(js)
      .property(
        js_string!("length"),
        buffer.length() as i32,
        Attribute::all(),
      )
      .property(
        js_string!("sampleRate"),
        buffer.sample_rate() as f64,
        Attribute::all(),
      )
      .build();
    Ok(JsValue::from(object))
  })
}

fn native_audio_buffer_number_of_channels(
  _this: &JsValue,
  args: &[JsValue],
  _js: &mut JsContext,
) -> JsResult<JsValue> {
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    Ok(JsValue::from(
      get_audio_buffer_ref(&engine, number_arg_usize(args, 0))?.number_of_channels() as i32,
    ))
  })
}

fn native_audio_buffer_to_channels(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    let buffer = get_audio_buffer_ref(&engine, number_arg_usize(args, 0))?;
    let channels = (0..buffer.number_of_channels())
      .map(|channel| {
        JsValue::from(JsArray::from_iter(
          buffer
            .get_channel_data(channel)
            .iter()
            .copied()
            .map(|sample| JsValue::from(sample as f64)),
          js,
        ))
      })
      .collect::<Vec<_>>();
    Ok(JsValue::from(JsArray::from_iter(channels, js)))
  })
}

fn native_audio_create_buffer_from_channels(
  _this: &JsValue,
  args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let context_id = number_arg_usize(args, 0);
  let sample_rate = native_number_arg(args, 2, js);
  let channels_value = args.get(1).cloned().unwrap_or_default();
  let channels_object = channels_value
    .as_object()
    .ok_or_else(|| js_type_error("channels must be an array"))?;
  let channel_count = array_len(&channels_object, js).map_err(anyhow_to_js_error)?;

  let mut channel_data = Vec::with_capacity(channel_count);
  let mut length = 0;
  for channel_index in 0..channel_count {
    let channel = channels_object
      .get(channel_index as u32, js)
      .map_err(js_error_to_js_result)?;
    let channel_object = channel
      .as_object()
      .ok_or_else(|| js_type_error("channel data must be array-like"))?;
    let channel_len = array_len(&channel_object, js).map_err(anyhow_to_js_error)?;
    length = length.max(channel_len);
    let mut samples = Vec::with_capacity(channel_len);
    for sample_index in 0..channel_len {
      let sample = channel_object
        .get(sample_index as u32, js)
        .map_err(js_error_to_js_result)?
        .to_number(js)
        .map_err(js_error_to_js_result)?;
      samples.push(if sample.is_finite() {
        sample as f32
      } else {
        0.0
      });
    }
    channel_data.push(samples);
  }

  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    get_audio_context_ref(&engine, context_id)?;
    if channel_data.is_empty() {
      channel_data.push(vec![0.0; length.max(1)]);
    }
    for samples in &mut channel_data {
      samples.resize(length, 0.0);
    }
    let buffer = WebAudioBuffer::from(
      channel_data,
      if sample_rate > 0.0 {
        sample_rate
      } else {
        44_100.0
      },
    );
    let id = engine.buffers.len();
    engine.buffers.push(Some(buffer));
    Ok(JsValue::from(id as i32))
  })
}

fn create_audio_node(
  context_id: usize,
  create: impl FnOnce(&WebAudioContext) -> AudioNodeHandle,
) -> JsResult<JsValue> {
  AUDIO_ENGINE.with(|engine| {
    let mut engine = engine.borrow_mut();
    let node = {
      let context = get_audio_context_ref(&engine, context_id)?;
      create(context)
    };
    let id = engine.nodes.len().max(1);
    if engine.nodes.is_empty() {
      engine.nodes.push(None);
    }
    engine.nodes.push(Some(node));
    Ok(JsValue::from(id as i32))
  })
}

fn with_audio_context(
  context_id: usize,
  f: impl FnOnce(&WebAudioContext) -> JsResult<JsValue>,
) -> JsResult<JsValue> {
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    f(get_audio_context_ref(&engine, context_id)?)
  })
}

fn get_audio_context_ref(engine: &AudioEngine, id: usize) -> JsResult<&WebAudioContext> {
  engine
    .contexts
    .get(id)
    .and_then(Option::as_ref)
    .ok_or_else(|| js_range_error("invalid AudioContext id"))
}

fn get_audio_buffer_ref(engine: &AudioEngine, id: usize) -> JsResult<&WebAudioBuffer> {
  engine
    .buffers
    .get(id)
    .and_then(Option::as_ref)
    .ok_or_else(|| js_range_error("invalid AudioBuffer id"))
}

fn get_audio_node_handle(engine: &AudioEngine, id: usize) -> JsResult<&AudioNodeHandle> {
  engine
    .nodes
    .get(id)
    .and_then(Option::as_ref)
    .ok_or_else(|| js_range_error("invalid AudioNode id"))
}

fn get_audio_node_handle_mut(
  engine: &mut AudioEngine,
  id: usize,
) -> JsResult<&mut AudioNodeHandle> {
  engine
    .nodes
    .get_mut(id)
    .and_then(Option::as_mut)
    .ok_or_else(|| js_range_error("invalid AudioNode id"))
}

fn with_audio_node_ref(handle: &AudioNodeHandle, f: impl FnOnce(&dyn WebAudioNode)) {
  match handle {
    AudioNodeHandle::BufferSource(node) => f(node),
    AudioNodeHandle::Gain(node) => f(node),
    AudioNodeHandle::Oscillator(node) => f(node),
    AudioNodeHandle::BiquadFilter(node) => f(node),
  }
}

fn connect_audio_node(source: &AudioNodeHandle, destination: &dyn WebAudioNode) {
  with_audio_node_ref(source, |source| {
    source.connect(destination);
  });
}

fn get_audio_param<'a>(handle: &'a AudioNodeHandle, name: &str) -> JsResult<&'a AudioParam> {
  match (handle, name) {
    (AudioNodeHandle::Gain(node), "gain") => Ok(node.gain()),
    (AudioNodeHandle::BufferSource(node), "playbackRate") => Ok(node.playback_rate()),
    (AudioNodeHandle::BufferSource(node), "detune") => Ok(node.detune()),
    (AudioNodeHandle::Oscillator(node), "frequency") => Ok(node.frequency()),
    (AudioNodeHandle::Oscillator(node), "detune") => Ok(node.detune()),
    (AudioNodeHandle::BiquadFilter(node), "frequency") => Ok(node.frequency()),
    (AudioNodeHandle::BiquadFilter(node), "detune") => Ok(node.detune()),
    (AudioNodeHandle::BiquadFilter(node), "Q") => Ok(node.q()),
    (AudioNodeHandle::BiquadFilter(node), "gain") => Ok(node.gain()),
    _ => Err(js_type_error(format!("unsupported AudioParam `{name}`"))),
  }
}

fn audio_param_event(
  args: &[JsValue],
  js: &mut JsContext,
  f: impl FnOnce(&AudioParam, f32, f64),
) -> JsResult<JsValue> {
  let node_id = number_arg_usize(args, 0);
  let name = string_arg(args, 1, js)?;
  let value = native_number_arg(args, 2, js);
  let time = native_number_arg(args, 3, js) as f64;
  AUDIO_ENGINE.with(|engine| {
    let engine = engine.borrow();
    f(
      get_audio_param(get_audio_node_handle(&engine, node_id)?, &name)?,
      value,
      time,
    );
    Ok(JsValue::undefined())
  })
}

fn audio_decode_input(value: &JsValue, js: &mut JsContext) -> JsResult<Vec<u8>> {
  if let Some(path) = value.as_string() {
    let path = path.to_std_string_escaped();
    if let Some(bytes) = decode_data_url(&path).map_err(anyhow_to_js_error)? {
      return Ok(bytes);
    }
    return fs::read(resolve_asset_path(&path))
      .map_err(|err| js_range_error(format!("failed to read audio file `{path}`: {err}")));
  }

  if let Some(object) = value.as_object() {
    if let Ok(buffer) = JsArrayBuffer::from_object(object.clone()) {
      let data = buffer
        .data()
        .ok_or_else(|| js_type_error("ArrayBuffer is detached"))?;
      return Ok(data.to_vec());
    }

    if let Ok(array_buffer) = object.get(js_string!("buffer"), js) {
      if let Some(buffer_object) = array_buffer.as_object() {
        if let Ok(buffer) = JsArrayBuffer::from_object(buffer_object.clone()) {
          let data = buffer
            .data()
            .ok_or_else(|| js_type_error("ArrayBuffer is detached"))?;
          let byte_offset = object
            .get(js_string!("byteOffset"), js)
            .ok()
            .and_then(|v| v.as_number())
            .unwrap_or(0.0) as usize;
          let byte_length = object
            .get(js_string!("byteLength"), js)
            .ok()
            .and_then(|v| v.as_number())
            .map(|v| v as usize)
            .unwrap_or(data.len().saturating_sub(byte_offset));
          let end = byte_offset.saturating_add(byte_length).min(data.len());
          return Ok(data[byte_offset.min(data.len())..end].to_vec());
        }
      }
    }

    let len = array_len(&object, js).map_err(anyhow_to_js_error)?;
    let mut bytes = Vec::with_capacity(len);
    for index in 0..len {
      let value = object
        .get(index as u32, js)
        .map_err(js_error_to_js_result)?;
      let byte = value.to_number(js).map_err(js_error_to_js_result)?;
      bytes.push(byte.clamp(0.0, 255.0) as u8);
    }
    return Ok(bytes);
  }

  Err(js_type_error(
    "decodeAudioData() expects a path string, ArrayBuffer, typed array, or byte array",
  ))
}

fn number_arg_usize(args: &[JsValue], index: usize) -> usize {
  args
    .get(index)
    .and_then(JsValue::as_number)
    .filter(|value| value.is_finite() && *value >= 0.0)
    .unwrap_or(0.0) as usize
}

fn string_arg(args: &[JsValue], index: usize, js: &mut JsContext) -> JsResult<String> {
  args
    .get(index)
    .unwrap_or(&JsValue::undefined())
    .to_string(js)
    .map(|value| value.to_std_string_escaped())
}

fn parse_oscillator_type(value: &str) -> JsResult<OscillatorType> {
  match value {
    "sine" => Ok(OscillatorType::Sine),
    "square" => Ok(OscillatorType::Square),
    "sawtooth" => Ok(OscillatorType::Sawtooth),
    "triangle" => Ok(OscillatorType::Triangle),
    _ => Err(js_type_error(format!(
      "unsupported OscillatorNode type `{value}`"
    ))),
  }
}

fn oscillator_type_name(value: OscillatorType) -> &'static str {
  match value {
    OscillatorType::Sine => "sine",
    OscillatorType::Square => "square",
    OscillatorType::Sawtooth => "sawtooth",
    OscillatorType::Triangle => "triangle",
    OscillatorType::Custom => "custom",
  }
}

fn parse_biquad_type(value: &str) -> JsResult<BiquadFilterType> {
  match value {
    "lowpass" => Ok(BiquadFilterType::Lowpass),
    "highpass" => Ok(BiquadFilterType::Highpass),
    "bandpass" => Ok(BiquadFilterType::Bandpass),
    "notch" => Ok(BiquadFilterType::Notch),
    "allpass" => Ok(BiquadFilterType::Allpass),
    "peaking" => Ok(BiquadFilterType::Peaking),
    "lowshelf" => Ok(BiquadFilterType::Lowshelf),
    "highshelf" => Ok(BiquadFilterType::Highshelf),
    _ => Err(js_type_error(format!(
      "unsupported BiquadFilterNode type `{value}`"
    ))),
  }
}

fn biquad_type_name(value: BiquadFilterType) -> &'static str {
  match value {
    BiquadFilterType::Lowpass => "lowpass",
    BiquadFilterType::Highpass => "highpass",
    BiquadFilterType::Bandpass => "bandpass",
    BiquadFilterType::Notch => "notch",
    BiquadFilterType::Allpass => "allpass",
    BiquadFilterType::Peaking => "peaking",
    BiquadFilterType::Lowshelf => "lowshelf",
    BiquadFilterType::Highshelf => "highshelf",
  }
}

fn js_type_error(message: impl Into<String>) -> boa_engine::JsError {
  JsNativeError::typ().with_message(message.into()).into()
}

fn js_range_error(message: impl Into<String>) -> boa_engine::JsError {
  JsNativeError::range().with_message(message.into()).into()
}

fn js_error_to_js_result(err: boa_engine::JsError) -> boa_engine::JsError {
  err
}

fn anyhow_to_js_error(err: anyhow::Error) -> boa_engine::JsError {
  js_type_error(err.to_string())
}
