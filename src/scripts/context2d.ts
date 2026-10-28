import { EngineMatrixInit } from "./types";
import { __engine_number, __engine_same_optional_number } from "./util";
import { DOMMatrix } from "./geometry";
import { Canvas, OffscreenCanvas } from "./canvas";
import { Image, ImageBitmap, ImageData } from "./image";
import { __engine_is_opaque_style, CanvasGradient, CanvasPattern } from "./paint";
import { __engine_is_path2d, __engine_path_segments, __engine_add_round_rect_path, Path2D } from "./path2d";
import { engineGlobal } from "./engine/engine-global";

export function __engine_is_full_canvas_rect(canvas: { width: number; height: number }, command: CanvasDrawCommand): boolean {
  return command.x === 0 &&
    command.y === 0 &&
    Number(command.width) >= canvas.width &&
    Number(command.height) >= canvas.height;
}

export function __engine_can_batch_draw_image(left: CanvasDrawCommand, right: CanvasDrawCommand): boolean {
  return left.sourceKind === right.sourceKind &&
    left.src === right.src &&
    left.globalAlpha === right.globalAlpha &&
    left.globalCompositeOperation === right.globalCompositeOperation &&
    left.imageSmoothingEnabled === right.imageSmoothingEnabled &&
    left.imageSmoothingQuality === right.imageSmoothingQuality &&
    __engine_same_optional_number(left.bitmapSx, right.bitmapSx) &&
    __engine_same_optional_number(left.bitmapSy, right.bitmapSy) &&
    __engine_same_optional_number(left.bitmapSw, right.bitmapSw) &&
    __engine_same_optional_number(left.bitmapSh, right.bitmapSh) &&
    __engine_same_optional_number(left.width, right.width) &&
    __engine_same_optional_number(left.height, right.height);
}

export class CanvasRenderingContext2D {
  canvas: Canvas | OffscreenCanvas;
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
  imageSmoothingQuality!: EngineImageSmoothingQuality | string;
  font: string;
  textAlign: string;
  textBaseline: string;
  direction: string;
  filter: string;
  shadowBlur: number;
  shadowColor: string;
  shadowOffsetX: number;
  shadowOffsetY: number;
  __drawStateVersion: number;
  __lineDash: number[];
  __path: CanvasPathSegment[];
  __stack: CanvasRenderingContext2DState[];
  __transformA: number;
  __transformB: number;
  __transformC: number;
  __transformD: number;
  __transformE: number;
  __transformF: number;
  __transformIsIdentity: boolean;
  __settings: CanvasRenderingContext2DSettings;
  __commands: CanvasDrawCommand[];
  __activeImageBatch: CanvasImageBatchCommand | null;
  __nativeDrawBatchesEnabled: boolean;

