# kopitiam

Kopitiam is a lightweight runtime that runs JavaScript Canvas 2D as standalone desktop app.

## Library Usage

Add the engine crate to your game project:

```toml
[dependencies]
anyhow = "1"
kopitiam = { git = "https://codeberg.org/goldenratio/kopitiam.git" }
```
Basic example usage demo: https://codeberg.org/goldenratio/kopitiam-example

Embed your game script:

```rust
use anyhow::Result;

const GAME_JS: &str = include_str!("game.js");

fn main() -> Result<()> {
    kopitiam::run_script(GAME_JS)
}
```

To configure the host runtime:

```rust
use anyhow::Result;
use kopitiam::{EngineOptions, run_script_with_options};

const GAME_JS: &str = include_str!("game.js");

fn main() -> Result<()> {
    let options = EngineOptions::default()
        .with_title("My Game")
        .with_canvas_size(960, 540)
        .with_resizable(true);

    run_script_with_options(GAME_JS, options)
}
```

For tooling or development runners, scripts can also be loaded from disk:

```rust
fn main() -> anyhow::Result<()> {
    kopitiam::run_script_file("game.js")
}
```

Embedders can expose host functions to JavaScript:

```rust
use kopitiam::{EngineOptions, JsContext, JsResult, JsValue};

fn native_ping(_this: &JsValue, args: &[JsValue], _js: &mut JsContext) -> JsResult<JsValue> {
    let value = args.first().and_then(JsValue::as_number).unwrap_or(0.0);
    Ok(JsValue::from(value + 1.0))
}

fn main() -> anyhow::Result<()> {
    kopitiam::run_script_with_options(
        "console.log(nativePing(41));",
        EngineOptions::default().with_native_function("nativePing", native_ping),
    )
}
```

## JavaScript Entry Point

The embedder creates one native window/canvas through `EngineOptions`. JavaScript
uses the pre-created `globalThis.canvas`, gets a 2D canvas context, and listens
for `tick` events:

```js
const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");
let x = 0;
let vx = 260;

canvas.addEventListener("tick", (event) => {
  const dt = Math.min(event.deltaTime / 1000, 1);

  x += vx * dt;
  if (x < 0 || x + 80 > canvas.width) {
    vx *= -1;
  }

  ctx.fillStyle = "#101820";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = "#e53935";
  ctx.fillRect(x, 220, 80, 80);
});
```

Supported browser-style APIs include:

- `globalThis.canvas`
- `exit()`
- `CanvasRenderingContext2D`
- Canvas `globalCompositeOperation` blend/composite modes
- `Canvas`
- `OffscreenCanvas`
- `Image`
- `ImageBitmap`
- `createImageBitmap(...)`
- `DOMMatrix`
- `AudioContext`
- pointer, wheel, keyboard, and `tick` events
- `console.log/info/debug/warn/error` in debug builds
- JavaScript `Promise`, `.then(...)`, and `async`/`await`

`requestAnimationFrame` is intentionally not part of the public runtime. Use the
engine `tick` event instead.

## Window Options

`EngineOptions` configures the single native window/canvas:

- `title`: window title
- `width`: logical canvas width
- `height`: logical canvas height
- `resizable`: lets the user resize the desktop window
- `scaleMode`: Flash-style stage scaling mode, defaults to `"showAll"`

The canvas keeps its original logical size and the window presents it using the
selected stage mode:

- `"exactFit"`: stretch the canvas to fill the window; aspect ratio may distort.
- `"showAll"`: preserve aspect ratio and show the whole canvas with letterbox/pillarbox space. This is the default.
- `"noBorder"`: preserve aspect ratio and fill the whole window; edges may be cropped.
- `"noScale"`: keep the canvas unscaled and centered.
- `"responsive"`: resize the canvas logical size to match the window.

Resizable windows dispatch browser-style `resize` events on the canvas and on
`globalThis`. The event dimensions describe the window size. For the Flash-style
stage modes, `canvas.width` and `canvas.height` remain the logical stage size.
For `"responsive"`, the canvas size changes with the window:

```rust
let options = EngineOptions::default()
    .with_canvas_size(640, 360)
    .with_resizable(true)
    .with_scale_mode("responsive");
```

```js
canvas.addEventListener("resize", (event) => {
  console.log(event.width, event.height);
});

addEventListener("resize", (event) => {
  console.log(event.innerWidth, event.innerHeight);
});
```
## Running Examples

Run a JS example through the CLI runner:

```sh
cargo run --release -- examples/basic.js
```

Run the Rust embedding example:

```sh
cargo run --release --example embed_game
```

Run the bunny benchmark with profiling:

```sh
CANVAS_PROFILE=1 cargo run --release -- examples/bunny_benchmark.js
```
