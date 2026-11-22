/* ESHANA Studio: Chakras. Sarvatobhadra (with vedha), Kota and Saṅghaṭṭa, drawn from vedic.chakra.* (the engine gives
   every cell, place, line and placement with its convention; this page only draws, animates and explains). */
SIGIL.chakras = `<rect x="3" y="3" width="18" height="18" rx="1.5" class="ring"/><rect x="7.5" y="7.5" width="9" height="9" class="ring"/><g class="spin"><line x1="3" y1="3" x2="21" y2="21" class="ray"/><line x1="21" y1="3" x2="3" y2="21" class="ray"/></g><circle cx="12" cy="12" r="1.6" class="core"/>`;

let CK_TAB = "sbc", CK_SET = "natal", CK_PICK = null;
const CK_AB = ["Asw", "Bha", "Kri", "Roh", "Mrg", "Ard", "Pun", "Pus", "Asl", "Mag", "PPh", "UPh", "Has", "Cht", "Swa", "Vis", "Anu", "Jye", "Mul", "PSh", "USh", "Abh", "Shr", "Dha", "Sat", "PBh", "UBh", "Rev"];
const CK_LINE = { front: "front (sammukha)", left: "left (vāma)", right: "right (dakṣiṇa)" };
const CK_KIND = { nakshatra: "Nakshatra", vowel: "Vowel (svara)", letter: "Letter (varṇa)", rashi: "Rāśi", tithi: "Tithi group and weekdays" };
const CK_ZONE = { stambha: "Stambha (pillar, the core)", madhya: "Madhya (middle)", prakara: "Prākāra (wall)", bahya: "Bāhya (outside)" };
const ckEn = id => (G[id] || G.lagna).en;
const ckAb = id => (G[id] || G.lagna).abbr;
const ckList = ids => (ids || []).map(ckEn).join(", ");
function ckHover(el, fn) {
  el.addEventListener("mousemove", ev => { const [t, rows, sub] = fn(); tip(ev, t, rows, sub); });
  el.addEventListener("mouseleave", untip);
}
/* a graha marker: natal solid, transit as a ring */
function ckMark(parent, id, x, y, r, k, transit) {
  const meta = G[id] || G.lagna;
  // the position sits on an outer group: the pop-in animation sets `transform` on the marker itself, which would
  // otherwise replace the translate and draw every marker at the origin
  const at = s("g", { transform: `translate(${x.toFixed(1)} ${y.toFixed(1)})` }, parent);
  const g = s("g", { class: "ck-pl" + (transit ? " tr" : ""), "data-graha": id }, at);
  g.style.setProperty("--k", k);
  s("circle", { r, fill: transit ? "var(--panel)" : meta.dot, stroke: meta.dot, "stroke-width": transit ? 2.4 : 1 }, g);
  const t = s("text", { fill: transit ? "var(--ink)" : ink(meta.dot), "font-size": r * 1.05 }, g); t.textContent = meta.abbr;
  if (!transit && GR[id]) g.addEventListener("click", () => go("#/planet/" + id));
  return g;
}
function ckPicker(svg, list, after) {
  const chips = [];
  const pick = id => {
    CK_PICK = id; svg.classList.toggle("picked", id != null);
    chips.forEach(c => c.classList.toggle("on", (c.dataset.graha || null) === id));
    after(id);
  };
  const mk = (id, text) => { const c = h("button", { class: "ck-chip" + (CK_PICK === id ? " on" : ""), "data-graha": id || "", type: "button", onclick: () => pick(id) }, id ? dot(id) : null, text); chips.push(c); return c; };
  const el = h("div", { class: "grp ck-pick" }, h("b", {}, "Vedha of"), mk(null, "all"), list.filter(g => g.id !== "lagna").map(g => mk(g.id, ckAb(g.id) + (g.retrograde ? "ᴿ" : ""))));
  setTimeout(() => pick(list.some(g => g.id === CK_PICK) ? CK_PICK : null), 0);
  return el;
}
const ckHow = (...kids) => card("How to read", ...kids);

/* short names as the published reference chart prints them, the rāśi abbreviations, and helpers shared by the
   Sarvatobhadra and the Kota */
const CK_SN = ["Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashi", "Ardra", "Punarvasu", "Pushya", "Ashlesha", "Magha", "P.Phalg.", "U.Phalg.", "Hasta", "Chitra",
  "Swati", "Vishakha", "Anuradha", "Jyeshtha", "Moola", "P.Shad.", "U.Shad.", "Abhijit", "Shravana", "Dhanish.", "Satabhi.", "P.Bhadra.", "U.Bhadra.", "Revati"];
