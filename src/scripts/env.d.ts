type ScaleMode = "responsive" | "exactFit" | "showAll" | "noBorder" | "noScale";
type EngineLineCap = "butt" | "round" | "square";
type EngineLineJoin = "round" | "bevel" | "miter";
type EngineImageSmoothingQuality = "low" | "medium" | "high";
type EngineCompositeOperation =
  | "source-over"
  | "source-in"
  | "source-out"
  | "source-atop"
  | "destination-over"
  | "destination-in"
  | "destination-out"
  | "destination-atop"
  | "lighter"
  | "copy"
  | "xor"
  | "multiply"
  | "screen"
  | "overlay"
  | "darken"
  | "lighten"
  | "color-dodge"
  | "color-burn"
  | "hard-light"
  | "soft-light"
  | "difference"
  | "exclusion"
  | "hue"
  | "saturation"
  | "color"
  | "luminosity";
type EngineFillRule = "nonzero" | "evenodd";
type EngineAudioContextState = "suspended" | "running" | "closed";
type EngineOscillatorType = "sine" | "square" | "sawtooth" | "triangle" | "custom";
type EngineBiquadFilterType =
  | "lowpass"
  | "highpass"
  | "bandpass"
  | "notch"
  | "allpass"
  | "peaking"
  | "lowshelf"
  | "highshelf";

interface EngineWindowConfig {
  title: string;
  width: number;
  height: number;
  resizable?: boolean;
  fullscreen?: boolean;
  scaleMode?: string;
}

interface EngineConsoleMessage {
  level: string;
  message: string;
}

interface TickEvent {
  readonly type: "tick";
  /**
   * time in milliseconds of this tick, since engine start
   */
  readonly lastTime: number;
  /**
   * Scalar representing the delta time factor.
   * This is a dimensionless value representing the fraction of a frame at the target framerate.
   * At 60 FPS, this value is typically around 1.0.
   *
   * This is NOT in milliseconds - it's a scalar multiplier for frame-independent animations.
  */
  readonly deltaTime: number;
  /**
   * milliseconds elapsed from last update.
   * Ideally value should be 16.66 ms
   */
  readonly elapsedMS: number;
  readonly target: unknown;
  readonly currentTarget: unknown;
}

interface CanvasRenderingContext2DSettings {
  colorSpace?: string;
  willReadFrequently?: boolean;
  alpha?: boolean;
  desynchronized?: boolean;
}

interface CanvasRenderingContext2DState {
  fillStyle: CanvasPaintStyle;
  strokeStyle: CanvasPaintStyle;
  globalAlpha: number;
  globalCompositeOperation: EngineCompositeOperation | string;
  lineWidth: number;
  lineCap: EngineLineCap | string;
  lineJoin: EngineLineJoin | string;
  miterLimit: number;
  lineDashOffset: number;
  imageSmoothingEnabled: boolean;
  imageSmoothingQuality: EngineImageSmoothingQuality | string;
  font: string;
  textAlign: string;
  textBaseline: string;
  direction: string;
  filter: string;
  shadowBlur: number;
  shadowColor: string;
  shadowOffsetX: number;
  shadowOffsetY: number;
  lineDash: number[];
  transformA: number;
  transformB: number;
  transformC: number;
  transformD: number;
  transformE: number;
  transformF: number;
  transformIsIdentity: boolean;
}

interface CanvasGradient {
  addColorStop(offset: number, color: string): void;
}

interface CanvasPattern {
  setTransform(transform?: any): void;
}

type CanvasPaintStyle = string | CanvasGradient | CanvasPattern;

interface ImageConstructor {
  new(width?: number, height?: number): import("./image").Image;
}

interface ImageData {
  data: Uint8ClampedArray;
  width: number;
  height: number;
  colorSpace: string;
  colorType: string;
  bytesPerPixel: number;
}

interface ImageDataConstructor {
  new(width: number, height: number, settings?: Record<string, unknown>): ImageData;
  new(data: Uint8ClampedArray | Uint8Array, width: number, height?: number, settings?: Record<string, unknown>): ImageData;
  new(imageData: ImageData): ImageData;
}

