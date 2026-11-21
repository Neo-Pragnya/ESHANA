// ESHANA universe: a canvas sky with depth. Three parallax star layers that twinkle, drifting nebulae, and (hero mode)
// the ecliptic as a ring of the 27 nakshatras: a plexus of stars linked to their neighbours that the cursor pulls on
// and springs back, with the nine grahas riding the band at their own speeds and lighting the nakshatra they pass.
// Shooting stars cross now and then. Paused off screen and on hidden tabs; one still frame under reduced motion.

type Mode = "hero" | "ambient";
interface Star { x: number; y: number; z: number; r: number; tw: number; ph: number }
interface Node { a: number; x: number; y: number; vx: number; vy: number; hx: number; hy: number; lit: number; name: string }
interface Rider { a: number; speed: number; color: string; r: number; label: string }
interface Meteor { x: number; y: number; vx: number; vy: number; life: number }

const NAKSHATRAS = [
  "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra", "Punarvasu", "Pushya", "Ashlesha", "Magha",
  "Purva Phalguni", "Uttara Phalguni", "Hasta", "Chitra", "Swati", "Vishakha", "Anuradha", "Jyeshtha", "Mula",
  "Purva Ashadha", "Uttara Ashadha", "Shravana", "Dhanishtha", "Shatabhisha", "Purva Bhadrapada", "Uttara Bhadrapada", "Revati",
];
// relative angular speeds (Moon fastest), colours from the studio's graha palette
const GRAHAS: Rider[] = [
  { label: "Su", speed: 1, color: "245,170,60", r: 4.2, a: 0.3 },
  { label: "Mo", speed: 13.4, color: "225,230,255", r: 3.2, a: 2.1 },
  { label: "Ma", speed: 0.53, color: "232,92,80", r: 2.8, a: 4.0 },
  { label: "Me", speed: 1.4, color: "108,207,140", r: 2.4, a: 0.9 },
  { label: "Ju", speed: 0.083, color: "245,196,81", r: 3.6, a: 5.2 },
  { label: "Ve", speed: 1.2, color: "255,182,222", r: 3, a: 1.4 },
  { label: "Sa", speed: 0.034, color: "143,123,255", r: 3.2, a: 3.3 },
  { label: "Ra", speed: -0.053, color: "120,130,170", r: 2.4, a: 2.6 },
  { label: "Ke", speed: -0.053, color: "120,130,170", r: 2.4, a: 2.6 + Math.PI },
];

