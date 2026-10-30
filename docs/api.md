# API

## Globals

Available via `globaThis`

| Global | Notes |
| --- | --- |
| `canvas` | A single pre-created native canvas is available on `globalThis` |
| `exit()` | Closes the application. |

## Main loop

There is no `requestAnimationFrame`. Listen for the `tick` event, which runs once per frame (capped at 60fps).

```js
canvas.addEventListener("tick", (event) => {
  // event.deltaTime: 0..1, where 1 is one frame at 60fps (or slower)
  // event.elapsedMS: real milliseconds since the previous tick
  // event.lastTime:  milliseconds since the engine started
  x += speed * event.deltaTime;
});
```

`deltaTime` is capped at 1, so on frames slower than 60fps motion based on it runs slower than real time. Use `elapsedMS` if you need exact time.

## Not available

- DOM: no `document`, elements or CSS. Create canvases with `new OffscreenCanvas(w, h)`.
- Timers: no `setTimeout`/`setInterval`. Use `tick`.
- Network: `fetch()` reads local files and `data:` URLs only (no `http(s)`), and the response is UTF-8 text.
- `performance.now()` is `Date.now()`, so it has millisecond precision.

## Events

`keydown`/`keyup` fire on `globalThis`. Pointer events and `resize` fire on `canvas`. `resize` also fires on `globalThis`.
`PointerEvent` and `KeyboardEvent` are aliases of the base `Event`.

## Audio

`AudioContext` and the related Web Audio classes exist only when the `audio` Cargo feature is enabled.

## Console

`console.log/info/debug/warn/error` print to the terminal. They are on in the CLI and, in the library, in debug builds unless you call `with_console(true)`.

## Rust options

`EngineOptions` configures the single native window:

```rust
let options = EngineOptions::default()
    .with_canvas_size(640, 360)
    .with_resizable(true)
    .with_scale_mode("responsive")
    .with_console(true);
```

`scaleMode` controls how the canvas is shown in the window:

| Mode | Behaviour |
| --- | --- |
| `"showAll"` (default) | Keep aspect ratio, show everything, letterbox if needed. |
| `"exactFit"` | Stretch to fill the window, aspect ratio may distort. |
| `"noBorder"` | Keep aspect ratio, fill the window, edges may be cropped. |
| `"noScale"` | No scaling, centered. |
| `"responsive"` | Resize the canvas to match the window. |