type VerbType =
  | "closePath"
  | "moveTo"
  | "lineTo"
  | "bezierCurveTo"
  | "quadraticCurveTo"
  | "rect"
  | "arc"
  | "arcTo"
  | "ellipse";

interface CanvasPathSegment {
  readonly verb: VerbType;
  x?: number;
  y?: number;
  cp1x?: number;
  cp1y?: number;
  cp2x?: number;
  cp2y?: number;
  cpx?: number;
  cpy?: number;
  x1?: number;
  y1?: number;
  x2?: number;
  y2?: number;
  width?: number;
  height?: number;
  radius?: number;
  radiusX?: number;
  radiusY?: number;
  rotation?: number;
  startAngle?: number;
  endAngle?: number;
  counterclockwise?: boolean;
}

interface Path2D {
  addPath(path: Path2D, transform?: any): void;
  closePath(): void;
  moveTo(x: number, y: number): void;
  lineTo(x: number, y: number): void;
  bezierCurveTo(cp1x: number, cp1y: number, cp2x: number, cp2y: number, x: number, y: number): void;
  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void;
  rect(x: number, y: number, width: number, height: number): void;
  roundRect(x: number, y: number, width: number, height: number, radii?: any): void;
  arc(x: number, y: number, radius: number, startAngle: number, endAngle: number, counterclockwise?: boolean): void;
  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void;
  ellipse(x: number, y: number, radiusX: number, radiusY: number, rotation: number, startAngle: number, endAngle: number, counterclockwise?: boolean): void;
}

interface Path2DConstructor {
  new(path?: Path2D | string): Path2D;
}

type DrawOperationType =
  | "cacheCanvasBitmap"
  | "drawImageBatch"
  | "putImageData"
  | "clipPath"
  | "clearRect"
  | "fillRect"
  | "drawImage"
  | "save"
  | "restore"
  | "translate"
  | "transform"
  | "setTransform"
  | "resetTransform"
  | "rotate"
  | "scale"
  | "exit"
  | "fillPath"
  | "strokePath"
  | "strokeRect"
  | "fillCircle"
  | "strokeLine";

interface CanvasDrawCommand {
  readonly op: DrawOperationType;
  path?: CanvasPathSegment[];
  sourceKind?: string;
  src?: string;
  imageSource?: EngineImageBitmapSource;
  drawStateVersion?: number;
  drawBatchId?: number;
  draws?: number[];
  commands?: CanvasDrawCommand[];
  [key: string]: unknown;
}

type CanvasImageBatchCommand = CanvasDrawCommand & {
  op: "drawImageBatch";
  sourceKind: string;
  src: string;
  globalAlpha: number;
  globalCompositeOperation: string;
  imageSmoothingEnabled: boolean;
  imageSmoothingQuality?: string;
  drawBatchId?: number;
  draws?: number[];
};

interface EngineCanvas {
  width: number;
  height: number;
  getContext(kind: "2d", settings?: CanvasRenderingContext2DSettings): CanvasRenderingContext2D | null;
  addEventListener(type: "tick", listener: (event: TickEvent) => void): void;
  addEventListener(type: string, listener: (event: unknown) => void): void;
  removeEventListener(type: string, listener: (event: unknown) => void): void;
  dispatchEvent(event: unknown): boolean;
}

