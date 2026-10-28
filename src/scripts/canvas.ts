import { __engine_number } from "./util";
import { EngineEventTarget } from "./event";
import { ImageBitmap } from "./image";
import { CanvasRenderingContext2D } from "./context2d";
import { engineGlobal } from "./engine/engine-global";

export class Canvas extends EngineEventTarget {
  __width: number;
  __height: number;
  __engine_type: "canvas";
  __engine_id: number;
  __engine_commands: CanvasDrawCommand[];
  __context2d: CanvasRenderingContext2D | null;

  constructor(width: number = 300, height: number = 150) {
    super();
    this.__width = 0;
    this.__height = 0;
    this.__engine_type = "canvas";
    this.__engine_id = ++engineGlobal.__engine_next_canvas_id;
    this.__engine_commands = engineGlobal.__engine_draw_commands;
    this.__context2d = null;
    this.width = width;
    this.height = height;
  }

  get width(): number {
    return this.__width;
  }

  set width(value: number) {
    this.__width = Math.max(0, Math.floor(__engine_number(value, 300)));
  }

  get height(): number {
    return this.__height;
  }

  set height(value: number) {
    this.__height = Math.max(0, Math.floor(__engine_number(value, 150)));
  }

  getContext(kind: string, settings: CanvasRenderingContext2DSettings | undefined = undefined): CanvasRenderingContext2D | null {
    if (kind !== "2d") {
      return null;
    }

    if (this.__context2d === null) {
      this.__context2d = new CanvasRenderingContext2D(this, settings);
    }

    return this.__context2d;
  }
}

export class OffscreenCanvas {
  __width: number;
  __height: number;
  __engine_type: "offscreenCanvas";
  __engine_id: number;
  __engine_commands: CanvasDrawCommand[];
  __context2d: CanvasRenderingContext2D | null;

  constructor(width: number = 300, height: number = 150) {
    this.__width = 0;
    this.__height = 0;
    this.__engine_type = "offscreenCanvas";
    this.__engine_id = ++engineGlobal.__engine_next_canvas_id;
    this.__engine_commands = [];
    this.__context2d = null;
    this.width = width;
    this.height = height;
  }

  get width(): number {
    return this.__width;
  }

  set width(value: number) {
    this.__width = Math.max(0, Math.floor(__engine_number(value, 300)));
    this.__engine_commands.length = 0;
    this.__context2d = null;
  }

  get height(): number {
    return this.__height;
  }

  set height(value: number) {
    this.__height = Math.max(0, Math.floor(__engine_number(value, 150)));
    this.__engine_commands.length = 0;
    this.__context2d = null;
  }

  getContext(kind: string, settings: CanvasRenderingContext2DSettings | undefined = undefined): CanvasRenderingContext2D | null {
    if (kind !== "2d") {
      return null;
    }

    if (this.__context2d === null) {
      this.__context2d = new CanvasRenderingContext2D(this, settings);
    }

    return this.__context2d;
  }

  transferToImageBitmap(): ImageBitmap {
    return new ImageBitmap(this);
  }
}
