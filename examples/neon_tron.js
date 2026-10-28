const canvas = globalThis.canvas;

const ctx = canvas.getContext("2d");
const keys = new Set();

const cell = 12;
const cols = Math.floor(canvas.width / cell);
const rows = Math.floor(canvas.height / cell);
const dirs = {
  up: { x: 0, y: -1 },
  down: { x: 0, y: 1 },
  left: { x: -1, y: 0 },
  right: { x: 1, y: 0 },
};
const turnKeys = {
  ArrowUp: dirs.up,
  w: dirs.up,
  W: dirs.up,
  ArrowDown: dirs.down,
  s: dirs.down,
  S: dirs.down,
  ArrowLeft: dirs.left,
  a: dirs.left,
  A: dirs.left,
  ArrowRight: dirs.right,
  d: dirs.right,
  D: dirs.right,
};

let occupied;
let player;
let rival;
let sparks;
let motes;
let pulses;
let stepTimer;
let lastTime;
let running;
let shake;

function keyOf(x, y) {
  return `${x},${y}`;
}

function isReverse(a, b) {
  return a.x + b.x === 0 && a.y + b.y === 0;
}

function reset() {
  occupied = new Set();
  sparks = [];
  motes = [];
  pulses = [];
  stepTimer = 0;
  lastTime = 0;
  running = true;
  shake = 0;

  player = {
    x: 14,
    y: Math.floor(rows * 0.5),
    dir: dirs.right,
    nextDir: dirs.right,
    color: "#18f6ff",
    glow: "#8bf9ff",
    trail: [],
  };

  rival = {
    x: cols - 15,
    y: Math.floor(rows * 0.5),
    dir: dirs.left,
    nextDir: dirs.left,
    color: "#ff2bd6",
    glow: "#ffc2f6",
    trail: [],
  };

  seedCycle(player);
  seedCycle(rival);

  for (let i = 0; i < 120; i += 1) {
    motes.push({
      x: Math.random() * canvas.width,
      y: Math.random() * canvas.height,
      vx: (Math.random() - 0.5) * 18,
      vy: -8 - Math.random() * 26,
      size: 0.6 + Math.random() * 2.8,
      pulse: Math.random() * Math.PI * 2,
      color: Math.random() > 0.5 ? "#18f6ff" : "#ff2bd6",
    });
  }
}

function seedCycle(cycle) {
  cycle.trail.length = 0;
  for (let i = 0; i < 4; i += 1) {
    const x = cycle.x - cycle.dir.x * i;
    const y = cycle.y - cycle.dir.y * i;
    cycle.trail.unshift({ x, y });
    occupied.add(keyOf(x, y));
  }
}

function blocked(x, y) {
  return x < 0 || y < 0 || x >= cols || y >= rows || occupied.has(keyOf(x, y));
}

function moveCycle(cycle) {
  if (!isReverse(cycle.dir, cycle.nextDir)) {
    cycle.dir = cycle.nextDir;
  }

  const nx = cycle.x + cycle.dir.x;
  const ny = cycle.y + cycle.dir.y;
  const hit = blocked(nx, ny);

  cycle.x = nx;
  cycle.y = ny;
  cycle.trail.push({ x: nx, y: ny });
  occupied.add(keyOf(nx, ny));
  trailEmbers(cycle);

  return hit;
}

function chooseRivalTurn() {
  const choices = [dirs.up, dirs.down, dirs.left, dirs.right].filter((dir) => {
    if (isReverse(rival.dir, dir)) return false;
    return !blocked(rival.x + dir.x, rival.y + dir.y);
  });

  if (choices.length === 0) return rival.dir;

  const aheadClear = !blocked(rival.x + rival.dir.x, rival.y + rival.dir.y);
  if (aheadClear && Math.random() > 0.22) return rival.dir;

  choices.sort((a, b) => {
    const ax = rival.x + a.x;
    const ay = rival.y + a.y;
    const bx = rival.x + b.x;
    const by = rival.y + b.y;
    const aScore = freeSpace(ax, ay, a) - Math.abs(ax - player.x) * 0.08 - Math.abs(ay - player.y) * 0.08;
    const bScore = freeSpace(bx, by, b) - Math.abs(bx - player.x) * 0.08 - Math.abs(by - player.y) * 0.08;
    return bScore - aScore;
  });

  return choices[0];
}

function freeSpace(x, y, dir) {
  let score = 0;
  for (let i = 0; i < 8; i += 1) {
    x += dir.x;
    y += dir.y;
    if (blocked(x, y)) break;
    score += 1;
  }
  return score;
}