const CK_RA = ["Ari", "Tau", "Gem", "Can", "Leo", "Vir", "Lib", "Sco", "Sag", "Cap", "Aqu", "Pis"];
/* nature class of a graha in a list of engine rows (malefic red, benefic green, the lagna neutral) */
const ckNat = (rows, id) => { if (id === "lagna") return "neu"; const r = (rows || []).find(x => x.id === id); return r && r.nature === "benefic" ? "ben" : "mal"; };
/* graha abbreviations as text ("Sa,Ju"), coloured by nature, `per` to a line; natal ones can open the graha page */
function ckGtext(parent, x, y, ids, natOf, per, o = {}) {
  const out = [];
  for (let i = 0; i < ids.length; i += per) {
    const t = s("text", { x: x.toFixed(1), y: (y + (i / per) * (o.lh || 12)).toFixed(1), class: "ck-gl", "font-size": o.size || 10.5, "data-fit": o.fit || null }, parent);
    ids.slice(i, i + per).forEach((id, k) => {
      if (k) s("tspan", { class: "ck-sep" }, t).textContent = ",";
      const sp = s("tspan", { class: `ck-g ${natOf(id)}` + (o.transit ? " tr" : ""), "data-graha": id }, t);
      sp.textContent = ckAb(id);
      if (o.click && !o.transit && GR[id]) sp.addEventListener("click", ev => { ev.stopPropagation(); go("#/planet/" + id); });
    });
    out.push(t);
  }
  return out;
}
/* squeeze any text marked data-fit="<px>" that is wider than that (once it is on screen) */
function ckFit(svg) {
  const run = () => svg.querySelectorAll("text[data-fit]").forEach(t => {
    const m = +t.dataset.fit;
    try { if (m && t.getComputedTextLength() > m) { t.setAttribute("textLength", m); t.setAttribute("lengthAdjust", "spacingAndGlyphs"); } } catch (e) { /* not rendered yet */ }
  });
  requestAnimationFrame(() => requestAnimationFrame(run));
}
const ckT = (parent, x, y, text, cls, size, fit, extra = {}) => {
  const { "text-anchor": anchor, ...rest } = extra;
  const t = s("text", { x: x.toFixed(1), y: y.toFixed(1), class: cls, "font-size": size, "data-fit": fit || null, ...rest }, parent);
  if (anchor) t.style.textAnchor = anchor; // inline, so the class rules (middle) do not win
  t.textContent = text; return t;
};