interface CanvasRenderingContext2D {
  canvas: import("./canvas").Canvas | import("./canvas").OffscreenCanvas;
  fillStyle: CanvasPaintStyle;
  strokeStyle: CanvasPaintStyle;
  globalAlpha: number;
  globalCompositeOperation: EngineCompositeOperation | string;
  lineWidth: number;
  lineCap: EngineLineCap | string;
  lineJoin: EngineLineJoin | string;
  miterLimit: number;
  lineDashOffset: number;
  imageSmoothingEnabled: boolean;
  imageSmoothingQuality: EngineImageSmoothingQuality | string;
  font: string;
  textAlign: string;
  textBaseline: string;
  direction: string;
  filter: string;
  shadowBlur: number;
  shadowColor: string;
  shadowOffsetX: number;
  shadowOffsetY: number;
  save(): void;
  restore(): void;
  reset(): void;
  translate(x: number, y: number): void;
  rotate(angle: number): void;
  scale(x: number, y?: number): void;
  transform(a: number, b: number, c: number, d: number, e: number, f: number): void;
  setTransform(a?: number | object, b?: number, c?: number, d?: number, e?: number, f?: number): void;
  resetTransform(): void;
  getTransform(): import("./geometry").DOMMatrix;
  beginPath(): void;
  closePath(): void;
  moveTo(x: number, y: number): void;
  lineTo(x: number, y: number): void;
  bezierCurveTo(cp1x: number, cp1y: number, cp2x: number, cp2y: number, x: number, y: number): void;
  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void;
  rect(x: number, y: number, width: number, height: number): void;
  roundRect(x: number, y: number, width: number, height: number, radii?: any): void;
  arc(x: number, y: number, radius: number, startAngle: number, endAngle: number, counterclockwise?: boolean): void;
  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void;
  ellipse(x: number, y: number, radiusX: number, radiusY: number, rotation: number, startAngle: number, endAngle: number, counterclockwise?: boolean): void;
  fill(rule?: EngineFillRule | string): void;
  fill(path: Path2D, rule?: EngineFillRule | string): void;
  stroke(path?: Path2D): void;
  clip(rule?: EngineFillRule | string): void;
  clip(path: Path2D, rule?: EngineFillRule | string): void;
  createLinearGradient(x0: number, y0: number, x1: number, y1: number): CanvasGradient;
  createRadialGradient(x0: number, y0: number, r0: number, x1: number, y1: number, r1: number): CanvasGradient;
  createConicGradient(startAngle: number, x: number, y: number): CanvasGradient;
  createPattern(image: EngineImageBitmapSource, repetition?: string | null): CanvasPattern | null;
  createImageData(width: number, height: number, settings?: Record<string, unknown>): ImageData;
  createImageData(imageData: ImageData): ImageData;
  getImageData(x: number, y: number, width: number, height: number, settings?: Record<string, unknown>): ImageData;
  putImageData(imageData: ImageData, dx: number, dy: number): void;
  putImageData(imageData: ImageData, dx: number, dy: number, dirtyX: number, dirtyY: number, dirtyWidth: number, dirtyHeight: number): void;
  clearRect(x: number, y: number, width: number, height: number): void;
  fillRect(x: number, y: number, width: number, height: number): void;
  strokeRect(x: number, y: number, width: number, height: number): void;
  measureText(text: string): { width: number };
  drawImage(image: EngineImageBitmapSource, ...args: number[]): void;
  drawImageBatch(image: EngineImageBitmapSource, draws: Iterable<number> | ArrayLike<number>): void;
  getLineDash(): number[];
  setLineDash(segments: Iterable<number> | ArrayLike<number>): void;
}

type EngineCanvasImageSource = import("./image").Image | import("./image").ImageBitmap | Blob | import("./canvas").Canvas | import("./canvas").OffscreenCanvas | string;
type EngineImageBitmapSource = EngineCanvasImageSource;
type EngineImageSourceKind = "image" | "canvas" | "canvasBitmap";

interface EngineImageBitmapCrop {
  sx: number;
  sy: number;
  sw: number;
  sh: number;
}

interface EngineAudioContextOptions {
  sampleRate?: number;
}

type EngineDecodeSuccessCallback = (buffer: import("./audio").AudioBuffer) => void;
type EngineDecodeErrorCallback = (error: unknown) => void;
type EngineImageCallback = (image: import("./image").Image) => void;

interface Blob {
  readonly size: number;
  readonly type: string;
  text(): Promise<string>;
}

interface BlobConstructor {
  new(blobParts?: unknown[], options?: Record<string, unknown>): Blob;
}

declare const canvas: EngineCanvas;
declare function exit(): void;

declare function fetch(input: unknown): Promise<import("./blob").EngineFetchResponse>;

