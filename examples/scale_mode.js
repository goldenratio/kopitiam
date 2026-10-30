// "responsive", "exactFit", "showAll", "noBorder", "noScale"
const scaleMode = "showAll";

const canvas = globalThis.canvas;

const ctx = canvas.getContext("2d");
let windowSize = { width: canvas.width, height: canvas.height };
let pointer = null;

addEventListener("resize", (event) => {
  windowSize = {
    width: event.innerWidth,
    height: event.innerHeight
  };
});

canvas.addEventListener("pointermove", (event) => {
  pointer = { x: event.offsetX, y: event.offsetY };
});

canvas.addEventListener("pointerleave", () => {
  pointer = null;
});

canvas.addEventListener("tick", (event) => {
  const time = event.lastTime / 1000;
  const w = canvas.width;
  const h = canvas.height;

  ctx.fillStyle = "#101820";
  ctx.fillRect(0, 0, w, h);

  ctx.fillStyle = "#263646";
  for (let x = 0; x < w; x += 40) {
    ctx.fillRect(x, 0, 2, h);
  }
  for (let y = 0; y < h; y += 40) {
    ctx.fillRect(0, y, w, 2);
  }

  ctx.fillStyle = "#f2c14e";
  ctx.fillRect(0, 0, 96, 64);
  ctx.fillStyle = "#e85d75";
  ctx.fillRect(w - 96, 0, 96, 64);
  ctx.fillStyle = "#2fbf71";
  ctx.fillRect(0, h - 64, 96, 64);
  ctx.fillStyle = "#55dde0";
  ctx.fillRect(w - 96, h - 64, 96, 64);

  const cx = w * 0.5 + Math.cos(time) * 120;
  const cy = h * 0.5 + Math.sin(time * 1.3) * 70;
  ctx.fillStyle = "#f6f7f9";
  ctx.fillRect(cx - 28, cy - 28, 56, 56);

  if (pointer) {
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(pointer.x - 6, pointer.y - 6, 12, 12);
  }

  ctx.fillStyle = "#1f2933";
  ctx.fillRect(24, h * 0.5 - 24, 250, 48);
  ctx.fillStyle = "#f6f7f9";
  ctx.fillRect(36, h * 0.5 - 6, Math.min(210, windowSize.width * 0.12), 12);
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
