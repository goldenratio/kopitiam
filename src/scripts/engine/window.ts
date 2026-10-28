import { EngineListener } from "../types";
import { __engine_call_profiled_tick_listener, EngineEvent, EngineEventTarget } from "../event";
import { engineGlobal } from "./engine-global";

engineGlobal.exit = function exit(): void {
  engineGlobal.__engine_draw_commands.push({ op: "exit" });
};

engineGlobal.__engine_global_event_target = new EngineEventTarget();

engineGlobal.addEventListener = function addEventListener(type: string, listener: EngineListener): void {
  engineGlobal.__engine_global_event_target.addEventListener(type, listener);
};

engineGlobal.removeEventListener = function removeEventListener(type: string, listener: EngineListener): void {
  engineGlobal.__engine_global_event_target.removeEventListener(type, listener);
};

engineGlobal.dispatchEvent = function dispatchEvent(event: EngineEvent): boolean {
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