  constructor(canvas: Canvas | OffscreenCanvas, settings: CanvasRenderingContext2DSettings | undefined = undefined) {
    this.canvas = canvas;
    this.__drawStateVersion = 0;
    let fillStyle: CanvasPaintStyle = "#000000";
    Object.defineProperty(this, "fillStyle", {
      configurable: true,
      enumerable: true,
      get: () => fillStyle,
      set: (value: CanvasPaintStyle) => {
        fillStyle = value;
        this.__drawStateVersion++;
      }
    });
    let strokeStyle: CanvasPaintStyle = "#000000";
    Object.defineProperty(this, "strokeStyle", {
      configurable: true,
      enumerable: true,
      get: () => strokeStyle,
      set: (value: CanvasPaintStyle) => {
        strokeStyle = value;
        this.__drawStateVersion++;
      }
    });
    let globalAlpha = 1;
    Object.defineProperty(this, "globalAlpha", {
      configurable: true,
      enumerable: true,
      get: () => globalAlpha,
      set: (value: number) => {
        globalAlpha = value;
        this.__drawStateVersion++;
      }
    });
    let globalCompositeOperation: EngineCompositeOperation | string = "source-over";
    Object.defineProperty(this, "globalCompositeOperation", {
      configurable: true,
      enumerable: true,
      get: () => globalCompositeOperation,
      set: (value: EngineCompositeOperation | string) => {
        globalCompositeOperation = value;
        this.__drawStateVersion++;
      }
    });
    this.lineWidth = 1;
    this.lineCap = "butt";
    this.lineJoin = "miter";
    this.miterLimit = 10;
    this.lineDashOffset = 0;
    let imageSmoothingEnabled = true;
    Object.defineProperty(this, "imageSmoothingEnabled", {
      configurable: true,
      enumerable: true,
      get: () => imageSmoothingEnabled,
      set: (value: boolean) => {
        imageSmoothingEnabled = value;
        this.__drawStateVersion++;
      }
    });
    let imageSmoothingQuality: EngineImageSmoothingQuality = "low";
    Object.defineProperty(this, "imageSmoothingQuality", {
      configurable: true,
      enumerable: true,
      get: () => imageSmoothingQuality,
      set: (value: EngineImageSmoothingQuality | string) => {
        if (value === "low" || value === "medium" || value === "high") {
          imageSmoothingQuality = value;
          this.__drawStateVersion++;
        }
      }
    });
    this.font = "10px sans-serif";
    this.textAlign = "start";
    this.textBaseline = "alphabetic";
    this.direction = "inherit";
    this.filter = "none";
    this.shadowBlur = 0;
    this.shadowColor = "rgba(0, 0, 0, 0)";
    this.shadowOffsetX = 0;
    this.shadowOffsetY = 0;
    this.__lineDash = [];
    this.__path = [];
    this.__stack = [];
    this.__transformA = 1;
    this.__transformB = 0;
    this.__transformC = 0;
    this.__transformD = 1;
    this.__transformE = 0;
    this.__transformF = 0;
    this.__transformIsIdentity = true;
    this.__settings = settings || {};
    this.__commands = canvas.__engine_commands;
    this.__activeImageBatch = null;
    this.__nativeDrawBatchesEnabled = this.__commands === engineGlobal.__engine_draw_commands;
    this.fillStyle = fillStyle;
    this.strokeStyle = strokeStyle;
    this.globalAlpha = globalAlpha;
    this.globalCompositeOperation = globalCompositeOperation;
    this.imageSmoothingEnabled = imageSmoothingEnabled;
    this.imageSmoothingQuality = imageSmoothingQuality;
  }

  __push(command: CanvasDrawCommand): void {
    this.__activeImageBatch = null;
    if (this.__stack.length === 0 && this.__transformIsIdentity && __engine_is_full_canvas_rect(this.canvas, command)) {
      if (command.op === "clearRect" || (
        command.op === "fillRect" &&
        command.globalAlpha === 1 &&
        (command.globalCompositeOperation === "source-over" || command.globalCompositeOperation === "copy") &&
        __engine_is_opaque_style(command.fillStyle)
      )) {
        // Compact in place. `filter()` allocates a temporary array on every
        // full-canvas clear, which is commonly once per frame and can trigger
        // Boa's GC even when there are no cached bitmaps to retain.
        let retainedCount = 0;
        for (let index = 0; index < this.__commands.length; index++) {
          const item = this.__commands[index];
          if (item && item.op === "cacheCanvasBitmap") {
            this.__commands[retainedCount++] = item;
          }
        }
        this.__commands.length = retainedCount;
      }
    }
    if (command.op === "drawImage" && command.sourceKind !== "canvas" && command.dw != null && command.dh != null && command.sx == null) {
      const last = this.__commands[this.__commands.length - 1];
      if (last && last.op === "drawImageBatch" && __engine_can_batch_draw_image(last, command)) {
        (last.draws ??= []).push(Number(command.dx), Number(command.dy), Number(command.dw), Number(command.dh));
        return;
      }

      if (last && last.op === "drawImage" && last.dw != null && last.dh != null && last.sx == null &&
        __engine_can_batch_draw_image(last, command)) {
        this.__commands[this.__commands.length - 1] = {
          op: "drawImageBatch",
          sourceKind: last.sourceKind,
          src: last.src,
          globalAlpha: last.globalAlpha,
          globalCompositeOperation: last.globalCompositeOperation,
          imageSmoothingEnabled: last.imageSmoothingEnabled,
          imageSmoothingQuality: last.imageSmoothingQuality,
          bitmapSx: last.bitmapSx,
          bitmapSy: last.bitmapSy,
          bitmapSw: last.bitmapSw,
          bitmapSh: last.bitmapSh,
          width: last.width,
          height: last.height,
          draws: [
            Number(last.dx),
            Number(last.dy),
            Number(last.dw),
            Number(last.dh),
            Number(command.dx),
            Number(command.dy),
            Number(command.dw),
            Number(command.dh)
          ]
        };
        return;
      }
    }
    this.__commands.push(command);
  }

