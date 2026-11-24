/* ESHANA Studio: the Daśā page. Every daśā system the engine computed (vedic.dasha.*) gets the same zoomable,
   labelled timeline: Vimśottarī, Yoginī, Aṣṭottarī, Ṣoḍaśottarī, Chara, Nārāyaṇa, Navāṃśa, Śūla and Kālachakra. */

/* ── a simple one-row timeline (kept for other pages) ─────────────────────────────────────────────────────────── */
function timeline(periods, labelKey, sub, onPick) {
  const W = 1000, H = 54, t0 = T(periods[0].start_utc), t1 = T(periods[periods.length - 1].end_utc), x = t => (t - t0) / (t1 - t0) * W;
  const svg = s("svg", { viewBox: `0 0 ${W} ${H + 22}`, role: "img", "aria-label": "Period timeline" });
  periods.forEach((p, i) => {
    const lord = (p.lord || "").toLowerCase(), col = (G[lord] || G.lagna).dot, a = x(T(p.start_utc)), bb = x(T(p.end_utc));
    const r = s("rect", { x: a, y: 4, width: Math.max(1, bb - a - 1), height: H - 8, rx: 6, fill: col, stroke: "rgba(0,0,0,.25)", class: "tl" }, svg);
    r.style.setProperty("--i", i);
    r.addEventListener("mousemove", ev => tip(ev, `${p[labelKey] || p.lord}`, [["From", dateOnly(p.start_utc)], ["To", dateOnly(p.end_utc)], ["Years", p.years && p.years.toFixed(2)]], sub));
    r.addEventListener("mouseleave", untip); r.addEventListener("click", () => onPick && onPick(p, i));
    if (bb - a > 40) { const t = s("text", { class: "lbl", x: (a + bb) / 2, y: H / 2, "font-size": 13, "font-weight": 700, fill: ink(col) }, svg); t.textContent = (p[labelKey] || p.lord).slice(0, bb - a > 90 ? 12 : 3); }
    if (bb - a > 30) { const yr = s("text", { x: a + 2, y: H + 16, "font-size": 11, fill: "var(--muted)" }, svg); yr.textContent = p.start_utc.slice(0, 4); }
  });
  if (REF_UTC && T(REF_UTC) > t0 && T(REF_UTC) < t1) { const nx = x(T(REF_UTC)); s("line", { x1: nx, x2: nx, y1: 0, y2: H, stroke: "var(--accent)", "stroke-width": 3, class: "now" }, svg); const t = s("text", { x: nx + 4, y: 12, "font-size": 12, "font-weight": 800, fill: "var(--accent)" }, svg); t.textContent = "NOW"; }
  return h("div", { class: "timeline" }, svg);
}

/* ── sub-periods computed in the page (deeper than the engine gave), by each system rule ──────────────────────── */
const VIM_ORDER = ["ketu", "venus", "sun", "moon", "mars", "rahu", "jupiter", "saturn", "mercury"];
const VIM_YEARS = { ketu: 7, venus: 20, sun: 6, moon: 10, mars: 7, rahu: 18, jupiter: 16, saturn: 19, mercury: 17 };
const DASHA_LEVELS = ["Mahādaśā", "Antardaśā", "Pratyantardaśā", "Sūkṣma daśā", "Prāṇa daśā"];
const cap = s0 => s0.charAt(0).toUpperCase() + s0.slice(1);
const VIM_BIRTH = vim => vim.mahadashas[0].start_utc;
/* proportional sub-periods: each lord share is its years out of the cycle, in order from the period own lord,
   clipped at birth */
