const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

const sprite = new OffscreenCanvas(160, 160);
const sctx = sprite.getContext("2d");
const tile = new Image();
tile.src = "examples/tile_0007.png";

sctx.fillStyle = "#0f172a";
sctx.fillRect(0, 0, sprite.width, sprite.height);
sctx.fillStyle = "#38bdf8";
sctx.beginPath();
sctx.arc(80, 80, 70, 0, Math.PI * 2);
sctx.fill();

for (let y = 20; y < 140; y += 24) {
  for (let x = 20; x < 140; x += 24) {
    sctx.drawImage(tile, x, y, 24, 24);
  }
}

sctx.strokeStyle = "#f8fafc";
sctx.lineWidth = 6;
sctx.strokeRect(8, 8, 144, 144);

let angle = 0;

canvas.addEventListener("tick", () => {
  angle += 0.025;

  ctx.fillStyle = "#111827";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.drawImage(sprite, 72, 72, 160, 160);

  ctx.save();
  ctx.translate(440, 225);
  ctx.rotate(angle);
  ctx.drawImage(sprite, -120, -120, 240, 240);
  ctx.restore();

  ctx.drawImage(sprite, 20, 20, 120, 120, 592, 96, 160, 160);
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
