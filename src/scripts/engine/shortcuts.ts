import { ImageBitmap } from "../image";
import { EngineResolvedPromise } from "../blob";
import { engineGlobal } from "./engine-global";

engineGlobal.clear = function clear(color: string = "#000000"): void {
  const ctx = engineGlobal.window!.getContext("2d")!;
  ctx.save();
  ctx.resetTransform();
  ctx.fillStyle = color;
  ctx.globalAlpha = 1;
  ctx.fillRect(0, 0, engineGlobal.window!.width, engineGlobal.window!.height);
  ctx.restore();
};


engineGlobal.fillRect = function fillRect(x: number, y: number, width: number, height: number, color: string = "#ffffff"): void {
  const ctx = engineGlobal.window!.getContext("2d")!;
  ctx.fillStyle = color;
  ctx.fillRect(x, y, width, height);
};


engineGlobal.strokeRect = function strokeRect(x: number, y: number, width: number, height: number, color: string = "#ffffff", lineWidth: number = 1): void {
  const ctx = engineGlobal.window!.getContext("2d")!;
  ctx.strokeStyle = color;
  ctx.lineWidth = lineWidth;
  ctx.strokeRect(x, y, width, height);
};

engineGlobal.createImageBitmap = function createImageBitmap(source: EngineImageBitmapSource, ...args: number[]): EngineResolvedPromise<ImageBitmap> {
  let crop = null;
  if (args.length >= 4) {
    crop = {
      sx: args[0],
      sy: args[1],
      sw: args[2],
      sh: args[3]
    };
  }
  return new EngineResolvedPromise(new ImageBitmap(source, crop));
};