function propChildren(p, seq, birthIso, yearDays) {
  const lord = (p.lord || "").toLowerCase(), k0 = seq.findIndex(x => x[0] === lord); if (k0 < 0) return [];
  const total = seq.reduce((a, x) => a + x[1], 0), ms = (yearDays || 365.25) * 864e5, end = T(p.end_utc);
  const full = (p.years || (end - T(p.start_utc)) / ms) * ms, start = end - full, birth = T(birthIso), out = [];
  let t = start;
  for (let i = 0; i < seq.length; i++) {
    const [l, y] = seq[(k0 + i) % seq.length], len = full * y / total, a = t, z = t + len; t = z;
    if (z <= birth) continue;
    out.push({ lord: cap(l), start_utc: new Date(Math.max(a, birth)).toISOString(), end_utc: new Date(z).toISOString(), years: len / ms, computed: true, clipped_at_birth: a < birth });
  }
  return out;
}
/* Vimśottarī: from the engine where it gave them (levels 1–3), otherwise by the same rule */
function vimChildren(p, vim) {
  const kids = p.antardashas || p.pratyantardashas || p.sookshmas || p.sub;
  if (kids && kids.length) return kids;
  return propChildren(p, VIM_ORDER.map(l => [l, VIM_YEARS[l]]), VIM_BIRTH(vim), vim.year_days);
}
const ASHTOTTARI_SEQ = [["sun", 6], ["moon", 15], ["mars", 8], ["mercury", 17], ["saturn", 10], ["jupiter", 19], ["rahu", 12], ["venus", 21]];
const SHODASHOTTARI_SEQ = [["sun", 11], ["mars", 12], ["jupiter", 13], ["saturn", 14], ["ketu", 15], ["moon", 16], ["mercury", 17], ["venus", 18]];
const YOGINI_GRAHA = { Mangala: "moon", Pingala: "sun", Dhanya: "jupiter", Bhramari: "mars", Bhadrika: "mercury", Ulka: "saturn", Siddha: "venus", Sankata: "rahu" };
const engineKids = p => p.antardashas || p.pratyantardashas || p.sookshmadashas || [];

/* sign colours by element (fire, earth, air, water), each sign of an element a different shade */
const SIGN_TONE = ["#e0603a", "#7fae4c", "#5fb0d8", "#4a78c8", "#f0923c", "#a7bb4f", "#86c9e3", "#3f9aa8", "#d9474d", "#6b8f3e", "#9db6ea", "#2f6fa0"];

/* ── the systems ──────────────────────────────────────────────────────────────────────────────────────────────── */
const DASHA_SYSTEMS = [
  { id: "vimshottari", tab: "Vimśottarī", depth: 5, kids: (p, v) => vimChildren(p, v), source: "levels 1–3 from the engine, deeper levels subdivided in the page by the Vimśottarī proportions" },
  { id: "yogini", tab: "Yoginī", depth: 3, kids: p => engineKids(p), source: "every level from the engine" },
  { id: "ashtottari", tab: "Aṣṭottarī", depth: 3, kids: (p, v) => engineKids(p).length ? engineKids(p) : propChildren(p, ASHTOTTARI_SEQ, v.mahadashas[0].start_utc, v.year_days), source: "levels 1–2 from the engine, the third subdivided in the page by the Aṣṭottarī proportions (years out of 108)" },
  { id: "shodashottari", tab: "Ṣoḍaśottarī", depth: 3, kids: (p, v) => engineKids(p).length ? engineKids(p) : propChildren(p, SHODASHOTTARI_SEQ, v.mahadashas[0].start_utc, v.year_days), source: "levels 1–2 from the engine, the third subdivided in the page by the Ṣoḍaśottarī proportions (years out of 116)" },
  { id: "chara", tab: "Chara", depth: 2, kids: p => engineKids(p), source: "mahādaśās and antardaśās from the engine" },
  { id: "narayana", tab: "Nārāyaṇa", depth: 2, kids: p => engineKids(p), source: "mahādaśās and antardaśās from the engine" },
  { id: "navamsha", tab: "Navāṃśa", depth: 2, kids: p => engineKids(p), source: "mahādaśās and antardaśās from the engine" },
  { id: "shoola", tab: "Śūla", depth: 2, kids: p => engineKids(p), source: "mahādaśās and antardaśās from the engine" },
  { id: "kalachakra", tab: "Kālachakra", depth: 2, kids: p => engineKids(p), source: "mahādaśās and antardaśās from the engine" },
  /* conditional nakṣatra daśās: each says whether its condition holds for this chart (shown above the timeline) */
  ...[["dvadashottari", "Dvādaśottarī"], ["panchottari", "Pañcottarī"], ["shatabdika", "Śatābdikā"], ["chaturashiti_sama", "Caturaśīti-sama"],
      ["dvisaptati_sama", "Dvisaptati-sama"], ["shashtihayani", "Ṣaṣṭihāyanī"], ["shattrimshat_sama", "Ṣaṭtriṃśat-sama"]]
    .map(([id, tab]) => ({ id, tab, depth: 2, conditional: true, kids: p => engineKids(p),
      source: "mahādaśās and antardaśās from the engine; a conditional daśā: see whether its condition holds for this chart" })),
];
const dashaValue = sys => val("vedic.dasha." + sys.id);
const isSign = p => p && p.sign != null;
const grahaOf = p => { const l = p.lord || ""; return (YOGINI_GRAHA[l] || YOGINI_GRAHA[p.yogini] || l).toLowerCase(); };
const periodColour = p => isSign(p) ? SIGN_TONE[p.sign] : (G[grahaOf(p)] || G.lagna).dot;
const periodName = p => isSign(p) ? `${SIGN_SKT[p.sign]} (${SIGN_EN[p.sign]})` : p.yogini ? `${p.yogini} · ${p.lord}` : p.lord;
const periodShort = (p, wide) => isSign(p) ? (wide ? `${GLYPH[p.sign]} ${SIGN_SKT[p.sign]}` : GLYPH[p.sign]) : wide ? (p.yogini || p.lord) : (p.yogini || p.lord || "").slice(0, 2);
const periodLen = y => y == null ? null : y >= 1 ? `${fix(y, 2)} years` : `${fix(y * 365.25, 1)} days`;
/* a period as a link: a graha to its page, a sign to the bhāva it is from the lagna */
function periodLink(p) {
  if (isSign(p)) { const hs = houseOf(p.sign); return hs ? h("a", { href: `#/house/${hs}`, class: "glink" }, h("span", { class: "dz-glyph", style: `color:${SIGN_TONE[p.sign]}` }, GLYPH[p.sign]), " ", SIGN_SKT[p.sign]) : h("span", {}, GLYPH[p.sign], " ", SIGN_SKT[p.sign]); }
  const g = grahaOf(p); return G[g] ? h("span", {}, glink(g), p.yogini || (YOGINI_GRAHA[p.lord] ? ` (${p.lord})` : "")) : h("span", {}, p.lord);
}