/* ── Sarvatobhadra ─────────────────────────────────────────────────────────────────────────────────────────────── */
function ckSbc(v) {
  if (!v) return missing("The Sarvatobhadra chakra", "The engine did not return vedic.chakra.sarvatobhadra.");
  // C = cell; the frame outside the grid has three bands: pada numbers (PB, next to the grid), pada sounds (SB) and
  // nakshatra names (NB, outermost)
  const set = CK_SET, list = (v.grahas[set] || []), C = 72, PB = 13, SB = 19, NB = 18, F = PB + SB + NB, M = 6, G0 = M + F, W = 9 * C + 2 * G0, G1 = G0 + 9 * C;
  const L = i => G0 + i * C, X = c => L(c) + C / 2, Y = r => L(r) + C / 2;
  const natOf = id => ckNat(list, id), transit = set === "transit";
  const svg = s("svg", { viewBox: `0 0 ${W} ${W}`, class: "ck-svg ck-sbc", role: "img", "aria-label": "Sarvatobhadra chakra, 9 by 9, with the nakshatra, pada sound and pada bands round it" });
  const gFrame = s("g", { class: "ck-frame" }, svg), gCells = s("g", { class: "ck-cells" }, svg), grid = s("g", { class: "ck-gridlines" }, svg),
    gLines = s("g", { class: "ck-lines" }, svg), gText = s("g", { class: "ck-texts" }, svg), gChips = s("g", { class: "ck-chips" }, svg);

  /* the frame: a point `off` px out from the grid edge, `frac` of the way along a border cell in clockwise order */
  const sideOf = c => c.row === 0 ? "top" : c.col === 8 ? "right" : c.row === 8 ? "bottom" : "left";
  const fxy = (side, c, off, frac) => side === "top" ? [L(c.col) + frac * C, G0 - off] : side === "right" ? [G1 + off, L(c.row) + frac * C]
    : side === "bottom" ? [L(c.col) + C - frac * C, G1 + off] : [G0 - off, L(c.row) + C - frac * C];
  const frect = (side, c, o0, o1, f0, f1, cls, parent) => {
    const [x0, y0] = fxy(side, c, o0, f0), [x1, y1] = fxy(side, c, o1, f1);
    return s("rect", { x: Math.min(x0, x1).toFixed(1), y: Math.min(y0, y1).toFixed(1), width: Math.abs(x1 - x0).toFixed(1), height: Math.abs(y1 - y0).toFixed(1), class: cls }, parent);
  };
  const ROT = { top: 0, right: 90, bottom: 0, left: -90 };
  const rotAt = (side, x, y) => ROT[side] ? `rotate(${ROT[side]} ${x.toFixed(1)} ${y.toFixed(1)})` : null;
  let fi = 0;
  for (const cell of v.cells.filter(c => c.kind === "nakshatra")) {
    const side = sideOf(cell), g = s("g", { class: "ck-fcell", "data-index": cell.index, "data-side": side, tabindex: 0 }, gFrame);
    g.style.setProperty("--i", fi++);
    frect(side, cell, PB + SB, F, 0, 1, "ck-fb ck-fname", g);
    const pads = cell.padas || [];
    pads.forEach((p, k) => {
      frect(side, cell, PB, PB + SB, k / 4, (k + 1) / 4, "ck-fb ck-fsound", g);
      frect(side, cell, 0, PB, k / 4, (k + 1) / 4, "ck-fb ck-fpada", g);
      const [sx, sy] = fxy(side, cell, PB + SB / 2, (k + .5) / 4), [px, py] = fxy(side, cell, PB / 2, (k + .5) / 4);
      ckT(g, sx, sy, p[0], "ck-syl", 12, 17, { "data-pada": k + 1 });
      ckT(g, px, py, String(k + 1), "ck-padn", 8.5);
    });
    const [nx, ny] = fxy(side, cell, PB + SB + NB / 2, .5);
    ckT(g, nx, ny, CK_SN[cell.index], "ck-fn", 10.5, C - 8, { transform: rotAt(side, nx, ny) });
    ckHover(g, () => [`${cell.dev} · ${cell.tr}`, pads.map((p, k) => [`Pada ${k + 1}`, `${p[0]} ${p[1]}`]), "the sound of each pada (nāmākṣara)"]);
  }
  // band names at the start of each band (clockwise): top-left, right-top, bottom-right, left-bottom
  const BANDS = [["Nakshatra", PB + SB + NB / 2], ["pada sound", PB + SB / 2], ["Pada", PB / 2]];
  for (const [txt, off] of BANDS) {
    ckT(gFrame, L(1) - 4, G0 - off, txt, "ck-band", 8, C - 8, { "text-anchor": "end" });
    ckT(gFrame, G1 + off, L(1) - 4, txt, "ck-band", 8, C - 8, { "text-anchor": "end", transform: `rotate(90 ${G1 + off} ${L(1) - 4})` });
    ckT(gFrame, L(8) + 4, G1 + off, txt, "ck-band", 8, C - 8, { "text-anchor": "start" });
    ckT(gFrame, G0 - off, L(8) + 4, txt, "ck-band", 8, C - 8, { "text-anchor": "end", transform: `rotate(-90 ${G0 - off} ${L(8) + 4})` });
  }

  /* the 81 cells */
  const targets = new Set((v.vedha_received || []).map(t => t.cell.join(",")));
  const chipAt = {};
  for (const cell of v.cells) {
    const key = cell.row + "," + cell.col, r = cell.row, c = cell.col, top = L(r), left = L(c), cx = X(c), cy = Y(r);
    const corner = (r === 0 || r === 8) && (c === 0 || c === 8);
    const by = (cell.pierced_by || []).filter(p => p.set === set);
    const mal = by.some(p => p.nature === "malefic"), ben = by.some(p => p.nature === "benefic");
    const ved = mal && ben ? "both" : mal ? "malefic" : ben ? "benefic" : "";
    const g = s("g", { class: `ck-cell k-${cell.kind}` + (corner ? " k-corner" : "") + (cell.birth_tithi || cell.birth_weekday ? " birth" : "") + (targets.has(key) ? " target" : ""),
      "data-row": r, "data-col": c, "data-kind": cell.kind, "data-vedha": ved, tabindex: 0 }, gCells);
    g.style.setProperty("--i", r + c);
    s("rect", { x: left + 1, y: top + 1, width: C - 2, height: C - 2, rx: 4, class: "ck-bg" }, g);
    if (mal) s("rect", { x: left + 3, y: top + 3, width: C - 6, height: C - 6, rx: 3, class: "ck-vb mal" }, g);
    if (ben) s("rect", { x: left + (mal ? 6.5 : 3), y: top + (mal ? 6.5 : 3), width: C - (mal ? 13 : 6), height: C - (mal ? 13 : 6), rx: 2.5, class: "ck-vb ben" }, g);
    if (targets.has(key) || cell.birth_tithi || cell.birth_weekday) s("path", { d: `M${left + C - 15},${top + 2} H${left + C - 2} V${top + 15} Z`, class: "ck-flag" }, g);
    const tg = s("g", { class: "ck-ct", "data-row": r, "data-col": c }, gText);
    tg.style.setProperty("--i", r + c);
    const occ = (transit ? cell.transit : cell.natal) || [];
    if (cell.kind === "nakshatra") {
      // chips sit along the inner edge (where the vedha lines start); the text uses the rest of the cell
      const side = sideOf(cell), n = occ.length, step = Math.min(16, (C - 12) / Math.max(1, n));
      const horiz = side === "top" || side === "bottom";
      const tx = side === "right" ? cx + 8 : side === "left" ? cx - 8 : cx, fit = horiz ? C - 10 : C - 26;
      const y0 = side === "top" ? top + 13 : side === "bottom" ? top + 28 : cy - 15;
      ckT(tg, tx, y0, cell.dev, "ck-dev", cell.dev.length > 7 ? 10 : 11.5, fit);
      ckGtext(tg, tx, y0 + 15, occ, natOf, 3, { transit, fit });
      occ.forEach((id, i) => {
        const along = (i - (n - 1) / 2) * step;
        const at = side === "top" ? [cx + along, top + C - 9] : side === "bottom" ? [cx + along, top + 9] : side === "right" ? [left + 9, cy + along] : [left + C - 9, cy + along];
        ckMark(gChips, id, at[0], at[1], 7, i, transit);
        chipAt[id] = at;
      });
    } else if (cell.kind === "rashi") {
      ckT(tg, cx, top + 13, CK_RA[cell.index], "ck-ra", 11.5, C - 8);
      ckT(tg, cx, top + 27, cell.dev, "ck-tr", 10, C - 8);
      ckGtext(tg, cx, top + 43, occ, natOf, 3, { transit, fit: C - 8 });
    } else if (cell.kind === "tithi") {
      ckT(tg, cx, top + 16, cell.dev, "ck-tr", 10.5, C - 8);
      ckT(tg, cx, top + 34, cell.weekdays.map(x => x.slice(0, 3)).join(" "), "ck-wd2", 11, C - 8);
      ckT(tg, cx, top + 51, cell.tr, "ck-tn", 10.5, C - 8);
    } else {
      ckT(tg, cx, cy - 5, cell.dev, "ck-dev ck-big", corner ? 30 : 24, C - 10);
      ckT(tg, cx, top + C - 12, cell.tr, "ck-tr", 9.5, C - 10);
    }
    ckHover(g, () => {
      const lines = by.map(p => `${ckEn(p.graha)} (${p.nature}): ${CK_LINE[p.line]}${p.emphasised ? ", its emphasised line" : ""}`);
      const rows = [["What", CK_KIND[cell.kind]], ["Natal here", ckList(cell.natal)], ["Transit here", ckList(cell.transit)],
        [`Pierced by (${set})`, lines.length ? lines.join("; ") : "none"],
        ["Border", ved === "both" ? "red and green: malefic and benefic vedha" : ved === "malefic" ? "red: malefic vedha" : ved === "benefic" ? "green: benefic vedha" : "none"]];
      if (cell.kind === "nakshatra" && cell.padas) rows.splice(1, 0, ["Pada sounds", cell.padas.map(p => p[0]).join(" ")]);
      if (cell.kind === "tithi") rows.splice(1, 0, ["Tithis", cell.tithis.join(", ") + " of each half"], ["Weekdays", cell.weekdays.join(", ")]);
      const tgt = (v.vedha_received || []).filter(t => t.cell.join(",") === key).map(t => t.label);
      if (tgt.length) rows.push(["Personal cell", tgt.join(", ")]);
      if (cell.birth_tithi) rows.push(["Birth", "the birth tithi falls in this group"]);
      if (cell.birth_weekday) rows.push(["Birth", "the birth weekday shares this cell"]);
      return [`${cell.dev} · ${cell.tr}`, rows, `row ${r + 1}, column ${c + 1}`];
    });
  }
  for (let i = 0; i <= 9; i++) {
    const p = L(i);
    const a = s("line", { x1: G0, x2: G1, y1: p, y2: p, class: "ck-grid", pathLength: 1 }, grid); a.style.animationDelay = i * 40 + "ms";
    const b = s("line", { x1: p, x2: p, y1: G0, y2: G1, class: "ck-grid", pathLength: 1 }, grid); b.style.animationDelay = i * 40 + 20 + "ms";
  }
  s("rect", { x: M, y: M, width: W - 2 * M, height: W - 2 * M, rx: 6, class: "ck-frame-edge" }, gFrame);
  list.forEach((gr, gi) => {
    if (!gr.lines) return;
    const [x1, y1] = chipAt[gr.id] || [X(gr.cell[1]), Y(gr.cell[0])];
    for (const name of ["front", "left", "right"]) {
      const cells = gr.lines[name]; if (!cells || !cells.length) continue;
      const end = cells[cells.length - 1], nat = gr.nature === "benefic" ? "ben" : "mal";
      const ln = s("line", { x1: x1.toFixed(1), y1: y1.toFixed(1), x2: X(end[1]), y2: Y(end[0]), pathLength: 1,
        class: `ck-line l-${name} ${nat}` + (gr.emphasis === name ? " emph" : ""), "data-graha": gr.id, "data-line": name }, gLines);
      ln.style.animationDelay = (600 + gi * 70) + "ms";
    }
  });
  const all = q => Array.from(svg.querySelectorAll(q));
  const hits = id => {
    all(".ck-line").forEach(l => l.classList.toggle("on", l.dataset.graha === id));
    const gr = list.find(g => g.id === id);
    const on = new Set(gr && gr.lines ? ["front", "left", "right"].flatMap(n => gr.lines[n].map(rc => rc.join(","))) : []);
    all(".ck-cell").forEach(c => c.classList.toggle("hit", on.has(c.dataset.row + "," + c.dataset.col)));
  };
  const picker = ckPicker(svg, list, hits);
  ckFit(svg);
  const fmt = arr => (arr || []).length ? arr.map(p => `${ckAb(p.graha)} ${p.line}${p.emphasised ? "*" : ""}`).join(", ") : "—";
  const rec = (v.vedha_received || []).map(t => [t.label, h("span", {}, h("b", { class: "ck-devi" }, t.dev), " ", t.name), fmt(t.natal),
    v.grahas.transit ? fmt(t.transit) : "—",
    `${t.natal_malefics} / ${t.natal_benefics}` + (t.transit_malefics != null ? ` · ${t.transit_malefics} / ${t.transit_benefics}` : ""),
    ckList(t.occupants) || "—"]);
  const gl = list.map(g => [g.id === "lagna" ? "Lagna" : glink(g.id), g.nakshatra, g.rashi, g.motion + (g.retrograde ? " (retrograde)" : ""),
    g.id === "lagna" ? "—" : CK_LINE[g.emphasis], g.nature]);
  const key = h("div", { class: "ck-key" },
    h("span", {}, h("i", { class: "sw mal" }), "red border: pierced by a malefic"), h("span", {}, h("i", { class: "sw ben" }), "green: by a benefic"),
    h("span", {}, h("i", { class: "sw both" }), "red and green: both"), h("span", {}, h("i", { class: "sw flag" }), "corner flag: a personal cell"),
    h("span", {}, h("b", { class: "ck-g mal" }, "Sa"), " ", h("b", { class: "ck-g ben" }, "Ju"), " malefic, benefic graha"));
  return [
    h("div", { class: "toolbar" }, picker),
    card("Sarvatobhadra chakra · सर्वतोभद्र चक्र", h("div", { class: "figure ck-fig ck-fig-sbc" }, svg), key,
      note(label("tradition_rule"), ` East at the top, south right, west bottom, north left. Round the grid: each border nakshatra’s name, the sounds of its four padas and the pada numbers, read clockwise. Corners: the vowels अ आ इ ई; then the letters and vowels, the twelve rāśis and, in the centre, the five tithi groups with their weekdays. Grahas are printed in their nakshatra and rāśi cells (${transit ? "transit" : "natal"} layer); the chips on a nakshatra cell’s inner edge are where its vedha lines start. Hover any cell; pick a graha to see its three lines (the emphasised one thicker).`)),
    card("Vedha received", table(["Target", "Cell", "Natal vedha", "Transit vedha", "Malefic / benefic (natal · transit)", "Natal occupants"], rec),
      note("* marks a graha’s emphasised line. Convention: ", v.convention, ".")),
    card(transit ? "Transit grahas" : "Natal grahas", table(["Graha", "Nakshatra", "Rāśi", "Motion", "Emphasised line", "Nature"], gl)),
    ckHow(note(label("tradition_rule"), " A graha standing in a nakshatra cell throws three lines across the grid: straight ahead to the far side, and the two diagonals. Every cell a line crosses (nakshatras, letters, vowels, rāśis, tithi groups and weekdays) receives its vedha; the border colour says whose (red malefic, green benefic, both). The cells that matter for a person are the Janma nakshatra, the lagna nakshatra, the Moon and lagna rāśis, the birth tithi group, the birth weekday and the first sound of the name (find it in the pada sound band); they carry a corner flag."),
      note(label("interpretation"), " In the Narapati-jayacharya tradition, as modern transit manuals paraphrase it, vedha from benefics on these personal cells is read as support and vedha from malefics as obstruction or loss; several malefics piercing the Janma nakshatra together are read as a difficult period, and a retrograde graha is said to strike harder. This is tradition, not prediction.")),
  ];
}