  getContextAttributes(): Record<string, unknown> {
    return {
      alpha: true,
      colorSpace: this.__settings.colorSpace || "srgb",
      desynchronized: false,
      willReadFrequently: !!this.__settings.willReadFrequently,
      width: this.canvas.width,
      height: this.canvas.height
    };
  }

  save(): void {
    this.__stack.push({
      fillStyle: this.fillStyle,
      strokeStyle: this.strokeStyle,
      globalAlpha: this.globalAlpha,
      globalCompositeOperation: this.globalCompositeOperation,
      lineWidth: this.lineWidth,
      lineCap: this.lineCap,
      lineJoin: this.lineJoin,
      miterLimit: this.miterLimit,
      lineDashOffset: this.lineDashOffset,
      imageSmoothingEnabled: this.imageSmoothingEnabled,
      imageSmoothingQuality: this.imageSmoothingQuality,
      font: this.font,
      textAlign: this.textAlign,
      textBaseline: this.textBaseline,
      direction: this.direction,
      filter: this.filter,
      shadowBlur: this.shadowBlur,
      shadowColor: this.shadowColor,
      shadowOffsetX: this.shadowOffsetX,
      shadowOffsetY: this.shadowOffsetY,
      lineDash: this.__lineDash.slice(),
      transformA: this.__transformA,
      transformB: this.__transformB,
      transformC: this.__transformC,
      transformD: this.__transformD,
      transformE: this.__transformE,
      transformF: this.__transformF,
      transformIsIdentity: this.__transformIsIdentity
    });
    this.__push({ op: "save" });
  }


  restore(): void {
    const state = this.__stack.pop();
    if (state) {
      Object.assign(this, state);
      this.__lineDash = state.lineDash.slice();
      this.__transformA = state.transformA;
      this.__transformB = state.transformB;
      this.__transformC = state.transformC;
      this.__transformD = state.transformD;
      this.__transformE = state.transformE;
      this.__transformF = state.transformF;
      this.__transformIsIdentity = state.transformIsIdentity;
    }
    this.__push({ op: "restore" });
  }


  reset(): void {
    this.restore();
    this.__stack.length = 0;
    this.resetTransform();
    this.beginPath();
  }


  translate(x: number, y: number): void {
    const tx = __engine_number(x);
    const ty = __engine_number(y);
    if (tx !== 0 || ty !== 0) {
      this.__transformIsIdentity = false;
    }
    this.__transformE = this.__transformA * tx + this.__transformC * ty + this.__transformE;
    this.__transformF = this.__transformB * tx + this.__transformD * ty + this.__transformF;
    this.__push({ op: "translate", x: tx, y: ty });
  }


  rotate(angle: number): void {
    const value = __engine_number(angle);
    if (value !== 0) {
      this.__transformIsIdentity = false;
    }
    const cos = Math.cos(value);
    const sin = Math.sin(value);
    const a = this.__transformA * cos + this.__transformC * sin;
    const b = this.__transformB * cos + this.__transformD * sin;
    const c = this.__transformA * -sin + this.__transformC * cos;
    const d = this.__transformB * -sin + this.__transformD * cos;
    this.__transformA = a;
    this.__transformB = b;
    this.__transformC = c;
    this.__transformD = d;
    this.__push({ op: "rotate", angle: value });
  }


  scale(x: number, y: number = x): void {
    const sx = __engine_number(x, 1);
    const sy = __engine_number(y, 1);
    if (sx !== 1 || sy !== 1) {
      this.__transformIsIdentity = false;
    }
    this.__transformA *= sx;
    this.__transformB *= sx;
    this.__transformC *= sy;
    this.__transformD *= sy;
    this.__push({ op: "scale", x: sx, y: sy });
  }


  transform(a: number, b: number, c: number, d: number, e: number, f: number): void {
    const ma = __engine_number(a, 1);
    const mb = __engine_number(b);
    const mc = __engine_number(c);
    const md = __engine_number(d, 1);
    const me = __engine_number(e);
    const mf = __engine_number(f);
    if (ma !== 1 || mb !== 0 || mc !== 0 || md !== 1 || me !== 0 || mf !== 0) {
      this.__transformIsIdentity = false;
    }
    const nextA = this.__transformA * ma + this.__transformC * mb;
    const nextB = this.__transformB * ma + this.__transformD * mb;
    const nextC = this.__transformA * mc + this.__transformC * md;
    const nextD = this.__transformB * mc + this.__transformD * md;
    const nextE = this.__transformA * me + this.__transformC * mf + this.__transformE;
    const nextF = this.__transformB * me + this.__transformD * mf + this.__transformF;
    this.__transformA = nextA;
    this.__transformB = nextB;
    this.__transformC = nextC;
    this.__transformD = nextD;
    this.__transformE = nextE;
    this.__transformF = nextF;
    this.__push({
      op: "transform",
      a: ma,
      b: mb,
      c: mc,
      d: md,
      e: me,
      f: mf
    });
  }


