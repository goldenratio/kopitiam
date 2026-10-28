import { EngineMatrixInit } from "./types";
import { __engine_number } from "./util";
import { Canvas, OffscreenCanvas } from "./canvas";
import { Image, ImageBitmap } from "./image";

export function __engine_is_opaque_style(style: unknown): boolean {
  if (typeof style !== "string") return false;
  const value = String(style).trim().toLowerCase();
  if (value === "transparent" || value.startsWith("rgba(") || value.startsWith("hsla(")) {
    return false;
  }

  return value === "black" || value === "white" || value === "red" || value === "green" ||
    value === "blue" ||
    /^#[0-9a-f]{3}$/.test(value) ||
    /^#[0-9a-f]{6}$/.test(value) ||
    (/^#[0-9a-f]{8}$/.test(value) && value.slice(7, 9) === "ff");
}

export class CanvasGradient {
  readonly type: "linearGradient" | "radialGradient" | "conicGradient";
  readonly x0: number;
  readonly y0: number;
  readonly x1: number;
  readonly y1: number;
  readonly r0: number;
  readonly r1: number;
  readonly startAngle: number;
  readonly stops: { offset: number; color: string }[];

  constructor(type: "linearGradient" | "radialGradient" | "conicGradient", values: Record<string, number>) {
    this.type = type;
    this.x0 = __engine_number(values.x0);
    this.y0 = __engine_number(values.y0);
    this.x1 = __engine_number(values.x1);
    this.y1 = __engine_number(values.y1);
    this.r0 = __engine_number(values.r0);
    this.r1 = __engine_number(values.r1);
    this.startAngle = __engine_number(values.startAngle);
    this.stops = [];
  }

  addColorStop(offset: number, color: string): void {
    const value = Number(offset);
    if (!Number.isFinite(value) || value < 0 || value > 1) {
      throw new RangeError("Color stop offset must be between 0 and 1");
    }
    this.stops.push({ offset: value, color: String(color) });
    this.stops.sort((a, b) => a.offset - b.offset);
  }
}

export class CanvasPattern {
  readonly type = "pattern";
  readonly sourceKind: EngineImageSourceKind;
  readonly src: string;
  readonly repetition: string;
  readonly width: number;
  readonly height: number;
  readonly commands: CanvasDrawCommand[];
  transform: { a: number; b: number; c: number; d: number; e: number; f: number };

  constructor(source: EngineImageBitmapSource, repetition: string | null | undefined = "repeat") {
    const normalizedRepetition = repetition == null || repetition === "" ? "repeat" : String(repetition);
    if (!["repeat", "repeat-x", "repeat-y", "no-repeat"].includes(normalizedRepetition)) {
      throw new TypeError("createPattern() repetition must be repeat, repeat-x, repeat-y, or no-repeat");
    }

    const bitmap = source instanceof ImageBitmap ? source : null;
    const canvasSource = source instanceof OffscreenCanvas || source instanceof Canvas ? source : null;
    const imageSource = source instanceof Image ? source : null;
    if (bitmap && bitmap.closed) {
      throw new Error("Cannot create a pattern from a closed ImageBitmap");
    }
    if (imageSource && !imageSource.complete) {
      throw new TypeError("createPattern() requires a loaded Image");
    }
    if (!(bitmap || canvasSource || imageSource || typeof source === "string")) {
      throw new TypeError("createPattern() requires an Image, image path, Canvas, OffscreenCanvas, or ImageBitmap");
    }

    this.sourceKind = bitmap ? bitmap.__engine_source_kind : canvasSource ? "canvas" : "image";
    this.src = bitmap
      ? bitmap.__engine_src
      : typeof source === "string"
        ? String(source)
        : imageSource
          ? String(imageSource.src)
          : String(canvasSource?.__engine_id ?? 0);
    this.repetition = normalizedRepetition;
    this.width = bitmap ? bitmap.__engine_source_width : canvasSource?.width ?? 0;
    this.height = bitmap ? bitmap.__engine_source_height : canvasSource?.height ?? 0;
    this.commands = bitmap ? bitmap.__engine_commands.slice() : canvasSource?.__engine_commands.slice() ?? [];
    this.transform = { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 };
  }

  setTransform(transform: EngineMatrixInit | null = null): void {
    if (Array.isArray(transform)) {
      this.transform = {
        a: __engine_number(transform[0], 1),
        b: __engine_number(transform[1]),
        c: __engine_number(transform[2]),
        d: __engine_number(transform[3], 1),
        e: __engine_number(transform[4]),
        f: __engine_number(transform[5])
      };
      return;
    }

    const matrix = transform || {};
    this.transform = {
      a: __engine_number(matrix.a ?? matrix.m11, 1),
      b: __engine_number(matrix.b ?? matrix.m12),
      c: __engine_number(matrix.c ?? matrix.m21),
      d: __engine_number(matrix.d ?? matrix.m22, 1),
      e: __engine_number(matrix.e ?? matrix.m41),
      f: __engine_number(matrix.f ?? matrix.m42)
    };
  }
}