/* ── Kota ──────────────────────────────────────────────────────────────────────────────────────────────────────── */
const CK_ZN = { bahya: "Bāhya", prakara: "Prākāra", madhya: "Madhya", stambha: "Stambha" };
function ckKota(v) {
  if (!v) return missing("The Kota chakra", "The Kota chakra needs the Moon.");
  const set = CK_SET, transit = set === "transit", rows = (transit ? v.transit : v.natal) || [], R = 320, P = 60, W = 2 * (R + P);
  const X = u => W / 2 + u * R;
  const natOf = id => ckNat(rows, id);
  const svg = s("svg", { viewBox: `0 0 ${W} ${W}`, class: "ck-svg ck-kota", role: "img", "aria-label": "Kota chakra: the fort of four zones with 28 nakshatras" });
  const clip = "ck-fort-" + Math.random().toString(36).slice(2, 8);
  s("rect", { x: X(-1), y: X(-1), width: 2 * R, height: 2 * R }, s("clipPath", { id: clip }, s("defs", {}, svg)));
  const zones = s("g", { class: "ck-zones" }, svg);
  ["bahya", "prakara", "madhya", "stambha"].forEach((z, i) => {
    const q = v.zone_size[z];
    const r = s("rect", { x: X(-q), y: X(-q), width: 2 * q * R, height: 2 * q * R, class: "ck-zone z-" + z, pathLength: 1, "data-zone": z }, zones);
    r.style.animationDelay = i * 160 + "ms";
  });
  // the paths in and out: two diagonals and the two cross bands, one union so the overlaps are not darker
  const d = 0.1, b = 0.125, o = d * Math.SQRT1_2, E = 1.1;
  const polys = [[[-b, -E], [b, -E], [b, E], [-b, E]], [[-E, -b], [E, -b], [E, b], [-E, b]],
    [[-E + o, -E - o], [E + o, E - o], [E - o, E + o], [-E - o, -E + o]], [[E + o, -E + o], [-E + o, E + o], [-E - o, E - o], [E - o, -E - o]]];
  const area = pts => pts.reduce((a, p, i) => { const q = pts[(i + 1) % pts.length]; return a + p[0] * q[1] - q[0] * p[1]; }, 0);
  const dPath = polys.map(pts => (area(pts) < 0 ? pts.slice().reverse() : pts)).map(pts => "M" + pts.map(([x, y]) => `${X(x).toFixed(1)},${X(y).toFixed(1)}`).join(" L") + " Z").join(" ");
  s("path", { d: dPath, class: "ck-bands", "clip-path": `url(#${clip})`, "fill-rule": "nonzero" }, svg);
  const labels = s("g", { class: "ck-zlabels" }, svg);
  ["bahya", "prakara", "madhya", "stambha"].forEach(z => {
    const q = v.zone_size[z], st = z === "stambha";
    ckT(labels, X(st ? -q : -q + 0.25) + 5, X(-q) + 11, CK_ZN[z], "ck-zlab2", st ? 8.5 : 9.5, null, { "data-zone": z, "text-anchor": "start" });
  });
  // arrows: entering at each corner along the diagonal, leaving at the middle of each side
  const arrows = s("g", { class: "ck-arrows" }, svg);
  const arrow = (u1, v1, u2, v2, cls, k) => {
    const x1 = X(u1), y1 = X(v1), x2 = X(u2), y2 = X(v2), dx = x2 - x1, dy = y2 - y1, len = Math.hypot(dx, dy), ux = dx / len, uy = dy / len, nx = -uy, ny = ux;
    const sw = 4.5, hw = 11, hl = 16, bx = x2 - ux * hl, by = y2 - uy * hl;
    const pts = [[x1 + nx * sw, y1 + ny * sw], [bx + nx * sw, by + ny * sw], [bx + nx * hw, by + ny * hw], [x2, y2], [bx - nx * hw, by - ny * hw], [bx - nx * sw, by - ny * sw], [x1 - nx * sw, y1 - ny * sw]];
    const a = s("polygon", { points: pts.map(p => p.map(n => n.toFixed(1)).join(",")).join(" "), class: "ck-arrow " + cls }, arrows);
    a.style.setProperty("--k", k);
    return a;
  };
  [[-1, -1], [1, -1], [1, 1], [-1, 1]].forEach(([sx, sy], k) => arrow(sx * 1.15, sy * 1.15, sx * 1.012, sy * 1.012, "in", k));
  [[0, -1], [1, 0], [0, 1], [-1, 0]].forEach(([sx, sy], k) => arrow(sx * 1.012, sy * 1.012, sx * 1.16, sy * 1.16, "out", k + 4));
  const DIRS = [["East", X(0) + 16, X(-1.09), "start"], ["South", X(1.09), X(0) + 24, "middle"], ["West", X(0) + 16, X(1.09), "start"], ["North", X(-1.09), X(0) + 24, "middle"]];
  for (const [n, x, y, an] of DIRS) ckT(svg, x, y, n, "ck-side", 11, null, { "text-anchor": an });

  const byPlace = {};
  rows.forEach(r => (byPlace[r.position] = byPlace[r.position] || []).push(r));
  const gPlaces = s("g", { class: "ck-places" }, svg);
  v.layout.forEach((p, i) => {
    const sx = Math.sign(p.x), sy = Math.sign(p.y), m = Math.max(Math.abs(p.x), Math.abs(p.y));
    let ax, ay, cell;
    if (p.zone === "stambha") {
      ax = sx * 0.125; ay = sy * 0.16;
      cell = sy ? [-0.25, sy < 0 ? -0.25 : 0.07, 0.25, sy < 0 ? -0.07 : 0.25] : [sx < 0 ? -0.25 : 0, -0.07, sx < 0 ? 0 : 0.25, 0.07];
    } else {
      ax = sx * (m - 0.125); ay = sy * (m - 0.125);
      cell = [ax - 0.125, ay - 0.125, ax + 0.125, ay + 0.125];
    }
    const g = s("g", { class: "ck-place" + (p.position === 1 ? " janma" : "") + " z-" + p.zone + " p-" + p.place, "data-position": p.position, "data-zone": p.zone, tabindex: 0 }, gPlaces);
    g.style.setProperty("--k", i);
    s("rect", { x: X(cell[0]) + 1.5, y: X(cell[1]) + 1.5, width: (cell[2] - cell[0]) * R - 3, height: (cell[3] - cell[1]) * R - 3, rx: 4, class: "ck-hit" }, g);
    const here = (byPlace[p.position] || []).map(r => r.id), lines = 1 + Math.ceil(here.length / 3), lh = 12.5, fit = Math.round((cell[2] - cell[0]) * R - 8);
    const y0 = X(ay) - (lines - 1) * lh / 2;
    const t = s("text", { x: X(ax).toFixed(1), y: y0.toFixed(1), class: "ck-kn", "font-size": 10.5, "data-fit": fit }, g);
    s("tspan", { class: "ck-knum" }, t).textContent = p.position + ". ";
    s("tspan", {}, t).textContent = CK_SN[p.index];
    ckGtext(g, X(ax), y0 + lh, here, natOf, 3, { transit, click: true, lh, fit });
    ckHover(g, () => [`${p.position}. ${p.dev} · ${p.tr}`, [["Zone", CK_ZONE[p.zone]], ["Direction", p.direction],
      ["Path", p.place === "core" ? "the stambha (the core)" : p.place === "entering" ? "entering the fort (corner to centre)" : "leaving the fort (centre to edge)"],
      ["Natal here", ckList((v.natal || []).filter(r => r.position === p.position).map(r => r.id))],
      ["Transit here", ckList((v.transit || []).filter(r => r.position === p.position).map(r => r.id))],
      ...(byPlace[p.position] || []).map(r => [ckEn(r.id), `${r.nature}, ${r.path}${r.retrograde ? " (retrograde)" : ""}`])], `counted from the Janma nakshatra ${v.janma.tr}`]);
  });
  // bottom-left inside the fort: Kota swāmī and pāla (the engine computes no Kota score) and the layer's moment
  const subj = D.subject || {};
  const info = s("g", { class: "ck-kinfo" }, svg), ix = X(-0.75) + 8;
  const sw = v.kota_swami, pl = v.kota_pala;
  const line = (y, k, val) => { const t = s("text", { x: ix, y, class: "ck-ki", "data-fit": Math.round(0.6 * R) }, info); t.style.textAnchor = "start"; s("tspan", { class: "k" }, t).textContent = k + " "; s("tspan", {}, t).textContent = val; return t; };
  line(X(0.8), "Kota swāmī", `${ckEn(sw.graha)}${sw[set] ? ` · place ${sw[set].position}, ${sw[set].zone}` : ""}`);
  line(X(0.8) + 15, "Kota pāla", `${ckEn(pl.graha)}${pl[set] ? ` · place ${pl[set].position}, ${pl[set].zone}` : ""}`);
  line(X(0.8) + 30, transit ? "Transit" : "Natal", transit ? fmtTime(v.transit_utc).slice(0, 16) + " UTC" : (subj.local ? fmtTime(subj.local).slice(0, 16) : "the birth moment"));
  ckFit(svg);

  const sum = transit ? v.transit_summary : v.natal_summary;
  const where = o => o ? `${o.nakshatra} · ${o.zone}, ${o.path}` : "—";
  const tiles = h("div", { class: "stats" },
    stat("Janma nakshatra", `${v.janma.dev} ${v.janma.tr}`, "place 1, top-left corner (east)"),
    stat("Kota swāmī", ckEn(v.kota_swami.graha), `${v.kota_swami.rule} · ${set}: ${where(v.kota_swami[set])}`, GR[v.kota_swami.graha] ? "#/planet/" + v.kota_swami.graha : null),
    stat("Kota pāla", ckEn(v.kota_pala.graha), `${v.kota_pala.rule} (${v.kota_pala.navamsha}) · ${set}: ${where(v.kota_pala[set])}`, GR[v.kota_pala.graha] ? "#/planet/" + v.kota_pala.graha : null),
    stat("Malefics inside", String(sum.malefics_inside.length), ckList(sum.malefics_inside) || "none"),
    stat("Benefics inside", String(sum.benefics_inside.length), ckList(sum.benefics_inside) || "none"));
  const tbl = table(["Graha", "Nakshatra", "Place", "Zone", "Direction", "Moving", "Nature"],
    rows.map(r => [GR[r.id] ? glink(r.id) : ckEn(r.id), r.nakshatra, r.position, CK_ZONE[r.zone].split(" (")[0], r.direction, r.path + (r.retrograde ? " (retrograde)" : ""), r.nature]));
  const moving = h("div", { class: "facts-wrap" }, facts([
    ["Malefics entering", ckList(sum.malefics_entering) || "none"], ["Malefics leaving", ckList(sum.malefics_leaving) || "none"],
    ["Benefics entering", ckList(sum.benefics_entering) || "none"], ["Benefics leaving", ckList(sum.benefics_leaving) || "none"],
    ["In the stambha", ckList(sum.in_stambha) || "none"]]));
  const key = h("div", { class: "ck-key" },
    h("span", {}, h("i", { class: "sw z1" }), "Bāhya"), h("span", {}, h("i", { class: "sw z2" }), "Prākāra"), h("span", {}, h("i", { class: "sw z3" }), "Madhya"),
    h("span", {}, h("i", { class: "sw z4" }), "Stambha"), h("span", {}, h("i", { class: "sw band" }), "paths in (diagonals) and out (cross bands)"),
    h("span", {}, h("b", { class: "ck-g mal" }, "Sa"), " ", h("b", { class: "ck-g ben" }, "Ju"), " malefic, benefic graha"));
  return [tiles,
    card("Kota chakra · कोट चक्र", h("div", { class: "figure ck-fig" }, svg), key,
      note(label("tradition_rule"), " Twenty-eight nakshatras numbered from the Janma nakshatra, seven per direction: each direction enters at its corner (arrow in) through bāhya, prākāra and madhya to the stambha, then leaves along its band (arrow out) through madhya, prākāra and bāhya. Grahas are printed under the nakshatra they occupy; hover a place for its zone and path, click a natal graha for its page.")),
    card(transit ? "Transit grahas in the fort" : "Natal grahas in the fort", tbl, moving, note("Convention: ", v.convention, ".")),
    ckHow(note(label("tradition_rule"), " The fort (koṭa) has four walls: stambha at the core, madhya, prākāra (the wall itself) and bāhya outside. A graha in a corner-to-centre place is entering, one in a centre-to-edge place is leaving; retrograde grahas move the other way."),
      note(label("interpretation"), " The tradition reads malefics inside the fort or entering it, especially in the stambha, as pressure on health and affairs, and benefics inside as protection; a strong Kota swāmī and Kota pāla are read as a well-guarded fort. Paraphrased from the Narapati-jayacharya tradition as modern manuals present it; this is tradition, not prediction.")),
  ];
}

