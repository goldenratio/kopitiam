import { DOMMatrix } from "./geometry";

export type EngineListener = (event: any) => void;
export type EngineListenerMap = Record<string, EngineListener[]>;
export type EngineEventInit = Record<string, any>;
export type EngineMatrixInit = DOMMatrix | Partial<Record<"a" | "b" | "c" | "d" | "e" | "f" | "m11" | "m12" | "m21" | "m22" | "m41" | "m42", number>> | number[];
export type EnginePointInit = Partial<Record<"x" | "y" | "z" | "w", number>>;