// ---------------------------------------------------------------
// placeholders substituted with literals by native_bridge.rs at load
// (identifiers, so the bundler can't constant-fold them away)
// ---------------------------------------------------------------
declare const __ENGINE_WINDOW_CONFIG__: EngineWindowConfig;
declare const __ENGINE_CONSOLE_ENABLED__: boolean;
declare const __ENGINE_PROFILE_ENABLED__: boolean;
declare const __ENGINE_AUDIO_ENABLED__: boolean;

// -----------------------------------------
// native functions implmeneted on Rust side
// -----------------------------------------
declare function __engine_fetch(url: string): string;
declare function __engine_create_draw_batch(x: number, y: number, width: number, height: number): number;
declare function __engine_append_draw_batch(
  drawBatchId: number,
  x: number,
  y: number,
  width: number,
  height: number,
): void;
declare function __engine_append_active_draw_batch(
  x: number,
  y: number,
  width: number,
  height: number,
): void;
declare function __engine_profile_begin(name: string): void;
declare function __engine_profile_end(name: string): void;
declare function __engine_get_image_data(
  canvasWidth: number,
  canvasHeight: number,
  commands: CanvasDrawCommand[],
  x: number,
  y: number,
  width: number,
  height: number,
): number[];

declare function __engine_audio_context_create(sampleRate?: number): number;
declare function __engine_audio_context_current_time(contextId: number): number;
declare function __engine_audio_context_sample_rate(contextId: number): number;
declare function __engine_audio_context_state(contextId: number): EngineAudioContextState | string;
declare function __engine_audio_context_resume(contextId: number): void;
declare function __engine_audio_context_suspend(contextId: number): void;
declare function __engine_audio_context_close(contextId: number): void;
declare function __engine_audio_create_gain(contextId: number): number;
declare function __engine_audio_create_buffer_source(contextId: number): number;
declare function __engine_audio_create_oscillator(contextId: number): number;
declare function __engine_audio_create_biquad_filter(contextId: number): number;
declare function __engine_audio_connect(contextId: number, sourceId: number, destinationId: number): void;
declare function __engine_audio_disconnect(nodeId: number): void;
declare function __engine_audio_param_get(nodeId: number, name: string): number;
declare function __engine_audio_param_set(nodeId: number, name: string, value: number): void;
declare function __engine_audio_param_set_at_time(nodeId: number, name: string, value: number, startTime: number): void;
declare function __engine_audio_param_linear_ramp_to_value_at_time(
  nodeId: number,
  name: string,
  value: number,
  endTime: number,
): void;
declare function __engine_audio_param_exponential_ramp_to_value_at_time(
  nodeId: number,
  name: string,
  value: number,
  endTime: number,
): void;
declare function __engine_audio_oscillator_get_type(nodeId: number): EngineOscillatorType | string;
declare function __engine_audio_oscillator_set_type(nodeId: number, value: EngineOscillatorType | string): void;
declare function __engine_audio_biquad_get_type(nodeId: number): EngineBiquadFilterType | string;
declare function __engine_audio_biquad_set_type(nodeId: number, value: EngineBiquadFilterType | string): void;
declare function __engine_audio_source_start(nodeId: number, when: number): void;
declare function __engine_audio_source_stop(nodeId: number, when: number): void;
declare function __engine_audio_source_start_at_with_offset(nodeId: number, when: number, offset: number): void;
declare function __engine_audio_source_start_at_with_offset_and_duration(
  nodeId: number,
  when: number,
  offset: number,
  duration: number,
): void;
declare function __engine_audio_buffer_source_set_buffer(nodeId: number, bufferId: number): void;
declare function __engine_audio_buffer_number_of_channels(bufferId: number | null): number;
declare function __engine_audio_buffer_to_channels(bufferId: number): number[][];
declare function __engine_audio_create_buffer_from_channels(
  contextId: number,
  channels: number[][] | null,
  sampleRate: number,
): number;
declare function __engine_audio_decode_audio_data(contextId: number, data: string | ArrayBuffer | ArrayLike<number> | object): number;
declare function __engine_audio_buffer_info(bufferId: number): { length: number; sampleRate: number };
