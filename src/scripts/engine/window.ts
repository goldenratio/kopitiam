import type { EngineGlobal } from "./types";

import { __engine_call_profiled_tick_listener, EngineEvent } from "../event";
import type { EngineListener } from "../types";

export function windowMixins(engineGlobal: EngineGlobal): void {

  engineGlobal.exit = function(): void {
    engineGlobal.__engine_draw_commands.push({ op: "exit" });
  };

  engineGlobal.addEventListener = function(type: string, listener: EngineListener): void {
    engineGlobal.__engine_global_event_target.addEventListener(type, listener);
  };

  engineGlobal.removeEventListener = function(type: string, listener: EngineListener): void {
    engineGlobal.__engine_global_event_target.removeEventListener(type, listener);
  };

  engineGlobal.dispatchEvent = function(event: EngineEvent): boolean {
    const propertyHandler = event && event.type ? engineGlobal[`on${event.type}`] : null;
    if (typeof propertyHandler === "function") {
      event.currentTarget = globalThis;
      __engine_call_profiled_tick_listener(propertyHandler, globalThis, event);
    }
    return engineGlobal.__engine_global_event_target.dispatchEvent(event);
  };

  engineGlobal.PointerEvent = EngineEvent;
  engineGlobal.KeyboardEvent = EngineEvent;

  engineGlobal.performance = engineGlobal.performance || {
    now() {
      return Date.now();
    }
  };

  engineGlobal.devicePixelRatio = engineGlobal.devicePixelRatio || 1;
}