  setTransform(a: number | EngineMatrixInit = 1, b: number = 0, c: number = 0, d: number = 1, e: number = 0, f: number = 0): void {
    if (typeof a === "object" && a !== null) {
      const matrix = new DOMMatrix(a);
      a = matrix.a;
      b = matrix.b;
      c = matrix.c;
      d = matrix.d;
      e = matrix.e;
      f = matrix.f;
    }
    const nextA = __engine_number(a, 1);
    const nextB = __engine_number(b);
    const nextC = __engine_number(c);
    const nextD = __engine_number(d, 1);
    const nextE = __engine_number(e);
    const nextF = __engine_number(f);
    const command: CanvasDrawCommand = {
      op: "setTransform",
      a: nextA,
      b: nextB,
      c: nextC,
      d: nextD,
      e: nextE,
      f: nextF
    };
    this.__transformIsIdentity = nextA === 1 && nextB === 0 && nextC === 0 &&
      nextD === 1 && nextE === 0 && nextF === 0;
    this.__transformA = nextA;
    this.__transformB = nextB;
    this.__transformC = nextC;
    this.__transformD = nextD;
    this.__transformE = nextE;
    this.__transformF = nextF;
    this.__push(command);
  }


  resetTransform(): void {
    this.__transformA = 1;
    this.__transformB = 0;
    this.__transformC = 0;
    this.__transformD = 1;
    this.__transformE = 0;
    this.__transformF = 0;
    this.__transformIsIdentity = true;
    this.__push({ op: "resetTransform" });
  }


  getTransform(): DOMMatrix {
    return new DOMMatrix([
      this.__transformA,
      this.__transformB,
      this.__transformC,
      this.__transformD,
      this.__transformE,
      this.__transformF
    ]);
  }


  beginPath(): void {
    this.__path = [];
  }


  closePath(): void {
    this.__path.push({ verb: "closePath" });
  }

  moveTo(x: number, y: number): void {
    this.__path.push({ verb: "moveTo", x: typeof x === "number" ? x : __engine_number(x), y: typeof y === "number" ? y : __engine_number(y) });
  }

  lineTo(x: number, y: number): void {
    this.__path.push({ verb: "lineTo", x: typeof x === "number" ? x : __engine_number(x), y: typeof y === "number" ? y : __engine_number(y) });
  }

  bezierCurveTo(cp1x: number, cp1y: number, cp2x: number, cp2y: number, x: number, y: number): void {
    this.__path.push({
      verb: "bezierCurveTo",
      cp1x: __engine_number(cp1x),
      cp1y: __engine_number(cp1y),
      cp2x: __engine_number(cp2x),
      cp2y: __engine_number(cp2y),
      x: __engine_number(x),
      y: __engine_number(y)
    });
  }

  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void {
    this.__path.push({
      verb: "quadraticCurveTo",
      cpx: __engine_number(cpx),
      cpy: __engine_number(cpy),
      x: __engine_number(x),
      y: __engine_number(y)
    });
  }

  rect(x: number, y: number, width: number, height: number): void {
    this.__path.push({
      verb: "rect",
      x: __engine_number(x),
      y: __engine_number(y),
      width: __engine_number(width),
      height: __engine_number(height)
    });
  }

  arc(x: number, y: number, radius: number, startAngle: number, endAngle: number, counterclockwise: boolean = false): void {
    const value = typeof radius === "number" ? radius : __engine_number(radius);
    if (value < 0) throw new RangeError("arc() radius must be non-negative");
    this.__path.push({
      verb: "arc",
      x: typeof x === "number" ? x : __engine_number(x),
      y: typeof y === "number" ? y : __engine_number(y),
      radius: value,
      startAngle: typeof startAngle === "number" ? startAngle : __engine_number(startAngle),
      endAngle: typeof endAngle === "number" ? endAngle : __engine_number(endAngle),
      counterclockwise: !!counterclockwise
    });
  }

  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void {
    const value = __engine_number(radius);
    if (value < 0) throw new RangeError("arcTo() radius must be non-negative");
    this.__path.push({
      verb: "arcTo",
      x1: __engine_number(x1),
      y1: __engine_number(y1),
      x2: __engine_number(x2),
      y2: __engine_number(y2),
      radius: value
    });
  }

