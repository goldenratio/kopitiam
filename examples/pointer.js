globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});

const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

const pointer = {
  x: canvas.width / 2,
  y: canvas.height / 2,
  down: false,
  inside: false,
};

canvas.addEventListener("pointerenter", () => {
  pointer.inside = true;
});

canvas.addEventListener("pointerleave", () => {
  pointer.inside = false;
  pointer.down = false;
});

canvas.addEventListener("pointermove", (event) => {
  pointer.x = event.offsetX;
  pointer.y = event.offsetY;
});

canvas.addEventListener("pointerdown", (event) => {
  pointer.down = true;
  pointer.x = event.offsetX;
  pointer.y = event.offsetY;
});

canvas.addEventListener("pointerup", () => {
  pointer.down = false;
});

canvas.addEventListener("tick", () => {
  ctx.fillStyle = "#111827";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = pointer.down ? "#ef4444" : pointer.inside ? "#22c55e" : "#64748b";
  ctx.fillRect(pointer.x - 8, pointer.y - 8, 16, 16);

  ctx.beginPath();
  ctx.arc(pointer.x, pointer.y, pointer.down ? 42 : 28, 0, Math.PI * 2);
  ctx.fill();
});
