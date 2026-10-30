const canvas = globalThis.canvas;

const ctx = canvas.getContext("2d");
let lastResize = { width: canvas.width, height: canvas.height };
let pulse = 0;

canvas.addEventListener("resize", (event) => {
  lastResize = {
    width: event.width,
    height: event.height
  };
  pulse = 1;
});

addEventListener("resize", (event) => {
  console.log(`resize ${event.innerWidth}x${event.innerHeight}`);
});

canvas.addEventListener("tick", (event) => {
  const dt = Math.min(event.deltaTime / 60, 1);
  pulse = Math.max(0, pulse - dt * 2);

  const width = canvas.width;
  const height = canvas.height;
  const inset = Math.max(18, Math.min(width, height) * 0.08);
  const boxWidth = Math.max(24, width - inset * 2);
  const boxHeight = Math.max(24, height - inset * 2);

  ctx.fillStyle = "#111820";
  ctx.fillRect(0, 0, width, height);

  ctx.fillStyle = pulse > 0 ? "#e85d75" : "#2fbf71";
  ctx.fillRect(inset, inset, boxWidth, boxHeight);

  ctx.fillStyle = "#f6f7f9";
  ctx.fillRect(width * 0.5 - 80, height * 0.5 - 20, 160, 40);

  ctx.fillStyle = "#1f2933";
  const barWidth = Math.max(20, Math.min(220, lastResize.width * 0.22));
  ctx.fillRect(width * 0.5 - barWidth * 0.5, height * 0.5 - 6, barWidth, 12);
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
