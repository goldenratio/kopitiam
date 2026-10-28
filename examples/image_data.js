const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");

const source = ctx.createImageData(180, 180);
for (let y = 0; y < source.height; y += 1) {
  for (let x = 0; x < source.width; x += 1) {
    const index = (y * source.width + x) * 4;
    const dx = x - source.width / 2;
    const dy = y - source.height / 2;
    const distance = Math.sqrt(dx * dx + dy * dy);
    source.data[index] = Math.max(0, 255 - distance * 2.4);
    source.data[index + 1] = (x / source.width) * 255;
    source.data[index + 2] = (y / source.height) * 255;
    source.data[index + 3] = distance < 86 ? 255 : 0;
  }
}

const scratch = ctx.createImageData(source);
for (let i = 0; i < scratch.data.length; i += 4) {
  scratch.data[i] = 255 - source.data[i];
  scratch.data[i + 1] = 255 - source.data[i + 1];
  scratch.data[i + 2] = 255 - source.data[i + 2];
  scratch.data[i + 3] = source.data[i + 3];
}

ctx.putImageData(source, 0, 0);
const captured = ctx.getImageData(0, 0, 180, 180);
ctx.clearRect(0, 0, 180, 180);

for (let i = 0; i < captured.data.length; i += 4) {
  captured.data[i] = 255 - captured.data[i];
  captured.data[i + 1] = 255 - captured.data[i + 1];
  captured.data[i + 2] = 255 - captured.data[i + 2];
}

canvas.addEventListener("tick", (event) => {
  const time = event.timestamp * 0.001;

  ctx.fillStyle = "#111827";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = "#1d4ed8";
  ctx.fillRect(70, 64, 260, 260);
  ctx.putImageData(source, 110, 104);

  ctx.putImageData(captured, 350, 104);

  const dirtyX = Math.floor((Math.sin(time * 2) * 0.5 + 0.5) * 112);
  const dirtyY = Math.floor((Math.cos(time * 1.5) * 0.5 + 0.5) * 112);
  ctx.fillStyle = "#0f172a";
  ctx.fillRect(590, 64, 160, 260);
  ctx.putImageData(scratch, 580, 94, dirtyX, dirtyY, 68, 68);

  ctx.strokeStyle = "#f8fafc";
  ctx.lineWidth = 4;
  ctx.strokeRect(108, 102, 184, 184);
  ctx.strokeRect(348, 102, 184, 184);
  ctx.strokeRect(578 + dirtyX, 92 + dirtyY, 72, 72);
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