export function startUniverse(canvas: HTMLCanvasElement, mode: Mode = "hero", front?: HTMLCanvasElement | null): void {
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  // the near half of the ecliptic (and what rides on it) draws on `front`, a canvas stacked above the content, so the
  // ring passes in front of the wheel card and behind it, like a planetary ring
  const fctx = (front && front.getContext("2d")) || ctx;
  const layer = (depth: number) => (depth > 0 ? fctx : ctx);
  const reduce =
    window.matchMedia("(prefers-reduced-motion: reduce)").matches ||
    new URLSearchParams(location.search).get("motion") === "static";
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  let w = 0, h = 0, visible = true, last = 0, clock = 0;
  const mouse = { x: -1e4, y: -1e4, px: 0, py: 0, inside: false };
  let stars: Star[] = [];
  let nodes: Node[] = [];
  const meteors: Meteor[] = [];
  const riders = GRAHAS.map((g) => ({ ...g }));

  const ecl = () => {
    const wide = w > 900 * dpr;
    const cx = wide ? w * 0.72 : w * 0.5, cy = wide ? h * 0.5 : h * 0.7;
    const rx = wide ? w * 0.33 : w * 0.56;
    return { cx, cy, rx, ry: rx * 0.3, tilt: -0.3 };
  };
  const onEcl = (a: number) => {
    const e = ecl();
    const x = Math.cos(a) * e.rx, y = Math.sin(a) * e.ry;
    return { x: e.cx + x * Math.cos(e.tilt) - y * Math.sin(e.tilt), y: e.cy + x * Math.sin(e.tilt) + y * Math.cos(e.tilt), depth: Math.sin(a) };
  };

  const seed = () => {
    const r = canvas.getBoundingClientRect();
    w = canvas.width = Math.max(1, Math.round(r.width * dpr));
    h = canvas.height = Math.max(1, Math.round(r.height * dpr));
    if (front) { front.width = w; front.height = h; }
    const density = mode === "hero" ? 1 / 5200 : 1 / 9000;
    const n = Math.min(520, Math.round((r.width * r.height) * density));
    stars = Array.from({ length: n }, () => {
      const z = Math.random();
      return { x: Math.random() * w, y: Math.random() * h, z, r: (0.35 + z * 1.25) * dpr, tw: 0.6 + Math.random() * 2.2, ph: Math.random() * 6.28 };
    });
    if (mode === "hero") {
      nodes = NAKSHATRAS.map((name, i) => {
        // each nakshatra sits on the band with a small seeded scatter, so the ring reads as a constellation
        const a = (i / 27) * Math.PI * 2 + Math.sin(i * 7.3) * 0.05;
        const p = onEcl(a);
        const jx = Math.sin(i * 12.9) * 10 * dpr, jy = Math.cos(i * 4.1) * 8 * dpr;
        return { a, name, x: p.x + jx, y: p.y + jy, vx: 0, vy: 0, hx: p.x + jx, hy: p.y + jy, lit: 0 };
      });
    }
  };

  const nebula = (t: number) => {
    const blobs = [
      { x: 0.18 + Math.sin(t * 0.00004) * 0.05, y: 0.85, r: 0.55, c: "143,123,255", a: mode === "hero" ? 0.22 : 0.07 },
      { x: 0.92, y: 0.08 + Math.cos(t * 0.00003) * 0.05, r: 0.42, c: "43,179,192", a: mode === "hero" ? 0.16 : 0.05 },
      { x: 0.62 + Math.cos(t * 0.00005) * 0.04, y: 0.45, r: 0.3, c: "224,88,156", a: mode === "hero" ? 0.08 : 0.03 },
    ];
    for (const b of blobs) {
      const R = b.r * Math.max(w, h);
      const g = ctx.createRadialGradient(b.x * w, b.y * h, 0, b.x * w, b.y * h, R);
      g.addColorStop(0, `rgba(${b.c},${b.a})`);
      g.addColorStop(1, `rgba(${b.c},0)`);
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, w, h);
    }
  };

  const drawStars = (t: number) => {
    const ox = mouse.inside ? (mouse.px / w - 0.5) : 0, oy = mouse.inside ? (mouse.py / h - 0.5) : 0;
    for (const s of stars) {
      const par = (s.z * s.z) * 18 * dpr;
      let x = (s.x - ox * par + t * 0.004 * s.z * dpr) % w;
      if (x < 0) x += w;
      const y = s.y - oy * par;
      const a = 0.35 + 0.65 * (0.5 + 0.5 * Math.sin(t * 0.001 * s.tw + s.ph)) * (0.4 + s.z * 0.6);
      ctx.fillStyle = s.z > 0.92 ? `rgba(255,236,190,${a})` : `rgba(226,222,255,${a})`;
      ctx.beginPath(); ctx.arc(x, y, s.r, 0, 6.2832); ctx.fill();
      if (s.z > 0.96) {  // a few bright stars get a cross flare
        ctx.strokeStyle = `rgba(255,236,190,${a * 0.35})`; ctx.lineWidth = 0.7 * dpr;
        ctx.beginPath(); ctx.moveTo(x - s.r * 5, y); ctx.lineTo(x + s.r * 5, y); ctx.moveTo(x, y - s.r * 5); ctx.lineTo(x, y + s.r * 5); ctx.stroke();
      }
    }
  };

  const drawEcliptic = (t: number, dt: number) => {
    const e = ecl();
    // the band: far half behind the content, near half in front
    for (const [c, from, to] of [[ctx, Math.PI, 2 * Math.PI], [fctx, 0, Math.PI]] as const) {
      c.save(); c.translate(e.cx, e.cy); c.rotate(e.tilt);
      c.strokeStyle = "rgba(143,123,255,.16)"; c.lineWidth = 14 * dpr;
      c.beginPath(); c.ellipse(0, 0, e.rx, e.ry, 0, from, to); c.stroke();
      c.strokeStyle = "rgba(245,196,81,.3)"; c.lineWidth = 1 * dpr; c.setLineDash([2 * dpr, 6 * dpr]);
      c.beginPath(); c.ellipse(0, 0, e.rx, e.ry, 0, from, to); c.stroke(); c.setLineDash([]);
      c.restore();
    }
    // twelve rāśi ticks
    for (let k = 0; k < 12; k++) {
      const a = (k / 12) * 6.2832, co = Math.cos(a), si = Math.sin(a), c = layer(si);
      c.save(); c.translate(e.cx, e.cy); c.rotate(e.tilt);
      c.strokeStyle = "rgba(245,196,81,.35)"; c.lineWidth = 1 * dpr;
      c.beginPath(); c.moveTo(co * e.rx * 0.95, si * e.ry * 0.95); c.lineTo(co * e.rx * 1.05, si * e.ry * 1.05); c.stroke();
      c.restore();
    }

    // physics: cursor repels, springs pull home, damping
    const R = 120 * dpr;
    for (const n of nodes) {
      const dx = n.x - mouse.x, dy = n.y - mouse.y, d = Math.hypot(dx, dy);
      if (d < R && d > 0.1) { const f = (1 - d / R) * 0.9 * dpr; n.vx += (dx / d) * f; n.vy += (dy / d) * f; }
      n.vx += (n.hx - n.x) * 0.02; n.vy += (n.hy - n.y) * 0.02;
      n.vx *= 0.9; n.vy *= 0.9; n.x += n.vx * dt; n.y += n.vy * dt;
      n.lit = Math.max(0, n.lit - 0.012 * dt);
    }
    // plexus links between near neighbours (and across the ring when the cursor brings them close)
    const L = 150 * dpr;
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i], b = nodes[j];
        const d = Math.hypot(a.x - b.x, a.y - b.y);
        const ring = j - i === 1 || (i === 0 && j === 26);
        if (!ring && d > L) continue;
        const near = Math.max(0, 1 - Math.hypot((a.x + b.x) / 2 - mouse.x, (a.y + b.y) / 2 - mouse.y) / (200 * dpr));
        const alpha = ring ? 0.22 + near * 0.5 + Math.max(a.lit, b.lit) * 0.4 : (1 - d / L) * 0.18 + near * 0.3;
        const c = layer(Math.sin(a.a) + Math.sin(b.a));
        c.strokeStyle = `rgba(183,169,255,${alpha})`; c.lineWidth = (ring ? 1 : 0.6) * dpr;
        c.beginPath(); c.moveTo(a.x, a.y); c.lineTo(b.x, b.y); c.stroke();
      }
    }
    // nakshatra stars, the one under the cursor named
    let hover: Node | null = null, best = 26 * dpr;
    for (const n of nodes) {
      const d = Math.hypot(n.x - mouse.x, n.y - mouse.y);
      if (d < best) { best = d; hover = n; }
      const near = Math.sin(n.a) > 0, c = layer(Math.sin(n.a));
      const r = (1.8 + n.lit * 2.4) * dpr * (near ? 1.15 : 0.85);
      const g = c.createRadialGradient(n.x, n.y, 0, n.x, n.y, r * 4);
      g.addColorStop(0, `rgba(255,240,200,${0.8 + n.lit * 0.2})`); g.addColorStop(1, "rgba(255,240,200,0)");
      c.fillStyle = g; c.beginPath(); c.arc(n.x, n.y, r * 4, 0, 6.2832); c.fill();
      c.fillStyle = "#fff6dc"; c.beginPath(); c.arc(n.x, n.y, r * 0.6, 0, 6.2832); c.fill();
    }
    // grahas ride the band; passing a nakshatra lights it
    for (const g of riders) {
      g.a += g.speed * 0.00006 * dt * 16;
      const p = onEcl(g.a);
      const c = layer(p.depth);
      const r = g.r * dpr * (p.depth > 0 ? 1.15 : 0.85);
      const glow = c.createRadialGradient(p.x, p.y, 0, p.x, p.y, r * 4);
      glow.addColorStop(0, `rgba(${g.color},.9)`); glow.addColorStop(1, `rgba(${g.color},0)`);
      c.fillStyle = glow; c.beginPath(); c.arc(p.x, p.y, r * 4, 0, 6.2832); c.fill();
      c.fillStyle = `rgba(${g.color},1)`; c.beginPath(); c.arc(p.x, p.y, r, 0, 6.2832); c.fill();
      const idx = ((Math.floor((((g.a % 6.2832) + 6.2832) % 6.2832) / (6.2832 / 27)) % 27) + 27) % 27;
      nodes[idx].lit = Math.min(1, nodes[idx].lit + 0.05 * dt);
    }
    if (hover) {
      fctx.font = `${12 * dpr}px Inter, system-ui, sans-serif`;
      fctx.fillStyle = "rgba(255,236,190,.95)";
      fctx.fillText(hover.name, hover.x + 10 * dpr, hover.y - 10 * dpr);
    }
    // a slow drift of the band's home positions keeps it alive without the cursor
    if (!reduce) for (const n of nodes) {
      n.a += 0.000012 * dt * 16;
      const p = onEcl(n.a);
      const i = NAKSHATRAS.indexOf(n.name);
      n.hx = p.x + Math.sin(i * 12.9) * 10 * dpr; n.hy = p.y + Math.cos(i * 4.1) * 8 * dpr;
    }
    void t;
  };

  const drawMeteors = (dt: number) => {
    if (!reduce && Math.random() < (mode === "hero" ? 0.006 : 0.0015) * dt) {
      const fromLeft = Math.random() < 0.5;
      meteors.push({ x: (fromLeft ? 0.1 : 0.5) * w + Math.random() * w * 0.4, y: -10, vx: (fromLeft ? 1 : -1) * (5 + Math.random() * 4) * dpr, vy: (3 + Math.random() * 3) * dpr, life: 1 });
    }
    for (let i = meteors.length - 1; i >= 0; i--) {
      const m = meteors[i];
      m.x += m.vx * dt; m.y += m.vy * dt; m.life -= 0.012 * dt;
      if (m.life <= 0 || m.y > h + 20) { meteors.splice(i, 1); continue; }
      const g = ctx.createLinearGradient(m.x, m.y, m.x - m.vx * 14, m.y - m.vy * 14);
      g.addColorStop(0, `rgba(255,240,210,${m.life})`); g.addColorStop(1, "rgba(255,240,210,0)");
      ctx.strokeStyle = g; ctx.lineWidth = 1.4 * dpr;
      ctx.beginPath(); ctx.moveTo(m.x, m.y); ctx.lineTo(m.x - m.vx * 14, m.y - m.vy * 14); ctx.stroke();
    }
  };

  const frame = (t: number) => {
    const dt = last ? Math.min(3, (t - last) / 16.67) : 1;
    last = t; clock = t;
    mouse.px += ((mouse.inside ? mouse.x : w / 2) - mouse.px) * 0.06;
    mouse.py += ((mouse.inside ? mouse.y : h / 2) - mouse.py) * 0.06;
    ctx.clearRect(0, 0, w, h);
    if (fctx !== ctx) fctx.clearRect(0, 0, w, h);
    nebula(t);
    drawStars(t);
    if (mode === "hero") drawEcliptic(t, dt);
    drawMeteors(dt);
  };

  seed();
  let resizeTimer = 0;
  window.addEventListener("resize", () => { clearTimeout(resizeTimer); resizeTimer = window.setTimeout(() => { seed(); if (reduce) frame(clock || 1); }, 120); });
  const host = canvas.parentElement ?? canvas;
  const track = mode === "hero" ? host : window;
  track.addEventListener("pointermove", (ev) => {
    const e = ev as PointerEvent, r = canvas.getBoundingClientRect();
    mouse.x = (e.clientX - r.left) * dpr; mouse.y = (e.clientY - r.top) * dpr; mouse.inside = true;
  });
  track.addEventListener("pointerleave", () => { mouse.inside = false; mouse.x = mouse.y = -1e4; });
  canvas.dataset.universe = "running";
  canvas.dataset.nakshatras = String(nodes.length);
  if (reduce) { frame(1); canvas.dataset.universe = "still"; return; }
  new IntersectionObserver(([e]) => (visible = e.isIntersecting)).observe(canvas);
  let frames = 0;
  const tick = (t: number) => {
    if (visible && !document.hidden) { frame(t); if (++frames % 30 === 0) canvas.dataset.frames = String(frames); }
    else last = 0;
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
}