function burst(x, y, color) {
  const px = x * cell + cell * 0.5;
  const py = y * cell + cell * 0.5;
  pulses.push({ x: px, y: py, age: 0, color });
  pulses.push({ x: px, y: py, age: -0.08, color: "#ffffff" });
  shake = 18;
  for (let i = 0; i < 230; i += 1) {
    const angle = Math.random() * Math.PI * 2;
    const speed = 70 + Math.random() * Math.random() * 780;
    const whiteHot = Math.random() > 0.76;
    sparks.push({
      x: px,
      y: py,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed,
      life: 0.25 + Math.random() * 0.95,
      age: 0,
      width: 1 + Math.random() * 5,
      drag: 0.955 + Math.random() * 0.04,
      color: whiteHot ? "#ffffff" : color,
    });
  }
}

function trailEmbers(cycle) {
  const px = cycle.x * cell + cell * 0.5 - cycle.dir.x * cell * 0.5;
  const py = cycle.y * cell + cell * 0.5 - cycle.dir.y * cell * 0.5;
  const sideX = -cycle.dir.y;
  const sideY = cycle.dir.x;

  for (let i = 0; i < 5; i += 1) {
    const side = (Math.random() - 0.5) * cell * 1.1;
    const speed = 35 + Math.random() * 110;
    sparks.push({
      x: px + sideX * side,
      y: py + sideY * side,
      vx: -cycle.dir.x * speed + sideX * (Math.random() - 0.5) * 70,
      vy: -cycle.dir.y * speed + sideY * (Math.random() - 0.5) * 70,
      life: 0.18 + Math.random() * 0.28,
      age: 0,
      width: 0.8 + Math.random() * 2.2,
      drag: 0.94,
      color: Math.random() > 0.82 ? cycle.glow : cycle.color,
    });
  }
}

function step() {
  rival.nextDir = chooseRivalTurn();

  const playerHit = moveCycle(player);
  const rivalHit = moveCycle(rival);
  if (playerHit || rivalHit || player.x === rival.x && player.y === rival.y) {
    running = false;
    if (playerHit || player.x === rival.x && player.y === rival.y) burst(player.x, player.y, player.color);
    if (rivalHit || player.x === rival.x && player.y === rival.y) burst(rival.x, rival.y, rival.color);
  }
}

function updateSparks(dt) {
  for (const spark of sparks) {
    spark.age += dt;
    spark.x += spark.vx * dt;
    spark.y += spark.vy * dt;
    spark.vx *= spark.drag;
    spark.vy *= spark.drag;
  }
  sparks = sparks.filter((spark) => spark.age < spark.life);

  for (const mote of motes) {
    mote.x += mote.vx * dt;
    mote.y += mote.vy * dt;
    mote.pulse += dt * 3;
    if (mote.y < -12) mote.y = canvas.height + 12;
    if (mote.x < -12) mote.x = canvas.width + 12;
    if (mote.x > canvas.width + 12) mote.x = -12;
  }

  for (const pulse of pulses) {
    pulse.age += dt;
  }
  pulses = pulses.filter((pulse) => pulse.age < 0.8);

  shake = Math.max(0, shake - dt * 42);
}

function update(event) {
  const time = event.timestamp;
  const dt = lastTime === 0 ? 0 : Math.min((time - lastTime) / 1000, 0.05);
  lastTime = time;

  if (keys.has("Enter") || keys.has(" ")) {
    reset();
    keys.delete("Enter");
    keys.delete(" ");
  }

  for (const key of keys) {
    const next = turnKeys[key];
    if (next && !isReverse(player.dir, next)) {
      player.nextDir = next;
      break;
    }
  }

  if (running) {
    stepTimer += dt;
    const interval = Math.max(0.038, 0.072 - player.trail.length * 0.00006);
    while (stepTimer >= interval && running) {
      stepTimer -= interval;
      step();
    }
  }

  updateSparks(dt);
  draw(time / 1000);
}

function draw(time) {
  ctx.save();
  if (shake > 0) {
    ctx.translate((Math.random() - 0.5) * shake, (Math.random() - 0.5) * shake);
  }

  drawBackground(time);

  ctx.globalCompositeOperation = "lighter";
  drawMotes(time);
  drawTrail(rival, time);
  drawTrail(player, time);
  drawCycle(rival, time);
  drawCycle(player, time);
  drawSparks();
  drawPulses();
  ctx.globalCompositeOperation = "source-over";

  if (!running && sparks.length === 0 && pulses.length === 0) {
    drawSilentRestartGlyph(time);
  }

  ctx.restore();
}