  ellipse(
    x: number,
    y: number,
    radiusX: number,
    radiusY: number,
    rotation: number,
    startAngle: number,
    endAngle: number,
    counterclockwise: boolean = false
  ): void {
    const rx = __engine_number(radiusX);
    const ry = __engine_number(radiusY);
    if (rx < 0 || ry < 0) throw new RangeError("ellipse() radii must be non-negative");
    this.__path.push({
      verb: "ellipse",
      x: __engine_number(x),
      y: __engine_number(y),
      radiusX: rx,
      radiusY: ry,
      rotation: __engine_number(rotation),
      startAngle: __engine_number(startAngle),
      endAngle: __engine_number(endAngle),
      counterclockwise: !!counterclockwise
    });
  }

  roundRect(x: number, y: number, width: number, height: number, radii: unknown = 0): void {
    __engine_add_round_rect_path(
      this.__path,
      __engine_number(x),
      __engine_number(y),
      __engine_number(width),
      __engine_number(height),
      radii
    );
  }

  fill(pathOrRule: Path2D | EngineFillRule | string = "nonzero", rule: EngineFillRule | string = "nonzero"): void {
    if (!__engine_is_path2d(pathOrRule) && this.__path.length === 1) {
      const seg = this.__path[0];
      if (seg.verb === "arc" && seg.startAngle != null && seg.endAngle != null && Math.abs(seg.endAngle - seg.startAngle) >= 6.28) {
        this.__push({
          op: "fillCircle",
          x: seg.x,
          y: seg.y,
          radius: seg.radius,
          fillStyle: this.fillStyle,
          globalAlpha: this.globalAlpha,
          globalCompositeOperation: this.globalCompositeOperation,
          imageSmoothingEnabled: this.imageSmoothingEnabled,
          imageSmoothingQuality: this.imageSmoothingQuality
        });
        return;
      }
    }
    const path = __engine_is_path2d(pathOrRule) ? __engine_path_segments(pathOrRule) : this.__path.slice();
    const fillRule = __engine_is_path2d(pathOrRule) ? rule : pathOrRule;
    this.__push({
      op: "fillPath",
      path,
      fillStyle: this.fillStyle,
      globalAlpha: this.globalAlpha,
      globalCompositeOperation: this.globalCompositeOperation,
      imageSmoothingEnabled: this.imageSmoothingEnabled,
      imageSmoothingQuality: this.imageSmoothingQuality,
      rule: String(fillRule)
    });
  }

  stroke(path?: Path2D): void {
    if (!path && this.__path.length === 2 && this.__path[0].verb === "moveTo" && this.__path[1].verb === "lineTo") {
      const p0 = this.__path[0];
      const p1 = this.__path[1];
      if (p0.x != null && p0.y != null && p1.x != null && p1.y != null) {
        this.__push({
          op: "strokeLine",
          x0: p0.x,
          y0: p0.y,
          x1: p1.x,
          y1: p1.y,
          strokeStyle: this.strokeStyle,
          globalAlpha: this.globalAlpha,
          globalCompositeOperation: this.globalCompositeOperation,
          imageSmoothingEnabled: this.imageSmoothingEnabled,
          imageSmoothingQuality: this.imageSmoothingQuality,
          lineWidth: this.lineWidth,
          lineCap: this.lineCap,
          lineJoin: this.lineJoin,
          miterLimit: this.miterLimit
        });
        return;
      }
    }
    this.__push({
      op: "strokePath",
      path: path instanceof Path2D ? __engine_path_segments(path) : this.__path.slice(),
      strokeStyle: this.strokeStyle,
      globalAlpha: this.globalAlpha,
      globalCompositeOperation: this.globalCompositeOperation,
      imageSmoothingEnabled: this.imageSmoothingEnabled,
      imageSmoothingQuality: this.imageSmoothingQuality,
      lineWidth: this.lineWidth,
      lineCap: this.lineCap,
      lineJoin: this.lineJoin,
      miterLimit: this.miterLimit
    });
  }

