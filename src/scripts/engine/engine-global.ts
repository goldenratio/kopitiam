import type { AudioBuffer, AudioBufferSourceNode, AudioContext, AudioNode, AudioParam, BiquadFilterNode, GainNode, OscillatorNode } from "../audio";
import type { EngineBlob, EngineFetchResponse } from "../blob";
import type { Canvas, OffscreenCanvas } from "../canvas";
import type { CanvasRenderingContext2D } from "../context2d";
import type { EngineEvent, EngineEventTarget } from "../event";
import type { DOMMatrix, DOMPoint } from "../geometry";
import type { Image, ImageBitmap, ImageData } from "../image";
import type { CanvasGradient, CanvasPattern } from "../paint";
import type { Path2D } from "../path2d";
import type { EngineEventInit, EngineListener } from "../types";

/** The engine's `globalThis`: native-bridge state plus the browser-like globals installed by the prelude. */
export interface EngineGlobal {
  // state shared with the Rust side
  __engine_window: EngineWindowConfig;
  __engine_draw_commands: CanvasDrawCommand[];
  __engine_console_messages: EngineConsoleMessage[];
  __engine_console_dirty: boolean;
  __engine_console_enabled: boolean;
  __engine_profile_enabled: boolean;
  __engine_next_canvas_id: number;
  __engine_next_bitmap_id: number;
  __engine_last_tick_timestamp: number;
  __engine_global_event_target: EngineEventTarget;

  // functions called from the Rust side
  __engine_invalidate_draw_batches(): void;
  __engine_dispatch_events(events: EngineEventInit[]): void;
  __engine_dispatch_key_events(events: EngineEventInit[]): void;
  __engine_dispatch_resize(width: number, height: number, resizeCanvas?: boolean): void;
  __engine_dispatch_tick(timestamp: number): void;

  // `on<type>` event handler properties
  [handler: `on${string}`]: ((event: EngineEvent) => void) | undefined;

  console: {
    log(...args: unknown[]): void;
    info(...args: unknown[]): void;
    debug(...args: unknown[]): void;
    warn(...args: unknown[]): void;
    error(...args: unknown[]): void;
  };
  performance: { now(): number };
  devicePixelRatio: number;

  canvas: Canvas;
  window: Canvas;

  addEventListener(type: string, listener: EngineListener): void;
  removeEventListener(type: string, listener: EngineListener): void;
  dispatchEvent(event: EngineEvent): boolean;
  fetch(input: unknown): Promise<EngineFetchResponse>;
  createImageBitmap(source: EngineImageBitmapSource, ...args: number[]): Promise<ImageBitmap>;

  exit(): void;

  // constructors
  Event: typeof EngineEvent;
  PointerEvent: typeof EngineEvent;
  KeyboardEvent: typeof EngineEvent;
  DOMMatrix: typeof DOMMatrix;
  DOMPoint: typeof DOMPoint;
  Canvas: typeof Canvas;
  OffscreenCanvas: typeof OffscreenCanvas;
  CanvasRenderingContext2D: typeof CanvasRenderingContext2D;
  Image: typeof Image;
  ImageBitmap: typeof ImageBitmap;
  ImageData: typeof ImageData;
  Blob: typeof EngineBlob;
  Response: typeof EngineFetchResponse;
  CanvasGradient: typeof CanvasGradient;
  CanvasPattern: typeof CanvasPattern;
  Path2D: typeof Path2D;
  AudioContext: typeof AudioContext;
  webkitAudioContext: typeof AudioContext;
  AudioNode: typeof AudioNode;
  AudioParam: typeof AudioParam;
  AudioBuffer: typeof AudioBuffer;
  AudioBufferSourceNode: typeof AudioBufferSourceNode;
  GainNode: typeof GainNode;
  OscillatorNode: typeof OscillatorNode;
  BiquadFilterNode: typeof BiquadFilterNode;
}

export const engineGlobal = globalThis as unknown as EngineGlobal;
