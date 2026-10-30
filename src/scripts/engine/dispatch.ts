import type { EngineGlobal } from "./types";

import type { EngineEventInit } from "../types";
import { EngineEvent } from "../event";

// we target 60 FPS
const FRAME_MS = 1000 / 60;

export function dispatchEventsMixins(engineGlobal: EngineGlobal): void {
  engineGlobal.__engine_dispatch_events = function(events: EngineEventInit[]): void {
    const target = engineGlobal.window;
    if (!target) {
      return;
    }

    for (const payload of events) {
      const event = new EngineEvent(payload.type, {
        ...payload,
        target,
        currentTarget: target,
        clientX: payload.x,
        clientY: payload.y,
        offsetX: payload.x,
        offsetY: payload.y,
        pageX: payload.pageX ?? payload.x,
        pageY: payload.pageY ?? payload.y
      });

      if (payload.coalesced) {
        const coalesced = [event];
        event.getCoalescedEvents = function() {
          return coalesced.slice();
        };
      }

      target.dispatchEvent(event);
    }
  };

  engineGlobal.__engine_dispatch_key_events = function(events: EngineEventInit[]): void {
    for (const payload of events) {
      const event = new EngineEvent(payload.type, {
        ...payload,
        target: globalThis,
        currentTarget: globalThis
      });
      engineGlobal.dispatchEvent(event);
    }
  };

  engineGlobal.__engine_dispatch_resize = function(width: number, height: number, resizeCanvas: boolean = true): void {
    const target = engineGlobal.window;
    if (!target) {
      return;
    }

    const nextWidth = Math.max(1, Math.floor(Number(width)));
    const nextHeight = Math.max(1, Math.floor(Number(height)));
    if (resizeCanvas) {
      target.width = nextWidth;
      target.height = nextHeight;
      engineGlobal.__engine_window.width = nextWidth;
      engineGlobal.__engine_window.height = nextHeight;
    }

    const init = {
      width: nextWidth,
      height: nextHeight,
      innerWidth: nextWidth,
      innerHeight: nextHeight
    };

    const canvasEvent = new EngineEvent("resize", {
      ...init,
      target,
      currentTarget: target
    });
    target.dispatchEvent(canvasEvent);

    const globalEvent = new EngineEvent("resize", {
      ...init,
      target: globalThis,
      currentTarget: globalThis
    });
    engineGlobal.dispatchEvent(globalEvent);
  };

  engineGlobal.__engine_dispatch_tick = function(timestamp: number): void {
    if (engineGlobal.__engine_profile_enabled) {
      __engine_profile_begin("tickDispatch");
    }

    try {
      const lastTimestamp = engineGlobal.__engine_last_tick_timestamp;
      const elapsedMS = lastTimestamp < 0 ? 0 : timestamp - lastTimestamp;
      const deltaTime = Math.min(elapsedMS / FRAME_MS, 1);
      engineGlobal.__engine_last_tick_timestamp = timestamp;

      const globalEvent = new EngineEvent("tick", {
        lastTime: timestamp,
        deltaTime,
        elapsedMS,
        target: globalThis,
        currentTarget: globalThis
      });
      engineGlobal.dispatchEvent(globalEvent);

      if (engineGlobal.window && (engineGlobal.window as unknown) !== engineGlobal) {
        const canvasEvent = new EngineEvent("tick", {
          lastTime: timestamp,
          deltaTime,
          elapsedMS,
          target: engineGlobal.window,
          currentTarget: engineGlobal.window
        });
        engineGlobal.window.dispatchEvent(canvasEvent);
      }
    } finally {
      if (engineGlobal.__engine_profile_enabled) {
        __engine_profile_end("tickDispatch");
      }
    }
  };
}
