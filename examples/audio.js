const canvas = globalThis.canvas;
const ctx = canvas.getContext("2d");
const audio = new AudioContext();

let started = false;
let pulse = 0;

function blip() {
  audio.resume();

  const now = audio.currentTime;
  const osc = audio.createOscillator();
  const gain = audio.createGain();

  osc.type = "triangle";
  osc.frequency.setValueAtTime(220, now);
  osc.frequency.exponentialRampToValueAtTime(880, now + 0.08);
  gain.gain.setValueAtTime(0.25, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.18);

  osc.connect(gain);
  gain.connect(audio.destination);
  osc.start(now);
  osc.stop(now + 0.2);

  pulse = 1;
}

canvas.addEventListener("pointerdown", () => {
  started = true;
  blip();
});

globalThis.addEventListener("keydown", (event) => {
  if (event.code === "Space") {
    started = true;
    blip();
  }
});

globalThis.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    globalThis.exit();
  }
});

canvas.addEventListener("tick", (event) => {
  pulse = Math.max(0, pulse - event.deltaTime / 21);

  ctx.fillStyle = "#101820";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = pulse > 0 ? "#f25f5c" : "#2ec4b6";
  ctx.fillRect(96, 96, 120 + pulse * 260, 96);

  ctx.fillStyle = started ? "#f6f7eb" : "#ffca3a";
  ctx.fillRect(96, 244, started ? 420 : 280, 18);
});