function drawBackground(time) {
  const bg = ctx.createLinearGradient(0, 0, canvas.width, canvas.height);
  bg.addColorStop(0, "#030510");
  bg.addColorStop(0.55, "#08051a");
  bg.addColorStop(1, "#010207");
  ctx.fillStyle = bg;
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.globalAlpha = 0.75;
  ctx.strokeStyle = "#10264a";
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let x = 0; x <= canvas.width; x += cell) {
    const drift = Math.sin(time * 1.4 + x * 0.025) * 1.5;
    ctx.moveTo(x + drift, 0);
    ctx.lineTo(x - drift, canvas.height);
  }
  for (let y = 0; y <= canvas.height; y += cell) {
    const drift = Math.cos(time * 1.2 + y * 0.025) * 1.5;
    ctx.moveTo(0, y + drift);
    ctx.lineTo(canvas.width, y - drift);
  }
  ctx.stroke();

  ctx.globalCompositeOperation = "lighter";
  ctx.globalAlpha = 0.08;
  for (let y = 3 + time * 26 % 18; y < canvas.height; y += 18) {
    ctx.fillStyle = "#18f6ff";
    ctx.fillRect(0, y, canvas.width, 1);
  }

  ctx.globalAlpha = 0.12;
  for (let i = 0; i < 16; i += 1) {
    const x = (time * 80 + i * 113) % (canvas.width + 80) - 40;
    const y = 36 + (i * 41 + Math.sin(time + i) * 80) % (canvas.height - 72);
    const streak = ctx.createLinearGradient(x - 40, y, x + 70, y);
    streak.addColorStop(0, "#18f6ff00");
    streak.addColorStop(0.5, i % 2 ? "#ff2bd6" : "#18f6ff");
    streak.addColorStop(1, "#18f6ff00");
    ctx.fillStyle = streak;
    ctx.fillRect(x - 40, y, 110, 2);
  }

  ctx.globalAlpha = 0.22 + Math.sin(time * 2) * 0.06;
  const horizon = ctx.createLinearGradient(0, 0, canvas.width, 0);
  horizon.addColorStop(0, "#18f6ff");
  horizon.addColorStop(0.5, "#7c3cff");
  horizon.addColorStop(1, "#ff2bd6");
  ctx.strokeStyle = horizon;
  ctx.lineWidth = 3;
  ctx.strokeRect(cell * 0.5, cell * 0.5, canvas.width - cell, canvas.height - cell);
  ctx.globalCompositeOperation = "source-over";
  ctx.globalAlpha = 1;
}

