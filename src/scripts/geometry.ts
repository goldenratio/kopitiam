import type { EngineMatrixInit, EnginePointInit } from "./types";
import { __engine_number } from "./util";

export class DOMPoint {
  x: number;
  y: number;
  z: number;
  w: number;

  constructor(x: unknown = 0, y: unknown = 0, z: unknown = 0, w: unknown = 1) {
    this.x = __engine_number(x);
    this.y = __engine_number(y);
    this.z = __engine_number(z);
    this.w = __engine_number(w, 1);
  }
}

export class DOMMatrix {
  a: number;
  b: number;
  c: number;
  d: number;
  e: number;
  f: number;
  is2D: boolean;

  constructor(init: EngineMatrixInit | undefined = undefined) {
    this.a = 1;
    this.b = 0;
    this.c = 0;
    this.d = 1;
    this.e = 0;
    this.f = 0;
    this.is2D = true;

    if (Array.isArray(init)) {
      if (init.length === 6) {
        [this.a, this.b, this.c, this.d, this.e, this.f] = init.map(Number);
      } else if (init.length >= 16) {
        this.a = Number(init[0]);
        this.b = Number(init[1]);
        this.c = Number(init[4]);
        this.d = Number(init[5]);
        this.e = Number(init[12]);
        this.f = Number(init[13]);
      }
    } else if (init && typeof init === "object") {
      this.a = __engine_number(init.a ?? init.m11, 1);
      this.b = __engine_number(init.b ?? init.m12, 0);
      this.c = __engine_number(init.c ?? init.m21, 0);
      this.d = __engine_number(init.d ?? init.m22, 1);
      this.e = __engine_number(init.e ?? init.m41, 0);
      this.f = __engine_number(init.f ?? init.m42, 0);
    }
  }

  get m11(): number { return this.a; }
  set m11(value: unknown) { this.a = __engine_number(value, 1); }
  get m12(): number { return this.b; }
  set m12(value: unknown) { this.b = __engine_number(value); }
  get m21(): number { return this.c; }
  set m21(value: unknown) { this.c = __engine_number(value); }
  get m22(): number { return this.d; }
  set m22(value: unknown) { this.d = __engine_number(value, 1); }
  get m41(): number { return this.e; }
  set m41(value: unknown) { this.e = __engine_number(value); }
  get m42(): number { return this.f; }
  set m42(value: unknown) { this.f = __engine_number(value); }

  multiply(other: EngineMatrixInit = new DOMMatrix()): DOMMatrix {
    const rhs = other instanceof DOMMatrix ? other : new DOMMatrix(other);
    const result = new DOMMatrix();
    result.a = this.a * rhs.a + this.c * rhs.b;
    result.b = this.b * rhs.a + this.d * rhs.b;
    result.c = this.a * rhs.c + this.c * rhs.d;
    result.d = this.b * rhs.c + this.d * rhs.d;
    result.e = this.a * rhs.e + this.c * rhs.f + this.e;
    result.f = this.b * rhs.e + this.d * rhs.f + this.f;
    return result;
  }

  multiplySelf(other: EngineMatrixInit = new DOMMatrix()): DOMMatrix {
    const result = this.multiply(other);
    return this.__copyFrom(result);
  }

  translate(tx: number = 0, ty: number = 0): DOMMatrix {
    return this.multiply(new DOMMatrix([1, 0, 0, 1, tx, ty]));
  }

  translateSelf(tx: number = 0, ty: number = 0): DOMMatrix {
    return this.__copyFrom(this.translate(tx, ty));
  }

  scale(scaleX: number = 1, scaleY: number = scaleX, _scaleZ: number = 1, originX: number = 0, originY: number = 0): DOMMatrix {
    return this
      .translate(originX, originY)
      .multiply(new DOMMatrix([scaleX, 0, 0, scaleY, 0, 0]))
      .translate(-originX, -originY);
  }

  scaleSelf(scaleX: number = 1, scaleY: number = scaleX, scaleZ: number = 1, originX: number = 0, originY: number = 0): DOMMatrix {
    return this.__copyFrom(this.scale(scaleX, scaleY, scaleZ, originX, originY));
  }

  rotate(rotZ: number = 0): DOMMatrix {
    const radians = __engine_number(rotZ) * Math.PI / 180;
    const cos = Math.cos(radians);
    const sin = Math.sin(radians);
    return this.multiply(new DOMMatrix([cos, sin, -sin, cos, 0, 0]));
  }

  rotateSelf(rotZ: number = 0): DOMMatrix {
    return this.__copyFrom(this.rotate(rotZ));
  }

  transformPoint(point: EnginePointInit = {}) {
    const x = __engine_number(point.x);
    const y = __engine_number(point.y);
    const z = __engine_number(point.z);
    const w = __engine_number(point.w, 1);
    return new DOMPoint(
      x * this.a + y * this.c + this.e,
      x * this.b + y * this.d + this.f,
      z,
      w
    );
  }

  toFloat32Array(): Float32Array {
    return new Float32Array([this.a, this.b, 0, 0, this.c, this.d, 0, 0, 0, 0, 1, 0, this.e, this.f, 0, 1]);
  }

  toFloat64Array(): Float64Array {
    return new Float64Array([this.a, this.b, 0, 0, this.c, this.d, 0, 0, 0, 0, 1, 0, this.e, this.f, 0, 1]);
  }

  __copyFrom(other: DOMMatrix): DOMMatrix {
    this.a = other.a;
    this.b = other.b;
    this.c = other.c;
    this.d = other.d;
    this.e = other.e;
    this.f = other.f;
    return this;
  }
}
