const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

async function main() {
  const tile = new Image();
  tile.src = "examples/tile_0007.png";

  const croppedTile = await createImageBitmap(tile, 4, 4, 16, 16);

  const offscreen = new OffscreenCanvas(180, 180);
  const octx = offscreen.getContext("2d");

  octx.fillStyle = "#0f172a";
  octx.fillRect(0, 0, offscreen.width, offscreen.height);
  octx.fillStyle = "#f59e0b";
  octx.beginPath();
  octx.arc(90, 90, 78, 0, Math.PI * 2);
  octx.fill();
  octx.drawImage(tile, 42, 42, 96, 96);

  const offscreenBitmap = await createImageBitmap(offscreen, 24, 24, 132, 132);

  let angle = 0;

  canvas.addEventListener("tick", () => {
    angle += 0.025;

    ctx.fillStyle = "#111827";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.drawImage(croppedTile, 80, 84, 128, 128);

    for (let x = 80; x < 720; x += 32) {
      ctx.drawImage(croppedTile, x, 320, 32, 32);
    }

    ctx.drawImage(offscreenBitmap, 266, 70, 180, 180);

    ctx.save();
    ctx.translate(600, 160);
    ctx.rotate(angle);
    ctx.drawImage(offscreenBitmap, -90, -90, 180, 180);
    ctx.restore();
  });
}

main();

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