  clip(pathOrRule: Path2D | EngineFillRule | string = "nonzero", rule: EngineFillRule | string = "nonzero"): void {
    const path = __engine_is_path2d(pathOrRule) ? __engine_path_segments(pathOrRule) : this.__path.slice();
    const clipRule = __engine_is_path2d(pathOrRule) ? rule : pathOrRule;
    this.__push({
      op: "clipPath",
      path,
      rule: String(clipRule)
    });
  }

  isPointInPath(): boolean {
    return false;
  }

  isPointInStroke(): boolean {
    return false;
  }

  clearRect(x: number, y: number, width: number, height: number): void {
    this.__push({
      op: "clearRect",
      x: __engine_number(x),
      y: __engine_number(y),
      width: __engine_number(width),
      height: __engine_number(height)
    });
  }

  fillRect(x: number, y: number, width: number, height: number): void {
    this.__push({
      op: "fillRect",
      x: __engine_number(x),
      y: __engine_number(y),
      width: __engine_number(width),
      height: __engine_number(height),
      fillStyle: this.fillStyle,
      globalAlpha: __engine_number(this.globalAlpha, 1),
      globalCompositeOperation: String(this.globalCompositeOperation),
      imageSmoothingEnabled: Boolean(this.imageSmoothingEnabled),
      imageSmoothingQuality: String(this.imageSmoothingQuality)
    });
  }

  strokeRect(x: number, y: number, width: number, height: number): void {
    this.__push({
      op: "strokeRect",
      x: __engine_number(x),
      y: __engine_number(y),
      width: __engine_number(width),
      height: __engine_number(height),
      strokeStyle: this.strokeStyle,
      globalAlpha: __engine_number(this.globalAlpha, 1),
      globalCompositeOperation: String(this.globalCompositeOperation),
      imageSmoothingEnabled: Boolean(this.imageSmoothingEnabled),
      imageSmoothingQuality: String(this.imageSmoothingQuality),
      lineWidth: __engine_number(this.lineWidth, 1),
      lineCap: String(this.lineCap),
      lineJoin: String(this.lineJoin),
      miterLimit: __engine_number(this.miterLimit, 10)
    });
  }

  fillText(): never {
    throw new Error("fillText() is part of the skia-canvas API surface but is not implemented yet");
  }

  strokeText(): never {
    throw new Error("strokeText() is part of the skia-canvas API surface but is not implemented yet");
  }

  measureText(text: string): { width: number } {
    return { width: String(text).length * 10 };
  }

