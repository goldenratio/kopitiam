const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

let angle = 0;

canvas.addEventListener("tick", (event) => {
  angle += event.deltaTime * 0.002;

  ctx.fillStyle = "#101820";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.save();
  ctx.beginPath();
  ctx.arc(canvas.width / 2, canvas.height / 2, 120, 0, Math.PI * 2);
  ctx.clip();

  ctx.fillStyle = "#f97316";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = "#2563eb";
  ctx.translate(canvas.width / 2, canvas.height / 2);
  ctx.rotate(angle);
  ctx.fillRect(-180, -40, 360, 80);
  ctx.restore();

  ctx.strokeStyle = "#f8fafc";
  ctx.lineWidth = 4;
  ctx.beginPath();
  ctx.arc(canvas.width / 2, canvas.height / 2, 120, 0, Math.PI * 2);
  ctx.stroke();
});


globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