/* ── the zoom: one row per level, each a thinner row opening out of the period chosen above it ────────────────── */
const DASHA_PATHS = {};
function dashaZoom(sys, v) {
  const levels = sys.depth, kidsOf = p => sys.kids(p, v) || [];
  // open on the periods running at the reference date; when that date is outside this system's span (a chart of
  // someone born long ago: the 120-year Vimśottarī ended before today), open on the periods running at birth
  const md = v.mahadashas, spanEnd = md.length ? T(md[md.length - 1].end_utc) : null, now = T(REF_UTC || new Date().toISOString());
  const ended = spanEnd != null && now >= spanEnd, ref = ended ? T(md[0].start_utc) + 1 : now;
  if (!DASHA_PATHS[sys.id]) { // open on the running chain
    const path = []; let list = v.mahadashas;
    for (let lvl = 0; lvl < levels && list.length; lvl++) {
      const i = list.findIndex(p => T(p.start_utc) <= ref && ref < T(p.end_utc)); if (i < 0) break;
      path.push(i); list = kidsOf(list[i]);
    }
    DASHA_PATHS[sys.id] = path;
  }
  const PATH = DASHA_PATHS[sys.id];
  const W = 1000, left = 150, rowH = [30, 24, 19, 15, 12], gap = 34, rows = [];
  let list = v.mahadashas;
  for (let lvl = 0; lvl < levels && list && list.length; lvl++) { rows.push(list); const i = PATH[lvl]; if (i == null || !list[i]) break; list = kidsOf(list[i]); }
  if (rows.length < levels && PATH.length === rows.length && list && list.length) rows.push(list);
  let y = 8; const ys = rows.map((_, k) => { const v0 = y; y += rowH[k] + gap; return v0; });
  const H = y - gap + 26;
  const svg = s("svg", { viewBox: `0 0 ${W} ${H}`, class: "dzoom", role: "img", "aria-label": `${sys.tab} daśā, ${levels} levels` });
  rows.forEach((periods, k) => {
    const t0 = T(periods[0].start_utc), t1 = T(periods[periods.length - 1].end_utc), X = t => left + (t - t0) / (t1 - t0) * (W - left - 4), sel = PATH[k];
    const lab = s("text", { x: left - 10, y: ys[k] + rowH[k] / 2 + 4, class: "dz-lab" }, svg); lab.textContent = DASHA_LEVELS[k];
    const g = s("g", { class: "dz-row", style: `--k:${k}` }, svg);
    periods.forEach((p, i) => {
      const col = periodColour(p), a = X(T(p.start_utc)), z = X(T(p.end_utc));
      const r = s("rect", { x: a, y: ys[k], width: Math.max(.8, z - a - .8), height: rowH[k], rx: Math.min(5, rowH[k] / 3), fill: col, class: "dz-bar" + (isSign(p) ? " dz-sign" : "") + (i === sel ? " sel" : sel != null ? " dim" : "") }, g);
      r.addEventListener("mousemove", ev => tip(ev, `${DASHA_LEVELS[k]}: ${periodName(p)}`, [
        ["Chain", rows.slice(0, k).map((rr, j) => periodName(rr[PATH[j]])).concat([periodName(p)]).join(" › ")],
        ["From", fmtTime(p.start_utc).slice(0, 16)], ["To", fmtTime(p.end_utc).slice(0, 16)], ["Length", periodLen(p.years)],
        isSign(p) && houseOf(p.sign) ? ["Bhāva from lagna", String(houseOf(p.sign))] : null,
        p.cycle ? ["Cycle", String(p.cycle)] : null,
        p.nakshatra ? ["Pada row", `${p.nakshatra} ${p.pada} (${p.chakra}), paramāyus ${p.paramayus}`] : null,
        p.gati_from_previous && p.gati_from_previous !== "regular" ? ["Gati", p.gati_from_previous] : null,
        p.nakshatra && p.deha === p.sign ? ["Deha", "the deha sign of its pada"] : null, p.nakshatra && p.jiva === p.sign ? ["Jīva", "the jīva sign of its pada"] : null,
        ["Source", p.computed ? "subdivided in the page" : "engine"]].filter(Boolean), k + 1 < levels ? "click to open its sub-periods" : "deepest level"));
      r.addEventListener("mouseleave", untip);
      r.addEventListener("click", () => { DASHA_PATHS[sys.id] = PATH.slice(0, k).concat([i]); route(); });
      if (z - a > 18) { const t = s("text", { x: (a + z) / 2, y: ys[k] + rowH[k] / 2 + 1, class: "dz-name", "font-size": Math.max(8.5, rowH[k] * .5), fill: ink(col) }, g); t.textContent = periodShort(p, z - a > 74); }
    });
    if (ref > t0 && ref < t1) { const nx = X(ref); s("line", { x1: nx, x2: nx, y1: ys[k] - 4, y2: ys[k] + rowH[k] + 4, class: "dz-now" }, svg); }
    if (sel != null && rows[k + 1]) { // the zoom bracket: from the chosen period out to the full width of the next row
      const p = periods[sel], a = X(T(p.start_utc)), z = X(T(p.end_utc)), y0 = ys[k] + rowH[k] + 2, y1 = ys[k + 1] - 3;
      s("path", { d: `M${a} ${y0}L${z} ${y0}L${W - 4} ${y1}L${left} ${y1}Z`, class: "dz-bracket", fill: periodColour(p) }, svg);
    }
    const yr = s("text", { x: left, y: ys[k] + rowH[k] + 12, class: "dz-date" }, svg); yr.textContent = dateOnly(periods[0].start_utc);
    const yr2 = s("text", { x: W - 4, y: ys[k] + rowH[k] + 12, class: "dz-date", "text-anchor": "end" }, svg); yr2.textContent = dateOnly(periods[periods.length - 1].end_utc);
  });
  const chain = rows.map((rr, j) => PATH[j] != null && rr[PATH[j]] ? rr[PATH[j]] : null).filter(Boolean);
  return h("div", {}, h("div", { class: "chain dz-chain" }, chain.map((p, j) => h("button", { class: "link", onclick: () => { DASHA_PATHS[sys.id] = PATH.slice(0, j); route(); } },
      h("small", {}, DASHA_LEVELS[j]), h("b", {}, periodLink(p)), h("small", {}, `${dateOnly(p.start_utc)} → ${dateOnly(p.end_utc)}`))),
    h("button", { class: "btn", onclick: () => { DASHA_PATHS[sys.id] = null; route(); } }, ended ? "Back to birth" : "Back to now")),
    ended ? note(`This ${sys.tab} span ended on ${dateOnly(md[md.length - 1].end_utc)}, before the reference date, so nothing is running now; the zoom opens on the periods running at birth.`) : null,
    h("div", { class: "timeline" }, svg),
    note(`Click any bar to open its sub-periods below it; the bracket shows the period being zoomed. Source: ${sys.source}.`));
}

