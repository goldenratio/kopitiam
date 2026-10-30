import type { EngineListener } from "./types";
import { __engine_number } from "./util";
import { EngineEvent, EngineEventTarget } from "./event";
import { Canvas, OffscreenCanvas } from "./canvas";
import { EngineBlob } from "./blob";
import { engineGlobal } from "./engine";

export class Image extends EngineEventTarget {
  __width: number;
  __height: number;
  complete: boolean;
  naturalWidth: number;
  naturalHeight: number;
  currentSrc: string;
  onload: EngineListener | null;
  onerror: EngineListener | null;
  crossOrigin: string | null;
  decoding: string;
  loading: string;
  referrerPolicy: string;
  __src: string;

  constructor(width: number = 0, height: number = 0) {
    super();
    this.__width = 0;
    this.__height = 0;
    this.complete = true;
    this.naturalWidth = 0;
    this.naturalHeight = 0;
    this.currentSrc = "";
    this.onload = null;
    this.onerror = null;
    this.crossOrigin = null;
    this.decoding = "auto";
    this.loading = "eager";
    this.referrerPolicy = "";
    this.__src = "";

    this.width = width;
    this.height = height;
  }

  get width(): number {
    return this.__width;
  }

  set width(value: number) {
    this.__width = Math.max(0, Math.floor(__engine_number(value)));
  }

  get height(): number {
    return this.__height;
  }

  set height(value: number) {
    this.__height = Math.max(0, Math.floor(__engine_number(value)));
  }

  get src(): string {
    return this.__src;
  }

  set src(value: string) {
    this.__src = String(value);
    this.currentSrc = this.__src;
    this.complete = true;
    if (this.__src.length > 0) {
      this.dispatchEvent(new EngineEvent("load", {
        target: this,
        currentTarget: this
      }));
    }
  }

  decode(): Promise<void> {
    return Promise.resolve();
  }
}

export class ImageBitmap {
  width: number;
  height: number;
  closed: boolean;
  __engine_source_kind: EngineImageSourceKind;
  __engine_src: string;
  __engine_commands: CanvasDrawCommand[];
  __engine_source_width: number;
  __engine_source_height: number;
  __engine_bitmap_sx: number | undefined;
  __engine_bitmap_sy: number | undefined;
  __engine_bitmap_sw: number | undefined;
  __engine_bitmap_sh: number | undefined;

  constructor(source: EngineImageBitmapSource, crop: EngineImageBitmapCrop | null = null) {
    this.width = 0;
    this.height = 0;
    this.__engine_source_kind = "image";
    this.__engine_src = "";
    this.__engine_commands = [];
    this.__engine_source_width = 0;
    this.__engine_source_height = 0;
    this.__engine_bitmap_sx = undefined;
    this.__engine_bitmap_sy = undefined;
    this.__engine_bitmap_sw = undefined;
    this.__engine_bitmap_sh = undefined;

    if (source instanceof ImageBitmap) {
      this.__engine_source_kind = source.__engine_source_kind;
      this.__engine_src = source.__engine_src;
      this.__engine_commands = source.__engine_commands.slice();
      this.__engine_source_width = source.__engine_source_width;
      this.__engine_source_height = source.__engine_source_height;
      this.__engine_bitmap_sx = source.__engine_bitmap_sx;
      this.__engine_bitmap_sy = source.__engine_bitmap_sy;
      this.__engine_bitmap_sw = source.__engine_bitmap_sw;
      this.__engine_bitmap_sh = source.__engine_bitmap_sh;
    } else if (source instanceof OffscreenCanvas || source instanceof Canvas) {
      this.__engine_source_kind = "canvasBitmap";
      this.__engine_src = String(++engineGlobal.__engine_next_bitmap_id);
      this.__engine_commands = source.__engine_commands.slice();
      this.__engine_source_width = source.width;
      this.__engine_source_height = source.height;
      engineGlobal.__engine_draw_commands.push({
        op: "cacheCanvasBitmap",
        src: this.__engine_src,
        width: this.__engine_source_width,
        height: this.__engine_source_height,
        commands: this.__engine_commands.slice()
      });
    } else if (source instanceof EngineBlob) {
      this.__engine_source_kind = "image";
      this.__engine_src = source.__engine_image_src();
      this.__engine_commands = [];
      this.__engine_source_width = 0;
      this.__engine_source_height = 0;
    } else {
      const imageSource = source as Image;
      const src = typeof source === "string" ? source : imageSource && imageSource.src;
      if (!src) {
        throw new TypeError("ImageBitmap source must be an Image, image path, Canvas, OffscreenCanvas, or ImageBitmap");
      }
      this.__engine_source_kind = "image";
      this.__engine_src = String(src);
      this.__engine_commands = [];
      this.__engine_source_width = typeof source === "string" ? 0 : imageSource.width ?? 0;
      this.__engine_source_height = typeof source === "string" ? 0 : imageSource.height ?? 0;
    }

    if (crop) {
      this.__engine_bitmap_sx = __engine_number(crop.sx);
      this.__engine_bitmap_sy = __engine_number(crop.sy);
      this.__engine_bitmap_sw = __engine_number(crop.sw);
      this.__engine_bitmap_sh = __engine_number(crop.sh);
      this.width = Math.max(0, Math.floor(this.__engine_bitmap_sw));
      this.height = Math.max(0, Math.floor(this.__engine_bitmap_sh));
    } else {
      this.width = Math.max(0, Math.floor(this.__engine_source_width));
      this.height = Math.max(0, Math.floor(this.__engine_source_height));
    }
    this.closed = false;
  }

