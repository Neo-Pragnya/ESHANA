// Renders a chart wheel as an SVG string from a real ESHANA result envelope (build time, trusted data).
// Zodiac runs counter-clockwise; the Ascendant sits on the left when the result has one.

/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any;

const TXT = "︎"; // ask for text presentation, not emoji
export const SIGN_GLYPHS = ["♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓"].map((g) => g + TXT);
export const BODY_GLYPHS: Record<string, string> = {
  sun: "☉", moon: "☽", mercury: "☿", venus: "♀", mars: "♂", jupiter: "♃", saturn: "♄", uranus: "♅",
  neptune: "♆", pluto: "♇", mean_node: "☊", true_node: "☊", south_node: "☋",
};
const ASPECT_COLOR: Record<string, string> = {
  conjunction: "#f5c451", sextile: "#2bb3c0", trine: "#2bb3c0", square: "#e0589c", opposition: "#e0589c",
};

const C = 200;
const pt = (r: number, phi: number): [number, number] => [C + r * Math.cos(phi), C - r * Math.sin(phi)];
const f = (n: number) => n.toFixed(2);

export function wheelSvg(env: Any, label: string): string {
  const asc: number | null = env.houses?.ascendant?.longitude_deg ?? null;
  const base = asc ?? 0;
  // screen angle for a longitude: ASC (or 0 degrees) at 180 degrees, increasing counter-clockwise
  const phi = (lon: number) => ((180 + (lon - base)) * Math.PI) / 180;
  const out: string[] = [];
  out.push(`<svg class="wheel" viewBox="0 0 400 400" role="img" aria-label="${label}">`);
  out.push(`<defs><radialGradient id="g-${label.length}" cx="50%" cy="50%" r="50%"><stop offset="0" stop-color="#2a2466" stop-opacity=".55"/><stop offset="1" stop-color="#090b1a" stop-opacity="0"/></radialGradient></defs>`);
  out.push(`<circle cx="${C}" cy="${C}" r="196" fill="url(#g-${label.length})"/>`);
  // slow decorative orbits
  out.push(`<g class="spin" fill="none" stroke-width="1"><ellipse cx="${C}" cy="${C}" rx="196" ry="70" stroke="#8f7bff" stroke-opacity=".28" transform="rotate(-24 ${C} ${C})"/><ellipse cx="${C}" cy="${C}" rx="196" ry="70" stroke="#2bb3c0" stroke-opacity=".25" transform="rotate(36 ${C} ${C})"/></g>`);
  // sign band
  for (let i = 0; i < 12; i++) {
    const a0 = phi(i * 30), a1 = phi((i + 1) * 30);
    const [x0, y0] = pt(188, a0), [x1, y1] = pt(188, a1), [x2, y2] = pt(156, a1), [x3, y3] = pt(156, a0);
    out.push(`<path d="M${f(x0)},${f(y0)} A188,188 0 0 0 ${f(x1)},${f(y1)} L${f(x2)},${f(y2)} A156,156 0 0 1 ${f(x3)},${f(y3)} Z" fill="${i % 2 ? "#161a3d" : "#1c2150"}" stroke="#3a3f7a" stroke-width=".8"/>`);
    const [gx, gy] = pt(172, phi(i * 30 + 15));
    out.push(`<text x="${f(gx)}" y="${f(gy)}" fill="#cfc9ff" font-size="15" text-anchor="middle" dominant-baseline="central"><title>${i * 30}°</title>${SIGN_GLYPHS[i]}</text>`);
  }
  for (let d = 0; d < 360; d += 5) {
    const [x0, y0] = pt(156, phi(d)), [x1, y1] = pt(d % 10 === 0 ? 149 : 152, phi(d));
    out.push(`<line x1="${f(x0)}" y1="${f(y0)}" x2="${f(x1)}" y2="${f(y1)}" stroke="#5a5fa8" stroke-width=".6"/>`);
  }
  out.push(`<circle cx="${C}" cy="${C}" r="150" fill="none" stroke="#3a3f7a" stroke-width=".8"/><circle cx="${C}" cy="${C}" r="78" fill="#0b0e24" stroke="#3a3f7a" stroke-width=".8"/>`);
  // house cusps
  const cusps: number[] | null = env.houses?.cusps_deg ?? null;
  if (cusps) {
    cusps.forEach((c: number, i: number) => {
      const [x0, y0] = pt(78, phi(c)), [x1, y1] = pt(156, phi(c));
      out.push(`<line x1="${f(x0)}" y1="${f(y0)}" x2="${f(x1)}" y2="${f(y1)}" stroke="${i % 3 === 0 ? "#f5c451" : "#4b5099"}" stroke-width="${i % 3 === 0 ? 1.3 : 0.7}"/>`);
      const mid = c + (((cusps[(i + 1) % 12] - c + 360) % 360) / 2);
      const [tx, ty] = pt(90, phi(mid));
      out.push(`<text x="${f(tx)}" y="${f(ty)}" fill="#7f84c9" font-size="9" text-anchor="middle" dominant-baseline="central">${i + 1}</text>`);
    });
  }
  // aspects inside the hub
  const pos: Record<string, number> = {};
  for (const p of env.positions as Any[]) pos[p.id] = p.longitude_deg;
  for (const a of (env.derived_attributes as Any[]).filter((x) => String(x.id).startsWith("western.aspect."))) {
    const [b1, b2] = a.derived_from as string[];
    if (pos[b1] == null || pos[b2] == null) continue;
    const [x0, y0] = pt(76, phi(pos[b1])), [x1, y1] = pt(76, phi(pos[b2]));
    out.push(`<line x1="${f(x0)}" y1="${f(y0)}" x2="${f(x1)}" y2="${f(y1)}" stroke="${ASPECT_COLOR[a.value.aspect] ?? "#888"}" stroke-width="1" stroke-opacity=".75"><title>${b1} ${a.value.aspect} ${b2}, orb ${a.value.orb_deg.toFixed(2)}°</title></line>`);
  }
  // bodies, spread into tiers where they crowd
  const bodies = (env.positions as Any[]).slice().sort((a, b) => a.longitude_deg - b.longitude_deg);
  const placed: { lon: number; tier: number }[] = [];
  for (const p of bodies) {
    const near = placed.filter((q) => Math.min(Math.abs(q.lon - p.longitude_deg), 360 - Math.abs(q.lon - p.longitude_deg)) < 9);
    const used = new Set(near.map((q) => q.tier));
    let tier = 0;
    while (used.has(tier)) tier++;
    placed.push({ lon: p.longitude_deg, tier });
    const r = 134 - tier * 17;
    const [x, y] = pt(r, phi(p.longitude_deg));
    const [tx, ty] = pt(149, phi(p.longitude_deg)), [tx2, ty2] = pt(r + 9, phi(p.longitude_deg));
    out.push(`<line x1="${f(tx)}" y1="${f(ty)}" x2="${f(tx2)}" y2="${f(ty2)}" stroke="#f5c451" stroke-opacity=".5" stroke-width=".7"/>`);
    const g = (BODY_GLYPHS[p.id] ?? "•") + TXT;
    const col = p.id === "sun" ? "#ffe08a" : p.id === "moon" ? "#e6ecff" : p.id.includes("node") ? "#9fb0ff" : "#f2eeff";
    out.push(`<text x="${f(x)}" y="${f(y)}" fill="${col}" font-size="15" text-anchor="middle" dominant-baseline="central"><title>${p.id} ${p.sign.dms} ${p.sign.name}${p.retrograde ? " (retrograde)" : ""}</title>${g}${p.retrograde ? '<tspan font-size="8" dy="5">℞</tspan>' : ""}</text>`);
  }
  if (asc != null) {
    const [ax, ay] = pt(196, phi(asc));
    out.push(`<text x="${f(ax - 2)}" y="${f(ay)}" fill="#f5c451" font-size="10" font-weight="700" text-anchor="end" dominant-baseline="central">AS</text>`);
    const mc: number | undefined = env.houses?.midheaven?.longitude_deg;
    if (mc != null) {
      const [mx, my] = pt(196, phi(mc));
      out.push(`<text x="${f(mx)}" y="${f(my + (my < 200 ? -3 : 9))}" fill="#f5c451" font-size="10" font-weight="700" text-anchor="middle">MC</text>`);
    }
  }
  out.push(`<circle cx="${C}" cy="${C}" r="3.2" fill="#f5c451"/></svg>`);
  return out.join("");
}
