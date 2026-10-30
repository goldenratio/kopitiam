const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");
const bunnyImage = new Image();
bunnyImage.src = "examples/tile_0007.png";

const BUNNY_COUNT = 2000;
const BUNNY_SIZE = 24;
const GRAVITY = 900;
const MIN_SPEED = 80;
const MAX_SPEED = 360;
const bunnies = [];

let seed = 0x12345678;
function random() {
  seed = (1664525 * seed + 1013904223) >>> 0;
  return seed / 0x100000000;
}

for (let i = 0; i < BUNNY_COUNT; i++) {
  bunnies.push({
    x: random() * (canvas.width - BUNNY_SIZE),
    y: random() * (canvas.height - BUNNY_SIZE),
    vx: (random() < 0.5 ? -1 : 1) * (MIN_SPEED + random() * (MAX_SPEED - MIN_SPEED)),
    vy: -MAX_SPEED + random() * MAX_SPEED * 2,
  });
}

let lastTime = 0;
let fpsStart = 0;
let fpsFrames = 0;

function frame(event) {
  const time = event.lastTime;
  const dt = lastTime === 0 ? 0 : Math.min((time - lastTime) / 1000, 1 / 30);
  lastTime = time;

  ctx.fillStyle = "#101820";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  for (const bunny of bunnies) {
    bunny.x += bunny.vx * dt;
    bunny.y += bunny.vy * dt;
    bunny.vy += GRAVITY * dt;

    if (bunny.x <= 0) {
      bunny.x = 0;
      bunny.vx = Math.abs(bunny.vx);
    } else if (bunny.x + BUNNY_SIZE >= canvas.width) {
      bunny.x = canvas.width - BUNNY_SIZE;
      bunny.vx = -Math.abs(bunny.vx);
    }

    if (bunny.y <= 0) {
      bunny.y = 0;
      bunny.vy = Math.abs(bunny.vy);
    } else if (bunny.y + BUNNY_SIZE >= canvas.height) {
      bunny.y = canvas.height - BUNNY_SIZE;
      bunny.vy = -Math.abs(bunny.vy) * (0.75 + random() * 0.2);
    }

    ctx.drawImage(bunnyImage, bunny.x, bunny.y, BUNNY_SIZE, BUNNY_SIZE);
  }

  fpsFrames++;
  if (fpsStart === 0) {
    fpsStart = time;
  } else if (time - fpsStart >= 1000) {
    const fps = (fpsFrames * 1000) / (time - fpsStart);
    console.log(`bunnies=${BUNNY_COUNT} fps=${fps.toFixed(1)} commands=${BUNNY_COUNT + 1}`);
    fpsStart = time;
    fpsFrames = 0;
  }
}

canvas.addEventListener("tick", frame);

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});
