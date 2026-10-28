const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

const panel = new Path2D();
panel.roundRect(48, 48, 704, 354, [
  { x: 36, y: 36 },
  { x: 72, y: 24 },
  { x: 24, y: 72 },
  48,
]);

const orbit = new Path2D();
orbit.ellipse(400, 225, 242, 108, -0.2, 0, Math.PI * 2);

const badge = new Path2D();
badge.roundRect(300, 162, 200, 126, 28);
badge.moveTo(338, 244);
badge.arcTo(400, 310, 462, 244, 56);
badge.arcTo(400, 178, 338, 244, 56);
badge.closePath();

const petal = new Path2D();
petal.ellipse(0, -78, 28, 78, 0.45, 0, Math.PI * 2);

const flower = new Path2D();
for (let i = 0; i < 8; i += 1) {
  const angle = (Math.PI * 2 * i) / 8;
  flower.ellipse(
    604 + Math.cos(angle) * 42,
    230 + Math.sin(angle) * 42,
    20,
    58,
    angle,
    0,
    Math.PI * 2
  );
}

const swoosh = new Path2D();
swoosh.moveTo(124, 318);
swoosh.arcTo(210, 196, 326, 316, 84);
swoosh.arcTo(420, 410, 520, 300, 64);
swoosh.arcTo(600, 210, 696, 296, 52);

const clonedSwoosh = new Path2D(swoosh);

canvas.addEventListener("tick", (event) => {
  const time = event.timestamp * 0.001;

  ctx.fillStyle = "#101820";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  const background = ctx.createLinearGradient(48, 48, 752, 402);
  background.addColorStop(0, "#0ea5e9");
  background.addColorStop(0.45, "#22c55e");
  background.addColorStop(1, "#f97316");

  ctx.save();
  ctx.clip(panel);
  ctx.fillStyle = background;
  ctx.fillRect(48, 48, 704, 354);

  ctx.strokeStyle = "#ffffff52";
  ctx.lineWidth = 2;
  for (let i = -120; i < canvas.width + 50; i += 28) {
    ctx.beginPath();
    ctx.moveTo(i + Math.sin(time) * 16, 48);
    ctx.lineTo(i - 180 + Math.sin(time) * 16, 402);
    ctx.stroke();
  }

  ctx.strokeStyle = "#f8fafc";
  ctx.lineWidth = 5;
  ctx.stroke(orbit);

  ctx.fillStyle = "#0f172ab8";
  ctx.fill(badge);
  ctx.strokeStyle = "#facc15";
  ctx.lineWidth = 4;
  ctx.stroke(badge);

  ctx.save();
  ctx.translate(400, 225);
  ctx.rotate(time);
  ctx.fillStyle = "#f8fafc";
  ctx.beginPath();
  ctx.ellipse(0, 0, 74, 34, 0.3, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  ctx.fillStyle = "#ec4899c7";
  ctx.fill(flower, "evenodd");

  ctx.strokeStyle = "#1e293b";
  ctx.lineWidth = 10;
  ctx.stroke(clonedSwoosh);
  ctx.strokeStyle = "#f8fafc";
  ctx.lineWidth = 4;
  ctx.stroke(swoosh);

  ctx.restore();

  ctx.strokeStyle = "#f8fafc";
  ctx.lineWidth = 4;
  ctx.stroke(panel);
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
