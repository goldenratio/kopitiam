const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

const tile = new Image();
tile.src = "examples/tile_0007.png";
let angle = 0;

canvas.addEventListener("tick", () => {
  angle += 0.03;
  const panelX = 48;
  const panelY = 48;
  const panelWidth = canvas.width - panelX * 2;
  const panelHeight = canvas.height - panelY * 2;

  ctx.fillStyle = "#101820";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = "#1f2937";
  ctx.fillRect(panelX, panelY, panelWidth, panelHeight);

  ctx.drawImage(tile, 80, 80, 96, 96);
  ctx.drawImage(tile, 208, 80, 144, 144);

  ctx.save();
  ctx.translate(500, 170);
  ctx.rotate(angle);
  ctx.drawImage(tile, -72, -72, 144, 144);
  ctx.restore();

  ctx.drawImage(tile, 4, 4, 16, 16, 608, 88, 128, 128);

  for (let y = 292; y < panelY + panelHeight - 14; y += 24) {
    for (let x = 80; x < panelX + panelWidth - 16; x += 24) {
      ctx.drawImage(tile, x, y, 24, 24);
    }
  }
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
