const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

const patternCanvas = new OffscreenCanvas(48, 48);
const pctx = patternCanvas.getContext("2d");

pctx.fillStyle = "#111827";
pctx.fillRect(0, 0, patternCanvas.width, patternCanvas.height);
pctx.fillStyle = "#f8fafc";
pctx.fillRect(0, 0, 24, 24);
pctx.fillStyle = "#38bdf8";
pctx.fillRect(24, 24, 24, 24);
pctx.strokeStyle = "#f59e0b";
pctx.lineWidth = 4;
pctx.beginPath();
pctx.moveTo(0, 48);
pctx.lineTo(48, 0);
pctx.stroke();

const pattern = ctx.createPattern(patternCanvas, "repeat");

function roundedPanel(x, y, width, height) {
  ctx.beginPath();
  ctx.moveTo(x + 16, y);
  ctx.lineTo(x + width - 16, y);
  ctx.quadraticCurveTo(x + width, y, x + width, y + 16);
  ctx.lineTo(x + width, y + height - 16);
  ctx.quadraticCurveTo(x + width, y + height, x + width - 16, y + height);
  ctx.lineTo(x + 16, y + height);
  ctx.quadraticCurveTo(x, y + height, x, y + height - 16);
  ctx.lineTo(x, y + 16);
  ctx.quadraticCurveTo(x, y, x + 16, y);
  ctx.closePath();
}

canvas.addEventListener("tick", (event) => {
  const time = event.timestamp * 0.001;

  ctx.fillStyle = "#0f172a";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  const background = ctx.createLinearGradient(0, 0, canvas.width, canvas.height);
  background.addColorStop(0, "#1d4ed8");
  background.addColorStop(0.5, "#14b8a6");
  background.addColorStop(1, "#f97316");
  ctx.fillStyle = background;
  roundedPanel(36, 36, 728, 132);
  ctx.fill();

  const radial = ctx.createRadialGradient(208, 300, 8, 208, 300, 112);
  radial.addColorStop(0, "#ffffff");
  radial.addColorStop(0.28, "#facc15");
  radial.addColorStop(1, "#ef4444");
  ctx.fillStyle = radial;
  ctx.beginPath();
  ctx.arc(208, 300, 112, 0, Math.PI * 2);
  ctx.fill();

  const conic = ctx.createConicGradient(time, 404, 300);
  conic.addColorStop(0, "#ec4899");
  conic.addColorStop(0.25, "#8b5cf6");
  conic.addColorStop(0.5, "#06b6d4");
  conic.addColorStop(0.75, "#22c55e");
  conic.addColorStop(1, "#ec4899");
  ctx.fillStyle = conic;
  ctx.beginPath();
  ctx.arc(404, 300, 112, 0, Math.PI * 2);
  ctx.fill();

  if (pattern) {
    pattern.setTransform({
      a: Math.cos(time) * 0.8,
      b: Math.sin(time) * 0.8,
      c: -Math.sin(time) * 0.8,
      d: Math.cos(time) * 0.8,
      e: 600,
      f: 300,
    });

    ctx.fillStyle = pattern;
    roundedPanel(548, 188, 180, 224);
    ctx.fill();
  }

  ctx.strokeStyle = "#f8fafc";
  ctx.lineWidth = 4;
  roundedPanel(36, 36, 728, 132);
  ctx.stroke();
  ctx.beginPath();
  ctx.arc(208, 300, 112, 0, Math.PI * 2);
  ctx.arc(404, 300, 112, 0, Math.PI * 2);
  ctx.stroke();
  roundedPanel(548, 188, 180, 224);
  ctx.stroke();
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