/* ── what each system says about itself ───────────────────────────────────────────────────────────────────────── */
const yesNo = b => b == null ? "unknown" : b ? "yes" : "no";
const rashiOf = i => i == null ? null : `${GLYPH[i]} ${SIGN_SKT[i]}`;
function systemFacts(v) {
  const rows = [];
  if (v.starting_lord) rows.push(["Starts with", v.starting_lord], ["Balance at birth", `${fix(v.balance_at_birth_years, 3)} years`]);
  if (v.starting_yogini) rows.push(["Starts with", v.starting_yogini], ["Balance at birth", `${fix(v.balance_at_birth_years, 3)} years`]);
  if (v.moon_nakshatra) rows.push(["Moon nakṣatra", v.moon_nakshatra + (v.moon_pada ? ` pada ${v.moon_pada}` : "")]);
  if (v.start_sign != null) rows.push(["First sign", rashiOf(v.start_sign)], ["Order", v.direction]);
  if (v.start_from) rows.push(["Chosen from", `${rashiOf(v.start_from.lagna)} (lagna) or ${rashiOf(v.start_from.seventh)} (7th): ${rashiOf(v.start_from.chosen)}${v.start_from.decided_by_rule ? ", by strength rule " + v.start_from.decided_by_rule : ", a tie"}`]);
  if (v.chakra) rows.push(["Chakra", v.chakra], ["Paramāyus of the janma pada", `${v.paramayus} years`], ["Deha · jīva", `${rashiOf(v.deha)} · ${rashiOf(v.jiva)}`], ["Balance at birth", `${fix(v.balance_at_birth_years, 3)} years`]);
  if (v.applicability) rows.push(["Classical condition holds", yesNo(v.applicability.applies)], ["Condition", v.applicability.rule]);
  if (v.year_days) rows.push(["Year", `${v.year_days} days`]);
  return rows;
}
function runningNow(v) {
  if (!v.current) return null;
  return card(`Running now · ${fmtTime(v.current.at_utc)}`, h("div", { class: "chain dz-running" }, v.current.periods.map((p, j) => h("div", { class: "link" },
    h("small", {}, DASHA_LEVELS[j] || p.level), h("b", {}, periodLink(p)), h("small", {}, `${dateOnly(p.start_utc)} → ${dateOnly(p.end_utc)}`)))));
}
function conventionsCard(v) {
  if (!v.conventions) return null;
  const rows = Object.entries(v.conventions).map(([k, x]) => [cap(k.replace(/_/g, " ")), String(x)]);
  return h("details", { class: "card dz-conv" }, h("summary", {}, "Conventions ", label("tradition_rule")), facts(rows),
    v.years_by_sign ? table(["Sign", "Lord used", "Counted", "Years"], v.years_by_sign.map(r => [rashiOf(r.sign), cap(r.lord), r.counted, String(r.years)])) : null);
}