  drawImage(image: EngineImageBitmapSource, a: number, b: number, c?: number, d?: number, e?: number, f?: number, g?: number, h?: number): void {
    if (this.__nativeDrawBatchesEnabled && arguments.length === 5 && image instanceof Image) {
      const srcText = image.__src;
      if (!srcText) {
        throw new TypeError("drawImage() requires a loaded Image");
      }

      let batch = this.__activeImageBatch;
      if (batch &&
        batch.imageSource === image &&
        batch.src === srcText &&
        batch.drawStateVersion === this.__drawStateVersion) {
        __engine_append_active_draw_batch(a, b, c ?? 0, d ?? 0);
        return;
      }

      const globalAlpha = Number(this.globalAlpha);
      const alpha = Number.isFinite(globalAlpha) ? globalAlpha : 1;
      const composite = String(this.globalCompositeOperation);
      const smoothing = this.imageSmoothingEnabled === true;
      const quality = String(this.imageSmoothingQuality);
      batch = {
        op: "drawImageBatch",
        sourceKind: "image",
        src: srcText,
        imageSource: image,
        drawStateVersion: this.__drawStateVersion,
        globalAlpha: alpha,
        globalCompositeOperation: composite,
        imageSmoothingEnabled: smoothing,
        imageSmoothingQuality: quality,
        drawBatchId: __engine_create_draw_batch(a, b, c ?? 0, d ?? 0)
      };
      this.__activeImageBatch = batch;
      this.__commands.push(batch);
      return;
    }

    const isBitmap = image instanceof ImageBitmap;
    if (isBitmap && image.closed) {
      throw new Error("Cannot draw a closed ImageBitmap");
    }

    const canvasSource = image instanceof OffscreenCanvas || image instanceof Canvas ? image : null;
    const imageSource = image instanceof Image ? image : null;
    const sourceKind = isBitmap
      ? image.__engine_source_kind
      : canvasSource ? "canvas" : "image";
    const src = isBitmap
      ? image.__engine_src
      : sourceKind === "canvas"
        ? String(canvasSource?.__engine_id ?? 0)
        : typeof image === "string" ? image : imageSource?.src;

    if (!src && sourceKind !== "canvas") {
      throw new TypeError("drawImage() requires an Image, image path, Canvas, or OffscreenCanvas");
    }

    if (this.__nativeDrawBatchesEnabled && !isBitmap && sourceKind === "image" && arguments.length === 5) {
      const srcText = String(src);
      const globalAlpha = Number(this.globalAlpha);
      const alpha = Number.isFinite(globalAlpha) ? globalAlpha : 1;
      const composite = String(this.globalCompositeOperation);
      const smoothing = Boolean(this.imageSmoothingEnabled);
      let batch = this.__activeImageBatch;
      if (batch &&
        batch.src === srcText &&
        batch.globalAlpha === alpha &&
        batch.globalCompositeOperation === composite &&
        batch.imageSmoothingEnabled === smoothing &&
        batch.imageSmoothingQuality === String(this.imageSmoothingQuality)) {
        __engine_append_draw_batch(batch.drawBatchId ?? 0, a, b, c ?? 0, d ?? 0);
        return;
      }

      batch = {
        op: "drawImageBatch",
        sourceKind: "image",
        src: srcText,
        globalAlpha: alpha,
        globalCompositeOperation: composite,
        imageSmoothingEnabled: smoothing,
        imageSmoothingQuality: String(this.imageSmoothingQuality),
        drawBatchId: __engine_create_draw_batch(a, b, c ?? 0, d ?? 0)
      };
      this.__activeImageBatch = batch;
      this.__commands.push(batch);
      return;
    }

    let command: CanvasDrawCommand;
    if (arguments.length === 3) {
      command = {
        op: "drawImage",
        sourceKind,
        src: String(src),
        dx: __engine_number(a),
        dy: __engine_number(b)
      };
    } else if (arguments.length === 5) {
      command = {
        op: "drawImage",
        sourceKind,
        src: String(src),
        dx: __engine_number(a),
        dy: __engine_number(b),
        dw: __engine_number(c),
        dh: __engine_number(d)
      };
    } else if (arguments.length === 9) {
      command = {
        op: "drawImage",
        sourceKind,
        src: String(src),
        sx: __engine_number(a),
        sy: __engine_number(b),
        sw: __engine_number(c),
        sh: __engine_number(d),
        dx: __engine_number(e),
        dy: __engine_number(f),
        dw: __engine_number(g),
        dh: __engine_number(h)
      };
    } else {
      throw new TypeError("drawImage() expects 3, 5, or 9 total arguments");
    }

    command.globalAlpha = __engine_number(this.globalAlpha, 1);
    command.globalCompositeOperation = String(this.globalCompositeOperation);
    command.imageSmoothingEnabled = Boolean(this.imageSmoothingEnabled);
    command.imageSmoothingQuality = String(this.imageSmoothingQuality);
    if (isBitmap) {
      command.bitmapSx = image.__engine_bitmap_sx;
      command.bitmapSy = image.__engine_bitmap_sy;
      command.bitmapSw = image.__engine_bitmap_sw;
      command.bitmapSh = image.__engine_bitmap_sh;
    }
    if (sourceKind === "canvas") {
      command.width = isBitmap ? image.__engine_source_width : canvasSource?.width ?? 0;
      command.height = isBitmap ? image.__engine_source_height : canvasSource?.height ?? 0;
      command.commands = isBitmap ? image.__engine_commands.slice() : canvasSource?.__engine_commands.slice() ?? [];
    } else if (isBitmap && sourceKind === "canvasBitmap") {
      command.width = image.__engine_source_width;
      command.height = image.__engine_source_height;
    }
    this.__push(command);
  }

