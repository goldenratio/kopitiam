import { ImageBitmap } from "../image";
import { engineGlobal } from "./engine-global";

engineGlobal.createImageBitmap = function createImageBitmap(source: EngineImageBitmapSource, ...args: number[]): Promise<ImageBitmap> {
  let crop = null;
  if (args.length >= 4) {
    crop = {
      sx: args[0],
      sy: args[1],
      sw: args[2],
      sh: args[3]
    };
  }
  return Promise.resolve(new ImageBitmap(source, crop));
};
