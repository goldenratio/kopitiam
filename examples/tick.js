const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

const rect = {
  x: 40,
  y: 175,
  width: 90,
  height: 90,
  speed: 280,
  direction: 1,
};

canvas.addEventListener("tick", (event) => {
  const deltaSeconds = event.deltaTime / 1000;
  rect.x += rect.speed * rect.direction * deltaSeconds;

  if (rect.x <= 0) {
    rect.x = 0;
    rect.direction = 1;
  }

  if (rect.x + rect.width >= canvas.width) {
    rect.x = canvas.width - rect.width;
    rect.direction = -1;
  }

  ctx.fillStyle = "#111827";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = "red";
  ctx.fillRect(rect.x, rect.y, rect.width, rect.height);
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