page("dasha", () => "Daśā", "Time", (b, arg) => {
  const avail = DASHA_SYSTEMS.filter(x => dashaValue(x)), ss = val("vedic.sade_sati");
  if (!avail.length) return put(b, missing("Daśā", "It needs an exact birth time."));
  const sys = avail.find(x => x.id === arg) || avail[0], v = dashaValue(sys);
  const out = [pageHead("dasha", "Daśā periods", "Choose a system, then zoom: click a period to open its sub-periods beneath it. The dashed line marks the reference instant recorded in this file.")];
  out.push(h("div", { class: "dz-tabs" }, tabs(avail.map(x => [x.id, x.tab]), sys.id, k => go("#/dasha/" + k))));
  const head = sys.id === "vimshottari" ? `Vimśottarī · balance at birth ${fix(v.balance_at_birth_years, 3)} years of ${v.starting_lord}`
    : sys.id === "yogini" ? `Yoginī · starts with ${v.starting_yogini}, balance ${fix(v.balance_at_birth_years, 3)} years` : `${sys.tab} daśā`;
  out.push(card(head, dashaZoom(sys, v)));
  const run = runningNow(v); if (run) out.push(run);
  out.push(card("About this system", facts(systemFacts(v))));
  const conv = conventionsCard(v); if (conv) out.push(conv);
  if (ss) out.push(card(`Sāḍe Sātī${ss.active_at_reference ? " · active now" : ""}`, table(["From", "To", "Phases"], ss.windows.map(w => [dateOnly(w.start), dateOnly(w.end), w.phases.map(p => `${p.phase} ${dateOnly(p.start)}→${dateOnly(p.end)}`).join(" · ")])), ss.note ? note(ss.note) : null));
  out.push(h("details", { class: "card" }, h("summary", {}, "Every mahādaśā and antardaśā"), table(["Mahādaśā", "Antardaśā", "From", "To"], v.mahadashas.flatMap(m => engineKids(m).map(a => [periodName(m), periodName(a), dateOnly(a.start_utc), dateOnly(a.end_utc)])))));
  put(b, out);
}, { blurb: "Vimśottarī, Yoginī, Aṣṭottarī, Ṣoḍaśottarī, Chara, Nārāyaṇa, Navāṃśa, Śūla, Kālachakra, Sāḍe Sātī" });
