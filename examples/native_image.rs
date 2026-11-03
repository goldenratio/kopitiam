use std::{fs, path::PathBuf};

use anyhow::Result;
use boa_engine::object::builtins::JsArray;
use kopitiam::{EngineOptions, JsContext, JsNativeError, JsResult, JsValue};

const GAME_JS: &str = r##"
globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});

const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");
const tileBytes = new Uint8Array(nativeTilePngBytes());
const tileBlob = new Blob([tileBytes], { type: "image/png" });
let tile = null;
let angle = 0;

createImageBitmap(tileBlob).then((bitmap) => {
  tile = bitmap;
});

canvas.addEventListener("tick", () => {
  angle += 0.03;
  const panelX = 48;
  const panelY = 48;
  const panelWidth = canvas.width - panelX * 2;
  const panelHeight = canvas.height - panelY * 2;

  ctx.fillStyle = "#0f172a";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = "#1f2937";
  ctx.fillRect(panelX, panelY, panelWidth, panelHeight);

  if (!tile) {
    return;
  }

  ctx.drawImage(tile, 80, 80, 96, 96);
  ctx.drawImage(tile, 208, 80, 144, 144);

  ctx.save();
  ctx.translate(500, 170);
  ctx.rotate(angle);
  ctx.drawImage(tile, -72, -72, 144, 144);
  ctx.restore();

  ctx.drawImage(tile, 4, 4, 16, 16, 608, 88, 128, 128);

  for (let y = 292; y < panelY + panelHeight - 14; y += 24) {
    for (let x = 80; x < panelX + panelWidth - 16; x += 24) {
      ctx.drawImage(tile, x, y, 24, 24);
    }
  }
});
"##;

fn main() -> Result<()> {
  kopitiam::run_script(
    GAME_JS,
    EngineOptions::default().with_native_function("nativeTilePngBytes", native_tile_png_bytes),
  )
}

fn native_tile_png_bytes(
  _this: &JsValue,
  _args: &[JsValue],
  js: &mut JsContext,
) -> JsResult<JsValue> {
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples")
    .join("tile_0007.png");

  let bytes = fs::read(&path).map_err(|err| -> boa_engine::JsError {
    JsNativeError::error()
      .with_message(format!("failed to read `{}`: {err}", path.display()))
      .into()
  })?;

  Ok(JsValue::from(JsArray::from_iter(
    bytes.into_iter().map(|byte| JsValue::from(byte as i32)),
    js,
  )))
}
