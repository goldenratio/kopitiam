import { engineGlobal } from "./engine";
import type { EngineListener, EngineListenerMap, EngineEventInit } from "./types";

export function __engine_call_profiled_tick_listener(listener: EngineListener, thisArg: unknown, event: EngineEvent): void {
  const profile_enabled = engineGlobal.__engine_profile_enabled === true;
  if (!profile_enabled || event.type !== "tick") {
    listener.call(thisArg, event);
    return;
  }

  __engine_profile_begin("tickListener");
  try {
    listener.call(thisArg, event);
  } finally {
    __engine_profile_end("tickListener");
  }
}

export class EngineEvent {
  type: string;
  target: unknown;
  currentTarget: unknown;
  defaultPrevented: boolean;
  getCoalescedEvents?: () => void;

  constructor(type: string, init: EngineEventInit = {}) {
    Object.assign(this, init);
    this.type = type;
    this.target = init.target ?? null;
    this.currentTarget = init.currentTarget ?? null;
    this.defaultPrevented = false;
  }

  preventDefault(): void {
    this.defaultPrevented = true;
  }

  stopPropagation(): void { }
}

export class EngineEventTarget {
  [key: string]: any;
  __listeners: EngineListenerMap;

  constructor() {
    this.__listeners = Object.create(null);
  }

  addEventListener(type: string, listener: EngineListener): void {
    if (typeof listener !== "function") {
      return;
    }
    const name = String(type);
    (this.__listeners[name] ??= []).push(listener);
  }

  removeEventListener(type: string, listener: EngineListener): void {
    const name = String(type);
    const listeners = this.__listeners[name];
    if (!listeners) {
      return;
    }
    this.__listeners[name] = listeners.filter((candidate) => candidate !== listener);
  }

  dispatchEvent(event: EngineEvent): boolean {
    if (!event || !event.type) {
      throw new TypeError("dispatchEvent requires an event with a type");
    }

    event.target ??= this;
    event.currentTarget = this;

    const propertyHandler = this["on" + event.type];
    if (typeof propertyHandler === "function") {
      __engine_call_profiled_tick_listener(propertyHandler, this, event);
    }

    const listeners = this.__listeners[event.type] ?? [];
    if (listeners.length === 1) {
      // Avoid allocating an iterator and snapshot array for the common case.
      // Keep the snapshot behavior when multiple listeners are present because
      // listeners may add/remove handlers while dispatching.
      __engine_call_profiled_tick_listener(listeners[0], this, event);
    } else {
      for (const listener of listeners.slice()) {
        __engine_call_profiled_tick_listener(listener, this, event);
      }
    }

    return !event.defaultPrevented;
  }
}
