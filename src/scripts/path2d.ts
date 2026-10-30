import type { EngineMatrixInit } from "./types";
import { __engine_number } from "./util";

export function __engine_is_path2d(value: unknown): value is Path2D {
  return value instanceof Path2D;
}

export function __engine_path_segments(value: Path2D | CanvasPathSegment[]): CanvasPathSegment[] {
  const segments = value instanceof Path2D ? value.__segments : value;
  return segments.map((segment) => ({ ...segment }));
}

export function __engine_radius_pair(value: unknown): { x: number; y: number } {
  if (typeof value === "number") {
    const radius = __engine_number(value);
    if (radius < 0) throw new RangeError("roundRect() radius must be non-negative");
    return { x: radius, y: radius };
  }

  if (typeof value === "object" && value !== null) {
    const point = value as Record<string, unknown>;
    const x = __engine_number(point.x);
    const y = __engine_number(point.y);
    if (x < 0 || y < 0) throw new RangeError("roundRect() radius must be non-negative");
    return { x, y };
  }

  const radius = __engine_number(value);
  if (radius < 0) throw new RangeError("roundRect() radius must be non-negative");
  return { x: radius, y: radius };
}

export function __engine_round_rect_radii(radii: unknown = 0): { x: number; y: number }[] {
  const values = Array.isArray(radii) ? radii : [radii];
  if (values.length < 1 || values.length > 4) {
    throw new RangeError("roundRect() radii must contain between 1 and 4 values");
  }

  const parsed = values.map(__engine_radius_pair);
  if (parsed.length === 1) return [parsed[0], parsed[0], parsed[0], parsed[0]];
  if (parsed.length === 2) return [parsed[0], parsed[1], parsed[0], parsed[1]];
  if (parsed.length === 3) return [parsed[0], parsed[1], parsed[2], parsed[1]];
  return parsed;
}

export function __engine_add_round_rect_path(
  path: CanvasPathSegment[],
  x: number,
  y: number,
  width: number,
  height: number,
  radii: unknown = 0
): void {
  const left = width < 0 ? x + width : x;
  const right = width < 0 ? x : x + width;
  const top = height < 0 ? y + height : y;
  const bottom = height < 0 ? y : y + height;
  const rectWidth = Math.abs(width);
  const rectHeight = Math.abs(height);
  const [topLeft, topRight, bottomRight, bottomLeft] = __engine_round_rect_radii(radii).map((radius) => ({
    x: Math.min(radius.x, rectWidth / 2),
    y: Math.min(radius.y, rectHeight / 2)
  }));
  const scale = Math.min(
    1,
    rectWidth / Math.max(1, topLeft.x + topRight.x),
    rectWidth / Math.max(1, bottomLeft.x + bottomRight.x),
    rectHeight / Math.max(1, topLeft.y + bottomLeft.y),
    rectHeight / Math.max(1, topRight.y + bottomRight.y)
  );

  for (const radius of [topLeft, topRight, bottomRight, bottomLeft]) {
    radius.x *= scale;
    radius.y *= scale;
  }

  path.push({ verb: "moveTo", x: left + topLeft.x, y: top });
  path.push({ verb: "lineTo", x: right - topRight.x, y: top });
  path.push({ verb: "quadraticCurveTo", cpx: right, cpy: top, x: right, y: top + topRight.y });
  path.push({ verb: "lineTo", x: right, y: bottom - bottomRight.y });
  path.push({ verb: "quadraticCurveTo", cpx: right, cpy: bottom, x: right - bottomRight.x, y: bottom });
  path.push({ verb: "lineTo", x: left + bottomLeft.x, y: bottom });
  path.push({ verb: "quadraticCurveTo", cpx: left, cpy: bottom, x: left, y: bottom - bottomLeft.y });
  path.push({ verb: "lineTo", x: left, y: top + topLeft.y });
  path.push({ verb: "quadraticCurveTo", cpx: left, cpy: top, x: left + topLeft.x, y: top });
  path.push({ verb: "closePath" });
}

export class Path2D {
  __segments: CanvasPathSegment[];

  constructor(path?: Path2D | string) {
    if (path == null) {
      this.__segments = [];
    } else if (path instanceof Path2D) {
      this.__segments = path.__segments.map((segment) => ({ ...segment }));
    } else {
      throw new TypeError("Path2D SVG path strings are not implemented");
    }
  }

  addPath(path: Path2D, transform?: EngineMatrixInit): void {
    if (!(path instanceof Path2D)) {
      throw new TypeError("Path2D.addPath() requires a Path2D");
    }
    if (transform != null) {
      throw new TypeError("Path2D.addPath() transform is not implemented");
    }
    this.__segments.push(...path.__segments.map((segment) => ({ ...segment })));
  }

  closePath(): void {
    this.__segments.push({ verb: "closePath" });
  }

  moveTo(x: number, y: number): void {
    this.__segments.push({ verb: "moveTo", x: __engine_number(x), y: __engine_number(y) });
  }

  lineTo(x: number, y: number): void {
    this.__segments.push({ verb: "lineTo", x: __engine_number(x), y: __engine_number(y) });
  }

  bezierCurveTo(cp1x: number, cp1y: number, cp2x: number, cp2y: number, x: number, y: number): void {
    this.__segments.push({
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
    this.__segments.push({
      verb: "quadraticCurveTo",
      cpx: __engine_number(cpx),
      cpy: __engine_number(cpy),
      x: __engine_number(x),
      y: __engine_number(y)
    });
  }

  rect(x: number, y: number, width: number, height: number): void {
    this.__segments.push({
      verb: "rect",
      x: __engine_number(x),
      y: __engine_number(y),
      width: __engine_number(width),
      height: __engine_number(height)
    });
  }

  roundRect(x: number, y: number, width: number, height: number, radii: unknown = 0): void {
    __engine_add_round_rect_path(
      this.__segments,
      __engine_number(x),
      __engine_number(y),
      __engine_number(width),
      __engine_number(height),
      radii
    );
  }

  arc(x: number, y: number, radius: number, startAngle: number, endAngle: number, counterclockwise: boolean = false): void {
    const value = __engine_number(radius);
    if (value < 0) throw new RangeError("arc() radius must be non-negative");
    this.__segments.push({
      verb: "arc",
      x: __engine_number(x),
      y: __engine_number(y),
      radius: value,
      startAngle: __engine_number(startAngle),
      endAngle: __engine_number(endAngle),
      counterclockwise: !!counterclockwise
    });
  }

  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void {
    const value = __engine_number(radius);
    if (value < 0) throw new RangeError("arcTo() radius must be non-negative");
    this.__segments.push({
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
    if (rx < 0 || ry < 0) throw new RangeError("ellipse() radius must be non-negative");
    this.__segments.push({
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
}