/* ── Saṅghaṭṭa ─────────────────────────────────────────────────────────────────────────────────────────────────── */
function ckSangh(v) {
  if (!v) return missing("The Saṅghaṭṭa chakra", "The engine did not return vedic.chakra.sanghatta.");
  const set = CK_SET, rows = (set === "natal" ? v.natal : v.transit) || [], R = 270, RI = 218, W = 2 * R + 40, c = W / 2;
  const pt = (deg, r) => [c - r * Math.sin(deg * DEG), c - r * Math.cos(deg * DEG)];
  const svg = s("svg", { viewBox: `0 0 ${W} ${W}`, class: "ck-svg ck-sangh", role: "img", "aria-label": "Saṅghaṭṭa chakra" });
  s("circle", { cx: c, cy: c, r: R, class: "ck-ring", pathLength: 1 }, svg);
  s("circle", { cx: c, cy: c, r: RI, class: "ck-ring", pathLength: 1 }, svg);
  const span = 360 / v.ring.length;
  const chords = s("g", { class: "ck-chords" }, svg);
  v.pairs_index.forEach(([a, b], i) => {
    const [x1, y1] = pt(v.ring[a].angle_deg, RI - 4), [x2, y2] = pt(v.ring[b].angle_deg, RI - 4);
    const l = s("line", { x1, y1, x2, y2, class: "ck-chord", pathLength: 1, "data-a": a, "data-b": b }, chords);
    l.style.animationDelay = 500 + i * 60 + "ms";
  });
  const occ = {};
  rows.forEach(r => (occ[r.nakshatra_index] = occ[r.nakshatra_index] || []).push(r.id));
  v.ring.forEach((n, i) => {
    const a0 = n.angle_deg - span / 2, a1 = n.angle_deg + span / 2;
    const [ox0, oy0] = pt(a0, R), [ox1, oy1] = pt(a1, R), [ix1, iy1] = pt(a1, RI), [ix0, iy0] = pt(a0, RI);
    const g = s("g", { class: "ck-sector" + (i === v.janma_index ? " janma" : "") + (i === v.lagna_index ? " lagna" : ""), "data-index": i, tabindex: 0 }, svg);
    g.style.setProperty("--i", i);
    s("path", { d: `M${ox0.toFixed(1)},${oy0.toFixed(1)} A${R},${R} 0 0 0 ${ox1.toFixed(1)},${oy1.toFixed(1)} L${ix1.toFixed(1)},${iy1.toFixed(1)} A${RI},${RI} 0 0 1 ${ix0.toFixed(1)},${iy0.toFixed(1)} Z` }, g);
    const [dx, dy] = pt(n.angle_deg, R - 17), [tx, ty] = pt(n.angle_deg, R - 36);
    const d = s("text", { x: dx, y: dy, class: "ck-dev", "font-size": n.dev.length > 7 ? 8.5 : 10.5 }, g); d.textContent = n.dev;
    const t = s("text", { x: tx, y: ty, class: "ck-tr" }, g); t.textContent = CK_AB[i];
    (occ[i] || []).forEach((id, k) => { const [mx, my] = pt(n.angle_deg, RI - 20 - k * 19); ckMark(svg, id, mx, my, 8.5, i % 9 + k, set === "transit"); });
    ckHover(g, () => {
      const by = r => (r || []).filter(x => (x.pierces_index || []).includes(i)).map(x => ckEn(x.id));
      return [`${n.dev} · ${n.tr}`, [["Vedha partner", n.partners.join(", ")], ["Natal here", ckList(v.natal.filter(x => x.nakshatra_index === i).map(x => x.id))],
        ["Transit here", v.transit ? ckList(v.transit.filter(x => x.nakshatra_index === i).map(x => x.id)) : ""],
        ["Pierced by (natal)", by(v.natal).join(", ") || "none"], ["Pierced by (transit)", v.transit ? by(v.transit).join(", ") || "none" : ""]],
        i === v.janma_index ? "the Janma nakshatra" : i === v.lagna_index ? "the lagna nakshatra" : ""];
    });
  });
  const all = q => Array.from(svg.querySelectorAll(q));
  const hits = id => {
    const r = rows.find(x => x.id === id), at = r ? r.nakshatra_index : -1;
    all(".ck-chord").forEach(l => l.classList.toggle("on", +l.dataset.a === at || +l.dataset.b === at));
    all(".ck-sector").forEach(g => g.classList.toggle("hit", r ? (r.pierces_index || []).includes(+g.dataset.index) : false));
  };
  const picker = ckPicker(svg, rows, hits);
  const tbl = table(["Graha", "Nakshatra", "Pierces", "Nature"], rows.map(r => [r.id === "lagna" ? "Lagna" : GR[r.id] ? glink(r.id) : ckEn(r.id), r.nakshatra, (r.pierces || []).join(", ") || "—", r.nature]));
  const rec = table(["Nakshatra", "Pierced by (natal)", "Pierced by (transit)"], (v.received || []).map(x => [x.name, ckList(x.natal) || "—", ckList(x.transit) || "—"]));
  return [
    h("div", { class: "toolbar" }, picker),
    card("Saṅghaṭṭa chakra · सङ्घट्ट चक्र", h("div", { class: "figure ck-fig" }, svg),
      note(label("tradition_rule"), " Twenty-eight nakshatras round the ring, Aśvinī at the top, counterclockwise. Each line joins two nakshatras that pierce each other. Pick a graha to light the line from its nakshatra.")),
    card(set === "natal" ? "What each natal graha pierces" : "What each transit graha pierces", tbl, note("Convention: ", v.convention, ".")),
    card("Nakshatras receiving vedha", rec),
    ckHow(note(label("tradition_rule"), " The pairs are the seven vertical and seven horizontal lines of the sapta-śalākā figure: the two nakshatras at the ends of one line are in vedha. A graha in one of them pierces the other."),
      note(label("interpretation"), " Muhūrta texts avoid a nakshatra pierced by a malefic for beginnings such as marriage, and read a benefic there as mild; applied to a chart, malefic vedha on the Janma nakshatra is read as strain. Paraphrased from the muhūrta tradition; this is tradition, not prediction.")),
  ];
}

