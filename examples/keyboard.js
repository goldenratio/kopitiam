const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

const heldKeys = new Set();
const eventLog = [];
const player = {
  x: canvas.width / 2 - 24,
  y: canvas.height / 2 - 24,
  size: 48,
  speed: 260,
};
const keyIndicators = [
  { key: "ArrowUp", x: 96, y: 42 },
  { key: "ArrowLeft", x: 42, y: 96 },
  { key: "ArrowDown", x: 96, y: 96 },
  { key: "ArrowRight", x: 150, y: 96 },
  { key: "w", x: 610, y: 42 },
  { key: "a", x: 556, y: 96 },
  { key: "s", x: 610, y: 96 },
  { key: "d", x: 664, y: 96 },
];

function record(event) {
  const modifiers = [
    event.shiftKey && "Shift",
    event.ctrlKey && "Ctrl",
    event.altKey && "Alt",
    event.metaKey && "Meta",
  ].filter(Boolean);

  const message = `${event.type}: key=${JSON.stringify(event.key)} code=${event.code}${modifiers.length ? ` ${modifiers.join("+")}` : ""}`;
  eventLog.unshift(message);
  eventLog.length = Math.min(eventLog.length, 8);
  console.log(message);
}

globalThis.addEventListener("keydown", (event) => {
  heldKeys.add(event.key);
  record(event);
});

globalThis.addEventListener("keyup", (event) => {
  heldKeys.delete(event.key);
  record(event);
});

let lastTime = 0;

function update(event) {
  const time = event.lastTime;
  const dt = lastTime === 0 ? 0 : (time - lastTime) / 1000;
  lastTime = time;

  let dx = 0;
  let dy = 0;
  if (heldKeys.has("ArrowLeft") || heldKeys.has("a")) dx -= 1;
  if (heldKeys.has("ArrowRight") || heldKeys.has("d")) dx += 1;
  if (heldKeys.has("ArrowUp") || heldKeys.has("w")) dy -= 1;
  if (heldKeys.has("ArrowDown") || heldKeys.has("s")) dy += 1;

  if (dx !== 0 || dy !== 0) {
    const length = Math.hypot(dx, dy);
    player.x += (dx / length) * player.speed * dt;
    player.y += (dy / length) * player.speed * dt;
  }

  player.x = Math.max(0, Math.min(canvas.width - player.size, player.x));
  player.y = Math.max(0, Math.min(canvas.height - player.size, player.y));

  ctx.fillStyle = "#101820";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = "#f2aa4c";
  ctx.fillRect(player.x, player.y, player.size, player.size);

  for (const indicator of keyIndicators) {
    ctx.fillStyle = heldKeys.has(indicator.key) ? "#2dd4bf" : "#334155";
    ctx.fillRect(indicator.x, indicator.y, 42, 42);
    ctx.fillStyle = "#0f172a";
    ctx.fillRect(indicator.x + 5, indicator.y + 5, 32, 32);
  }

  const modifiers = [
    ["Shift", 42],
    ["Control", 142],
    ["Alt", 242],
    ["Meta", 342],
  ];

  for (const [key, x] of modifiers) {
    ctx.fillStyle = heldKeys.has(key) ? "#38bdf8" : "#475569";
    ctx.fillRect(x, canvas.height - 64, 80, 32);
    ctx.fillStyle = "#0f172a";
    ctx.fillRect(x + 5, canvas.height - 59, 70, 22);
  }

  for (let i = 0; i < eventLog.length; i++) {
    const width = Math.min(720, 80 + eventLog[i].length * 7);
    ctx.fillStyle = i === 0 ? "#f2aa4c" : "#64748b";
    ctx.fillRect(40, 170 + i * 24, width, 14);
  }
}

canvas.addEventListener("tick", update);

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