  close(): void {
    this.closed = true;
    this.__engine_commands = [];
  }
}

export class ImageData {
  data: Uint8ClampedArray;
  width: number;
  height: number;
  colorSpace: string;
  colorType: string;
  bytesPerPixel: number;

  constructor(
    dataOrWidth: Uint8ClampedArray | Uint8Array | ImageData | number,
    widthOrHeight?: number | Record<string, unknown>,
    heightOrSettings?: number | Record<string, unknown>,
    settings: Record<string, unknown> = {}
  ) {
    if (typeof heightOrSettings === "object" && heightOrSettings !== null) {
      settings = heightOrSettings;
      heightOrSettings = undefined;
    }
    this.colorSpace = String(settings.colorSpace ?? "srgb");
    this.colorType = String(settings.colorType ?? "rgba");
    this.bytesPerPixel = 4;

    if (dataOrWidth instanceof ImageData) {
      this.width = dataOrWidth.width;
      this.height = dataOrWidth.height;
      this.colorSpace = dataOrWidth.colorSpace;
      this.colorType = dataOrWidth.colorType;
      this.data = new Uint8ClampedArray(dataOrWidth.data);
      return;
    }

    if (typeof dataOrWidth === "number") {
      this.width = Math.floor(Math.abs(__engine_number(dataOrWidth)));
      this.height = Math.floor(Math.abs(__engine_number(widthOrHeight)));
      if (this.width <= 0 || this.height <= 0) {
        throw new RangeError("ImageData dimensions must be non-zero");
      }
      this.data = new Uint8ClampedArray(this.width * this.height * this.bytesPerPixel);
      return;
    }

    if (!(dataOrWidth instanceof Uint8ClampedArray || dataOrWidth instanceof Uint8Array)) {
      throw new TypeError("ImageData data must be a Uint8ClampedArray or Uint8Array");
    }

    this.width = Math.floor(Math.abs(__engine_number(widthOrHeight)));
    const inferredHeight = dataOrWidth.byteLength / Math.max(1, this.width) / this.bytesPerPixel;
    this.height = Math.floor(Math.abs(heightOrSettings == null ? inferredHeight : __engine_number(heightOrSettings)));
    if (this.width <= 0 || this.height <= 0) {
      throw new RangeError("ImageData dimensions must be non-zero");
    }
    if (dataOrWidth.byteLength !== this.width * this.height * this.bytesPerPixel) {
      throw new TypeError("ImageData dimensions must match buffer length");
    }
    this.data = new Uint8ClampedArray(dataOrWidth);
  }
}