  drawImageBatch(image: EngineImageBitmapSource, draws: Iterable<number> | ArrayLike<number>): void {
    const isBitmap = image instanceof ImageBitmap;
    if (isBitmap && image.closed) {
      throw new Error("Cannot draw a closed ImageBitmap");
    }

    const imageSource = image instanceof Image ? image : null;
    const sourceKind = isBitmap
      ? image.__engine_source_kind
      : image instanceof OffscreenCanvas || image instanceof Canvas ? "canvas" : "image";
    if (sourceKind === "canvas") {
      throw new TypeError("drawImageBatch() does not support live Canvas sources");
    }

    const src = isBitmap
      ? image.__engine_src
      : typeof image === "string" ? image : imageSource?.src;
    if (!src) {
      throw new TypeError("drawImageBatch() requires an Image, image path, or ImageBitmap");
    }

    const command: CanvasImageBatchCommand = {
      op: "drawImageBatch",
      sourceKind,
      src: String(src),
      globalAlpha: __engine_number(this.globalAlpha, 1),
      globalCompositeOperation: String(this.globalCompositeOperation),
      imageSmoothingEnabled: Boolean(this.imageSmoothingEnabled),
      imageSmoothingQuality: String(this.imageSmoothingQuality),
      draws: Array.isArray(draws) ? draws : Array.from(draws || [], Number)
    };

    if (isBitmap) {
      command.bitmapSx = image.__engine_bitmap_sx;
      command.bitmapSy = image.__engine_bitmap_sy;
      command.bitmapSw = image.__engine_bitmap_sw;
      command.bitmapSh = image.__engine_bitmap_sh;
    }
    if (isBitmap && sourceKind === "canvasBitmap") {
      command.width = image.__engine_source_width;
      command.height = image.__engine_source_height;
    }

    this.__push(command);
  }

  createLinearGradient(x0: number, y0: number, x1: number, y1: number): CanvasGradient {
    return new CanvasGradient("linearGradient", { x0, y0, x1, y1 });
  }

  createRadialGradient(x0: number, y0: number, r0: number, x1: number, y1: number, r1: number): CanvasGradient {
    if (__engine_number(r0) < 0 || __engine_number(r1) < 0) {
      throw new RangeError("Gradient radius must be non-negative");
    }
    return new CanvasGradient("radialGradient", { x0, y0, r0, x1, y1, r1 });
  }

  createConicGradient(startAngle: number, x: number, y: number): CanvasGradient {
    return new CanvasGradient("conicGradient", { startAngle, x0: x, y0: y });
  }

  createPattern(image: EngineImageBitmapSource, repetition: string | null = "repeat"): CanvasPattern | null {
    return new CanvasPattern(image, repetition);
  }

  createImageData(widthOrImageData: number | ImageData, height?: number, settings: Record<string, unknown> = {}): ImageData {
    if (widthOrImageData instanceof ImageData) {
      return new ImageData(widthOrImageData.width, widthOrImageData.height, {
        colorSpace: widthOrImageData.colorSpace,
        colorType: widthOrImageData.colorType
      });
    }
    return new ImageData(widthOrImageData, height, undefined, settings);
  }

  getImageData(x: number, y: number, width: number, height: number, settings: Record<string, unknown> = {}): ImageData {
    const sx = Math.trunc(__engine_number(x));
    const sy = Math.trunc(__engine_number(y));
    const sw = Math.trunc(__engine_number(width));
    const sh = Math.trunc(__engine_number(height));
    if (sw === 0 || sh === 0) {
      throw new RangeError("getImageData() width and height must be non-zero");
    }

    const data = __engine_get_image_data(
      this.canvas.width,
      this.canvas.height,
      this.__commands,
      sx,
      sy,
      sw,
      sh
    );
    return new ImageData(
      new Uint8ClampedArray(data),
      Math.abs(sw),
      Math.abs(sh),
      {
        colorSpace: settings.colorSpace ?? this.__settings.colorSpace ?? "srgb",
        colorType: "rgba"
      }
    );
  }

  putImageData(imageData: ImageData, dx: number, dy: number, dirtyX?: number, dirtyY?: number, dirtyWidth?: number, dirtyHeight?: number): void {
    if (!(imageData instanceof ImageData)) {
      throw new TypeError("putImageData() expects an ImageData");
    }

    const command: CanvasDrawCommand = {
      op: "putImageData",
      data: imageData.data,
      width: imageData.width,
      height: imageData.height,
      dx: Math.trunc(__engine_number(dx)),
      dy: Math.trunc(__engine_number(dy))
    };

    if (arguments.length >= 7) {
      command.dirtyX = Math.trunc(__engine_number(dirtyX));
      command.dirtyY = Math.trunc(__engine_number(dirtyY));
      command.dirtyWidth = Math.trunc(__engine_number(dirtyWidth));
      command.dirtyHeight = Math.trunc(__engine_number(dirtyHeight));
    }

    this.__push(command);
  }

  getLineDash(): number[] {
    return this.__lineDash.slice();
  }

  setLineDash(segments: Iterable<number> | ArrayLike<number>): void {
    this.__lineDash = Array.from(segments || [], Number).filter(Number.isFinite);
  }
}
