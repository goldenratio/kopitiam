import type { EngineGlobal } from "./types";

function __engine_console_format(value: unknown) {
  if (typeof value === "string") {
    return value;
  }

  if (value instanceof Error) {
    return value.stack || value.message || String(value);
  }

  if (typeof value === "object" && value !== null) {
    try {
      return JSON.stringify(value);
    } catch (_) {
      return String(value);
    }
  }

  return String(value);
}

export function consoleMixin(engineGlobal: EngineGlobal): void {
  const console = {
    log(...args: unknown[]) {
      if (!engineGlobal.__engine_console_enabled) return;
      engineGlobal.__engine_console_dirty = true;
      engineGlobal.__engine_console_messages.push({
        level: "log",
        message: args.map(__engine_console_format).join(" ")
      });
    },
    info(...args: unknown[]) {
      if (!engineGlobal.__engine_console_enabled) return;
      engineGlobal.__engine_console_dirty = true;
      engineGlobal.__engine_console_messages.push({
        level: "info",
        message: args.map(__engine_console_format).join(" ")
      });
    },
    debug(...args: unknown[]) {
      if (!engineGlobal.__engine_console_enabled) return;
      engineGlobal.__engine_console_dirty = true;
      engineGlobal.__engine_console_messages.push({
        level: "debug",
        message: args.map(__engine_console_format).join(" ")
      });
    },
    warn(...args: unknown[]) {
      if (!engineGlobal.__engine_console_enabled) return;
      engineGlobal.__engine_console_dirty = true;
      engineGlobal.__engine_console_messages.push({
        level: "warn",
        message: args.map(__engine_console_format).join(" ")
      });
    },
    error(...args: unknown[]) {
      if (!engineGlobal.__engine_console_enabled) return;
      engineGlobal.__engine_console_dirty = true;
      engineGlobal.__engine_console_messages.push({
        level: "error",
        message: args.map(__engine_console_format).join(" ")
      });
    }
  };

  engineGlobal.console = console;
}
