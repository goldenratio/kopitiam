use anyhow::Result;
use kopitiam::{EngineOptions, JsContext, JsResult, JsValue};

const GAME_JS: &str = concat!(
  include_str!("basic.js"),
  "\nconsole.log('nativeHostPing returned', nativeHostPing(41));\n"
);

fn main() -> Result<()> {
  kopitiam::run_script(
    GAME_JS,
    EngineOptions::default().with_native_function("nativeHostPing", native_host_ping),
  )
}

fn native_host_ping(_this: &JsValue, args: &[JsValue], _js: &mut JsContext) -> JsResult<JsValue> {
  let value = args.first().and_then(JsValue::as_number).unwrap_or(0.0);
  println!("nativeHostPing called from JavaScript with {value}");
  Ok(JsValue::from(value + 1.0))
}