function drawMotes(time) {
  for (const mote of motes) {
    const glow = 0.35 + Math.sin(mote.pulse + time * 2) * 0.25;
    ctx.globalAlpha = glow;
    ctx.fillStyle = mote.color;
    ctx.beginPath();
    ctx.arc(mote.x, mote.y, mote.size, 0, Math.PI * 2);
    ctx.fill();

    ctx.globalAlpha = glow * 0.12;
    ctx.beginPath();
    ctx.arc(mote.x, mote.y, mote.size * 5, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.globalAlpha = 1;
}

function drawTrail(cycle, time) {
  if (cycle.trail.length < 2) return;

  const pulse = 0.75 + Math.sin(time * 8 + cycle.trail.length * 0.04) * 0.25;
  for (const width of [26, 17, 10, 4]) {
    ctx.globalAlpha = width === 26 ? 0.06 : width === 17 ? 0.13 : width === 10 ? 0.28 : 0.98;
    ctx.strokeStyle = width === 4 ? cycle.color : cycle.glow;
    ctx.lineWidth = width;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.beginPath();
    for (let i = 0; i < cycle.trail.length; i += 1) {
      const point = cycle.trail[i];
      const x = point.x * cell + cell * 0.5;
      const y = point.y * cell + cell * 0.5;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();
  }

  const head = cycle.trail[cycle.trail.length - 1];
  ctx.globalAlpha = 0.18 * pulse;
  ctx.fillStyle = cycle.glow;
  ctx.beginPath();
  ctx.arc(head.x * cell + cell * 0.5, head.y * cell + cell * 0.5, cell * 2.1, 0, Math.PI * 2);
  ctx.fill();

  ctx.globalAlpha = 0.24 * pulse;
  ctx.strokeStyle = cycle.color;
  ctx.lineWidth = 1.5;
  for (let i = Math.max(0, cycle.trail.length - 22); i < cycle.trail.length; i += 3) {
    const point = cycle.trail[i];
    const r = 2 + (i % 5);
    ctx.beginPath();
    ctx.arc(point.x * cell + cell * 0.5, point.y * cell + cell * 0.5, r, 0, Math.PI * 2);
    ctx.stroke();
  }
  ctx.globalAlpha = 1;
}

function drawCycle(cycle, time) {
  const cx = cycle.x * cell + cell * 0.5;
  const cy = cycle.y * cell + cell * 0.5;
  const angle = Math.atan2(cycle.dir.y, cycle.dir.x);

  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(angle);

  ctx.globalAlpha = 0.28;
  ctx.fillStyle = cycle.glow;
  ctx.beginPath();
  ctx.ellipse(-cell * 0.15, 0, cell * 1.8, cell * 1.1, 0, 0, Math.PI * 2);
  ctx.fill();

  ctx.globalAlpha = 0.16;
  ctx.fillStyle = cycle.color;
  ctx.beginPath();
  ctx.ellipse(-cell * 0.8, 0, cell * 2.8, cell * 0.75, 0, 0, Math.PI * 2);
  ctx.fill();

  ctx.globalAlpha = 1;
  ctx.fillStyle = cycle.color;
  ctx.beginPath();
  ctx.moveTo(cell * 0.9, 0);
  ctx.lineTo(-cell * 0.65, -cell * 0.55);
  ctx.lineTo(-cell * 0.35, 0);
  ctx.lineTo(-cell * 0.65, cell * 0.55);
  ctx.closePath();
  ctx.fill();

  ctx.globalAlpha = 0.6;
  ctx.strokeStyle = "#ffffff";
  ctx.lineWidth = 1.2;
  ctx.beginPath();
  ctx.moveTo(cell * 0.45, -cell * 0.16);
  ctx.lineTo(-cell * 0.35, -cell * 0.36);
  ctx.moveTo(cell * 0.45, cell * 0.16);
  ctx.lineTo(-cell * 0.35, cell * 0.36);
  ctx.stroke();

  ctx.globalAlpha = 0.7 + Math.sin(time * 18) * 0.2;
  ctx.fillStyle = "#ffffff";
  ctx.beginPath();
  ctx.arc(cell * 0.45, 0, 1.8, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
  ctx.globalAlpha = 1;
}

function drawSparks() {
  for (const spark of sparks) {
    const t = 1 - spark.age / spark.life;
    ctx.globalAlpha = Math.max(0, t);
    ctx.strokeStyle = spark.color;
    ctx.lineWidth = spark.width + t * 4;
    ctx.beginPath();
    ctx.moveTo(spark.x, spark.y);
    ctx.lineTo(spark.x - spark.vx * 0.035, spark.y - spark.vy * 0.035);
    ctx.stroke();

    if (t > 0.35) {
      ctx.globalAlpha = t * 0.14;
      ctx.fillStyle = spark.color;
      ctx.beginPath();
      ctx.arc(spark.x, spark.y, spark.width * 4.5, 0, Math.PI * 2);
      ctx.fill();
    }
  }
  ctx.globalAlpha = 1;
}

function drawPulses() {
  for (const pulse of pulses) {
    if (pulse.age < 0) continue;
    const t = pulse.age / 0.8;
    ctx.globalAlpha = Math.max(0, 1 - t);
    ctx.strokeStyle = pulse.color;
    ctx.lineWidth = 8 * (1 - t);
    ctx.beginPath();
    ctx.arc(pulse.x, pulse.y, 8 + t * 150, 0, Math.PI * 2);
    ctx.stroke();

    ctx.globalAlpha = Math.max(0, 0.25 - t * 0.25);
    ctx.fillStyle = pulse.color;
    ctx.beginPath();
    ctx.arc(pulse.x, pulse.y, 20 + t * 80, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.globalAlpha = 1;
}

function drawSilentRestartGlyph(time) {
  const cx = canvas.width * 0.5;
  const cy = canvas.height * 0.5;
  const radius = 28 + Math.sin(time * 4) * 4;

  ctx.globalCompositeOperation = "lighter";
  ctx.globalAlpha = 0.7;
  ctx.strokeStyle = "#18f6ff";
  ctx.lineWidth = 5;
  ctx.beginPath();
  ctx.arc(cx, cy, radius, -Math.PI * 0.25, Math.PI * 1.35);
  ctx.stroke();

  ctx.fillStyle = "#ff2bd6";
  ctx.beginPath();
  ctx.moveTo(cx + radius * 0.84, cy - radius * 0.54);
  ctx.lineTo(cx + radius * 1.25, cy - radius * 0.44);
  ctx.lineTo(cx + radius * 0.98, cy - radius * 0.1);
  ctx.closePath();
  ctx.fill();
  ctx.globalAlpha = 1;
  ctx.globalCompositeOperation = "source-over";
}

globalThis.addEventListener("keydown", (event) => {
  keys.add(event.key);
});

globalThis.addEventListener("keyup", (event) => {
  keys.delete(event.key);
});

globalThis.addEventListener("blur", () => {
  keys.clear();
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});

reset();
canvas.addEventListener("tick", update);
