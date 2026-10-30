export function __engine_bytes_to_base64(bytes: number[]): string {
  const table = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let output = "";
  for (let index = 0; index < bytes.length; index += 3) {
    const b0 = bytes[index] ?? 0;
    const b1 = bytes[index + 1] ?? 0;
    const b2 = bytes[index + 2] ?? 0;
    output += table[b0 >> 2];
    output += table[((b0 & 0x03) << 4) | (b1 >> 4)];
    output += index + 1 < bytes.length ? table[((b1 & 0x0f) << 2) | (b2 >> 6)] : "=";
    output += index + 2 < bytes.length ? table[b2 & 0x3f] : "=";
  }

  return output;
}

export function __engine_blob_part_bytes(part: unknown): number[] {
  if (typeof part === "string") {
    const bytes: number[] = [];
    for (let index = 0; index < part.length; index++) {
      bytes.push(part.charCodeAt(index) & 0xff);
    }
    return bytes;
  }

  if (part instanceof Uint8Array || part instanceof Uint8ClampedArray) {
    return Array.from(part, (byte) => Number(byte) & 0xff);
  }

  if (part instanceof ArrayBuffer) {
    return Array.from(new Uint8Array(part), (byte) => Number(byte) & 0xff);
  }

  if (Array.isArray(part)) {
    return part.map((byte) => Number(byte) & 0xff);
  }

  if (part instanceof EngineBlob) {
    return part.__engine_bytes.slice();
  }

  if (part && typeof part === "object" && "length" in part) {
    return Array.from(part as ArrayLike<unknown>, (byte) => Number(byte) & 0xff);
  }

  return [];
}

export class EngineBlob {
  __engine_src: string;
  __engine_text: string | undefined;
  __engine_bytes: number[];
  type: string;
  size: number;

  constructor(parts: unknown[] = [], options: Record<string, unknown> = {}) {
    this.__engine_src = "";
    this.__engine_text = undefined;
    this.__engine_bytes = [];
    this.type = String(options.type ?? "");

    const sourceParts = Array.isArray(parts) ? parts : [parts];
    for (const part of sourceParts) {
      this.__engine_bytes.push(...__engine_blob_part_bytes(part));
    }
    this.size = this.__engine_bytes.length;
  }

  static fromUrl(src: string, text: string | undefined = undefined, type: string = ""): EngineBlob {
    const blob = new EngineBlob([], { type });
    blob.__engine_src = String(src);
    blob.__engine_text = text;
    blob.size = text?.length ?? 0;
    return blob;
  }

  __engine_image_src(): string {
    if (!this.__engine_src) {
      const mimeType = this.type || "application/octet-stream";
      this.__engine_src = `data:${mimeType};base64,${__engine_bytes_to_base64(this.__engine_bytes)}`;
    }
    return this.__engine_src;
  }

  async text(): Promise<string> {
    if (this.__engine_text === undefined) {
      this.__engine_text = this.__engine_src
        ? __engine_fetch(this.__engine_src)
        : this.__engine_bytes.map((byte) => String.fromCharCode(byte)).join("");
    }
    return this.__engine_text;
  }
}

export class EngineFetchResponse {
  ok: boolean;
  status: number;
  statusText: string;
  url: string;
  redirected: boolean;
  type: string;
  bodyUsed: boolean;
  __engine_text: string | undefined;

  constructor(url: string) {
    this.ok = true;
    this.status = 200;
    this.statusText = "OK";
    this.url = String(url);
    this.redirected = false;
    this.type = "basic";
    this.bodyUsed = false;
    this.__engine_text = undefined;
  }

  __engine_read_text(): string {
    if (this.__engine_text === undefined) {
      this.__engine_text = __engine_fetch(this.url);
    }
    return this.__engine_text;
  }

  async text(): Promise<string> {
    this.bodyUsed = true;
    return this.__engine_read_text();
  }

  async json(): Promise<unknown> {
    this.bodyUsed = true;
    return JSON.parse(this.__engine_read_text());
  }

  async blob(): Promise<EngineBlob> {
    this.bodyUsed = true;
    return EngineBlob.fromUrl(this.url, this.__engine_text);
  }
}
