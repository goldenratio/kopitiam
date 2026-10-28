use anyhow::{Result, anyhow};
use boa_engine::{Context as JsContext, JsNativeError};

pub trait AnyhowJsExt<T> {
  fn context_js(self, js: &mut JsContext, message: &'static str) -> Result<T>;
}

impl<T> AnyhowJsExt<T> for boa_engine::JsResult<T> {
  fn context_js(self, js: &mut JsContext, message: &'static str) -> Result<T> {
    self.map_err(|err| {
      let pretty = err.to_string();
      let display = if let Some(message) = js_error_message(&pretty) {
        format!("{message}\n{pretty}")
      } else {
        match err.try_native(js) {
          Ok(native) => {
            if let Some(message) = js_error_message(native.message()) {
              format!("{message}\n{pretty}")
            } else {
              let native_message = native.to_string();
              if native_message == pretty {
                native_message
              } else if pretty.starts_with(&format!("{native_message}\n")) {
                pretty
              } else {
                format!("{native_message}\n{pretty}")
              }
            }
          }
          Err(_) => pretty,
        }
      };

      anyhow!("{message}\n{display}")
    })
  }
}

pub fn js_error(err: boa_engine::JsError) -> anyhow::Error {
  if let Some(native) = err.as_native() {
    return js_native_error(native);
  }

  let message = format!("{err:?}");
  js_error_message(&message).unwrap_or_else(|| anyhow!("{message}"))
}

fn js_native_error(err: &JsNativeError) -> anyhow::Error {
  if let Some(message) = js_error_message(err.message()) {
    return message;
  }

  anyhow!("{:?}: {}", err.kind(), err.message())
}

fn js_error_message(message: &str) -> Option<anyhow::Error> {
  if message.contains("__engine_audio_context_create is not defined") {
    return Some(anyhow!(
      "AudioContext is not available. You can enable `audio` feature by: \
       Rebuild or run with `--features audio` to enable Web Audio support."
    ));
  }

  None
}
