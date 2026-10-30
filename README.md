# kopitiam

Kopitiam is a lightweight runtime that runs JavaScript Canvas 2D as standalone desktop app.

### Docs
- [API](./docs/api.md)
- Basic Javascript Example [./examples/basic.js](./examples/basic.js)
- Standalone App Example: https://codeberg.org/goldenratio/kopitiam-example

## CLI Usage

Simply point .js script to the binary,

```console
kopitiam ./game.js --size=900x600
```

## Library Usage

It's used to create standalone executable. Add the engine crate to your game project:

```toml
[dependencies]
anyhow = "1"
kopitiam = { git = "https://codeberg.org/goldenratio/kopitiam.git" }
```

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

## Running Examples from this Repo


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