/* ── the page ──────────────────────────────────────────────────────────────────────────────────────────────────── */
page("chakras", () => "Chakras", "Analysis", (b) => {
  const sbc = val("vedic.chakra.sarvatobhadra"), kota = val("vedic.chakra.kota"), sg = val("vedic.chakra.sanghatta");
  const head = pageHead("chakras", "Chakras", "Sarvatobhadra, Kota and Saṅghaṭṭa: three classical nakshatra chakras, natal and in transit.");
  if (!sbc && !kota && !sg) return put(b, head, missing("The chakras", "Ask for the chakras feature (features: chakras, or all)."));
  const tutc = (sbc && sbc.transit_utc) || (kota && kota.transit_utc);
  if (!tutc && CK_SET === "transit") CK_SET = "natal";
  const pick = k => { CK_TAB = k; CK_PICK = null; route(); };
  const bar = h("div", { class: "toolbar" },
    h("div", { class: "grp" }, h("b", {}, "Chakra"), tabs([["sbc", "Sarvatobhadra"], ["kota", "Kota"], ["sanghatta", "Saṅghaṭṭa"]], CK_TAB, pick)),
    tutc ? h("div", { class: "grp" }, h("b", {}, "Layer"), tabs([["natal", "Natal"], ["transit", "Transit"]], CK_SET, k => { CK_SET = k; route(); })) : null);
  const when = tutc ? note(`Transit layer: ${fmtTime(tutc)} (the report reference instant).`) : note("No reference instant was given, so only natal placements are shown.");
  const body = CK_TAB === "kota" ? ckKota(kota) : CK_TAB === "sanghatta" ? ckSangh(sg) : ckSbc(sbc);
  const warn = [sbc, kota, sg].find(x => x && x.note);
  put(b, head, bar, when, warn ? note(warn.note) : null, body);
}, { blurb: "Sarvatobhadra, Kota and Saṅghaṭṭa" });
