import { EngineEvent } from "../event";
import { DOMPoint, DOMMatrix } from "../geometry";
import { Canvas, OffscreenCanvas } from "../canvas";
import { Image, ImageBitmap, ImageData } from "../image";
import { CanvasGradient, CanvasPattern } from "../paint";
import { EngineBlob, EngineFetchResponse } from "../blob";
import { Path2D } from "../path2d";
import { CanvasRenderingContext2D } from "../context2d";
import { AudioParam, AudioNode, GainNode, OscillatorNode, BiquadFilterNode, AudioBuffer, AudioBufferSourceNode, AudioContext } from "../audio";
import { engineGlobal } from "./engine-global";

engineGlobal.__engine_window = __ENGINE_WINDOW_CONFIG__;

engineGlobal.__engine_draw_commands = [] as CanvasDrawCommand[];
engineGlobal.__engine_console_messages = [];
engineGlobal.__engine_console_dirty = false;
engineGlobal.__engine_console_enabled = __ENGINE_CONSOLE_ENABLED__;
engineGlobal.__engine_profile_enabled = __ENGINE_PROFILE_ENABLED__;
engineGlobal.__engine_next_canvas_id = 0;
engineGlobal.__engine_next_bitmap_id = 0;
engineGlobal.__engine_last_tick_timestamp = -1;
const engineAudioEnabled = __ENGINE_AUDIO_ENABLED__;

engineGlobal.Canvas = Canvas;
engineGlobal.OffscreenCanvas = OffscreenCanvas;
engineGlobal.CanvasRenderingContext2D = CanvasRenderingContext2D;
engineGlobal.Image = Image;
engineGlobal.ImageBitmap = ImageBitmap;
engineGlobal.Blob = EngineBlob;
engineGlobal.Response = EngineFetchResponse;
engineGlobal.ImageData = ImageData;
engineGlobal.CanvasGradient = CanvasGradient;
engineGlobal.CanvasPattern = CanvasPattern;
engineGlobal.Path2D = Path2D;
if (engineAudioEnabled) {
  engineGlobal.AudioContext = AudioContext;
  engineGlobal.webkitAudioContext = AudioContext;
  engineGlobal.AudioNode = AudioNode;
  engineGlobal.AudioParam = AudioParam;
  engineGlobal.AudioBuffer = AudioBuffer;
  engineGlobal.AudioBufferSourceNode = AudioBufferSourceNode;
  engineGlobal.GainNode = GainNode;
  engineGlobal.OscillatorNode = OscillatorNode;
  engineGlobal.BiquadFilterNode = BiquadFilterNode;
}
engineGlobal.canvas = new Canvas(engineGlobal.__engine_window.width, engineGlobal.__engine_window.height);
engineGlobal.window = engineGlobal.canvas;
engineGlobal.__engine_invalidate_draw_batches = function __engine_invalidate_draw_batches(): void {
  const canvasContext = engineGlobal.canvas.__context2d;
  if (canvasContext) {
    canvasContext.__activeImageBatch = null;
  }
};
engineGlobal.Event = EngineEvent;
engineGlobal.DOMMatrix = DOMMatrix;
engineGlobal.DOMPoint = DOMPoint;
engineGlobal.fetch = function fetch(input: unknown): Promise<EngineFetchResponse> {
  const url = typeof input === "object" && input !== null && "url" in input
    ? String((input as { url: unknown }).url)
    : String(input);
  return Promise.resolve(new EngineFetchResponse(url));
};
