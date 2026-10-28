export function __engine_number(value: unknown, fallback = 0) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}

export function __engine_string(value: unknown, fallback: string) {
  return value == null ? fallback : String(value);
}

export function __engine_same_optional_number(left: unknown, right: unknown): boolean {
  return (left == null && right == null) || Number(left) === Number(right);
}
