const canvas = globalThis.canvas;

const ctx = canvas.getContext("2d");
const tile = new Image();
tile.src = "examples/tile_0007.png";

const modes = [
  "source-over",
  "source-in",
  "source-out",
  "source-atop",
  "destination-over",
  "destination-in",
  "destination-out",
  "destination-atop",
  "lighter",
  "copy",
  "xor",
  "multiply",
  "screen",
  "overlay",
  "darken",
  "lighten",
  "color-dodge",
  "color-burn",
  "hard-light",
  "soft-light",
  "difference",
  "exclusion",
  "hue",
  "saturation",
  "color",
  "luminosity"
];

console.log(`blend modes: ${modes.join(", ")}`);

function panelRect(index) {
  const columns = 7;
  const gap = 12;
  const margin = 24;
  const panelWidth = (canvas.width - margin * 2 - gap * (columns - 1)) / columns;
  const rows = Math.ceil(modes.length / columns);
  const panelHeight = (canvas.height - 52 - gap * (rows - 1)) / rows;
  const x = margin + (index % columns) * (panelWidth + gap);
  const y = 28 + Math.floor(index / columns) * (panelHeight + gap);
  return { x, y, width: panelWidth, height: panelHeight };
}

function drawMode(index, mode, time) {
  const panel = panelRect(index);
  const cx = panel.x + panel.width * 0.5;
  const cy = panel.y + panel.height * 0.52;
  const size = Math.min(panel.width, panel.height) * 0.62;
  const pulse = Math.sin(time * 2 + index * 0.7) * 5;

  ctx.save();

  ctx.globalCompositeOperation = "source-over";
  ctx.fillStyle = "#18212b";
  ctx.fillRect(panel.x, panel.y, panel.width, panel.height);

  ctx.fillStyle = "#293747";
  ctx.fillRect(panel.x + 6, panel.y + 6, panel.width - 12, panel.height - 12);

  ctx.fillStyle = "#e85d75";
  ctx.fillRect(panel.x + 16, panel.y + 24, panel.width * 0.58, panel.height * 0.62);

  ctx.fillStyle = "#2fbf71";
  ctx.fillRect(panel.x + panel.width * 0.34, panel.y + 42, panel.width * 0.52, panel.height * 0.52);

  ctx.globalCompositeOperation = mode;
  ctx.globalAlpha = 0.9;

  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(Math.sin(time + index) * 0.16);
  ctx.drawImage(tile, -size * 0.5, -size * 0.5 + pulse, size, size);
  ctx.restore();

  ctx.globalAlpha = 1;
  ctx.globalCompositeOperation = "source-over";

  ctx.restore();
}

canvas.addEventListener("tick", (event) => {
  const time = event.lastTime / 1000;

  ctx.globalCompositeOperation = "source-over";
  ctx.globalAlpha = 1;
  ctx.fillStyle = "#0f141b";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  for (let i = 0; i < modes.length; i++) {
    drawMode(i, modes[i], time);
  }
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
