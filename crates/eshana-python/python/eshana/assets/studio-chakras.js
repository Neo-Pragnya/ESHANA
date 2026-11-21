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
  const g = s("g", { class: "ck-pl" + (transit ? " tr" : ""), "data-graha": id, transform: `translate(${x.toFixed(1)} ${y.toFixed(1)})` }, parent);
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

/* ── Sarvatobhadra ─────────────────────────────────────────────────────────────────────────────────────────────── */
function ckSbc(v) {
  if (!v) return missing("The Sarvatobhadra chakra", "The engine did not return vedic.chakra.sarvatobhadra.");
  const set = CK_SET, list = (v.grahas[set] || []), C = 74, M = 8, W = C * 9 + 2 * M;
  const X = c => M + c * C + C / 2, Y = r => M + r * C + C / 2;
  const svg = s("svg", { viewBox: `0 0 ${W} ${W}`, class: "ck-svg ck-sbc", role: "img", "aria-label": "Sarvatobhadra chakra, 9 by 9" });
  const targets = new Set((v.vedha_received || []).map(t => t.cell.join(",")));
  const gCells = s("g", { class: "ck-cells" }, svg);
  for (const cell of v.cells) {
    const key = cell.row + "," + cell.col;
    const g = s("g", { class: `ck-cell k-${cell.kind}` + (cell.birth_tithi || cell.birth_weekday ? " birth" : "") + (targets.has(key) ? " target" : ""),
      "data-row": cell.row, "data-col": cell.col, "data-kind": cell.kind, tabindex: 0 }, gCells);
    g.style.setProperty("--i", cell.row + cell.col);
    s("rect", { x: M + cell.col * C + 1.5, y: M + cell.row * C + 1.5, width: C - 3, height: C - 3, rx: 7, class: "ck-bg" }, g);
    const cx = X(cell.col), cy = Y(cell.row);
    if (cell.kind === "nakshatra") {
      const d = s("text", { x: cx, y: cy - 20, class: "ck-dev", "font-size": cell.dev.length > 7 ? 9.5 : 12 }, g); d.textContent = cell.dev;
      const t = s("text", { x: cx, y: cy - 6, class: "ck-tr" }, g); t.textContent = cell.tr.length > 11 ? CK_AB[cell.index] : cell.tr;
    } else if (cell.kind === "tithi") {
      const d = s("text", { x: cx, y: cy - 20, class: "ck-dev", "font-size": 14 }, g); d.textContent = cell.dev;
      const t = s("text", { x: cx, y: cy - 5, class: "ck-tr" }, g); t.textContent = cell.tr;
      const w = s("text", { x: cx, y: cy + 9, class: "ck-tr ck-wd" }, g); w.textContent = cell.weekdays.map(x => x.slice(0, 3)).join(" ");
    } else {
      const big = cell.kind === "rashi" ? 14 : 22;
      const d = s("text", { x: cx, y: cy - (cell.kind === "rashi" ? 18 : 14), class: "ck-dev", "font-size": big }, g); d.textContent = cell.dev;
      const t = s("text", { x: cx, y: cy + (cell.kind === "rashi" ? -3 : 6), class: "ck-tr" }, g); t.textContent = cell.tr;
    }
    const occ = (set === "natal" ? cell.natal : cell.transit) || [];
    occ.forEach((id, i) => {
      const per = 4, r = 8.5, row = Math.floor(i / per), col = i % per, n = Math.min(per, occ.length - row * per);
      ckMark(g, id, cx + (col - (n - 1) / 2) * 17.5, cy + 14 + row * 17, r, i, set === "transit");
    });
    ckHover(g, () => {
      const by = (cell.pierced_by || []).filter(p => p.set === set).map(p => `${ckEn(p.graha)}: ${CK_LINE[p.line]}${p.emphasised ? " (its emphasised line)" : ""}`);
      const rows = [["What", CK_KIND[cell.kind]], ["Natal here", ckList(cell.natal)], ["Transit here", ckList(cell.transit)],
        [`Pierced by (${set})`, by.length ? by.join("; ") : "none"]];
      if (cell.kind === "tithi") rows.splice(1, 0, ["Tithis", cell.tithis.join(", ") + " of each half"], ["Weekdays", cell.weekdays.join(", ")]);
      if (cell.birth_tithi) rows.push(["Birth", "the birth tithi falls in this group"]);
      if (cell.birth_weekday) rows.push(["Birth", "the birth weekday shares this cell"]);
      return [`${cell.dev} · ${cell.tr}`, rows, `row ${cell.row + 1}, column ${cell.col + 1}`];
    });
  }
  const grid = s("g", { class: "ck-gridlines" }, svg);
  for (let i = 0; i <= 9; i++) {
    const p = M + i * C;
    const a = s("line", { x1: M, x2: W - M, y1: p, y2: p, class: "ck-grid", pathLength: 1 }, grid); a.style.animationDelay = i * 40 + "ms";
    const b = s("line", { x1: p, x2: p, y1: M, y2: W - M, class: "ck-grid", pathLength: 1 }, grid); b.style.animationDelay = i * 40 + 20 + "ms";
  }
  const gLines = s("g", { class: "ck-lines" }, svg);
  list.forEach((gr, gi) => {
    if (!gr.lines) return;
    for (const name of ["front", "left", "right"]) {
      const cells = gr.lines[name]; if (!cells || !cells.length) continue;
      const end = cells[cells.length - 1];
      const ln = s("line", { x1: X(gr.cell[1]), y1: Y(gr.cell[0]), x2: X(end[1]), y2: Y(end[0]), pathLength: 1, stroke: (G[gr.id] || G.lagna).dot,
        class: `ck-line l-${name}` + (gr.emphasis === name ? " emph" : ""), "data-graha": gr.id, "data-line": name }, gLines);
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
  const fmt = arr => (arr || []).length ? arr.map(p => `${ckAb(p.graha)} ${p.line}${p.emphasised ? "*" : ""}`).join(", ") : "—";
  const rec = (v.vedha_received || []).map(t => [t.label, h("span", {}, h("b", { class: "ck-devi" }, t.dev), " ", t.name), fmt(t.natal),
    v.grahas.transit ? fmt(t.transit) : "—",
    `${t.natal_malefics} / ${t.natal_benefics}` + (t.transit_malefics != null ? ` · ${t.transit_malefics} / ${t.transit_benefics}` : ""),
    ckList(t.occupants) || "—"]);
  const gl = list.map(g => [g.id === "lagna" ? "Lagna" : glink(g.id), g.nakshatra, g.rashi, g.motion + (g.retrograde ? " (retrograde)" : ""),
    g.id === "lagna" ? "—" : CK_LINE[g.emphasis], g.nature]);
  return [
    h("div", { class: "toolbar" }, picker),
    card("Sarvatobhadra chakra · सर्वतोभद्र चक्र", h("div", { class: "figure ck-fig" }, svg),
      note(label("tradition_rule"), " East at the top, south right, west bottom, north left. Nakshatra cells carry the grahas by nakshatra and rāśi cells by sign; solid markers are natal, rings are transit. Hover any cell for what pierces it; pick a graha to see its three vedha lines (the emphasised one is thicker).")),
    card("Vedha received", table(["Target", "Cell", "Natal vedha", "Transit vedha", "Malefic / benefic (natal · transit)", "Natal occupants"], rec),
      note("* marks a graha’s emphasised line. Convention: ", v.convention, ".")),
    card(set === "natal" ? "Natal grahas" : "Transit grahas", table(["Graha", "Nakshatra", "Rāśi", "Motion", "Emphasised line", "Nature"], gl)),
    ckHow(note(label("tradition_rule"), " A graha standing in a nakshatra cell throws three lines across the grid: straight ahead to the far side, and the two diagonals. Every cell a line crosses (nakshatras, letters, vowels, rāśis, tithi groups and weekdays) receives its vedha. The cells that matter for a person are the Janma nakshatra, the lagna nakshatra, the Moon and lagna rāśis, the birth tithi group, the birth weekday and the first letter of the name; hover a letter cell to see who pierces it."),
      note(label("interpretation"), " In the Narapati-jayacharya tradition, as modern transit manuals paraphrase it, vedha from benefics on these personal cells is read as support and vedha from malefics as obstruction or loss; several malefics piercing the Janma nakshatra together are read as a difficult period, and a retrograde graha is said to strike harder. This is tradition, not prediction.")),
  ];
}

/* ── Kota ──────────────────────────────────────────────────────────────────────────────────────────────────────── */
function ckKota(v) {
  if (!v) return missing("The Kota chakra", "The Kota chakra needs the Moon.");
  const set = CK_SET, rows = (set === "natal" ? v.natal : v.transit) || [], R = 250, P = 64, W = 2 * (R + P);
  const X = x => W / 2 + x * R, Y = y => W / 2 + y * R;
  const svg = s("svg", { viewBox: `0 0 ${W} ${W}`, class: "ck-svg ck-kota", role: "img", "aria-label": "Kota chakra" });
  ["bahya", "prakara", "madhya", "stambha"].forEach((z, i) => {
    const q = v.zone_size[z];
    const r = s("rect", { x: X(-q), y: Y(-q), width: 2 * q * R, height: 2 * q * R, rx: 4, class: "ck-zone z-" + z, pathLength: 1 }, svg);
    r.style.animationDelay = i * 160 + "ms";
    const t = s("text", { x: X(-q) + 6, y: Y(-q) + 14, class: "ck-zlab" }, svg); t.textContent = z === "bahya" ? "" : CK_ZONE[z].split(" ")[0];
    ckHover(r, () => [CK_ZONE[z], [["Places", v.layout.filter(p => p.zone === z).map(p => p.tr).join(", ")]]]);
  });
  const sideLab = [["East", 0, -1.13], ["South", 1.13, 0], ["West", 0, 1.13], ["North", -1.13, 0]];
  for (const [n, x, y] of sideLab) { const t = s("text", { x: X(x), y: Y(y), class: "ck-side" }, svg); t.textContent = n; }
  for (let gI = 0; gI < 4; gI++) {
    const pts = v.layout.slice(gI * 7, gI * 7 + 7).map(p => `${X(p.x).toFixed(1)},${Y(p.y).toFixed(1)}`).join(" ");
    const pl = s("polyline", { points: pts, class: "ck-path", pathLength: 1 }, svg); pl.style.animationDelay = 500 + gI * 150 + "ms";
  }
  const byPlace = {};
  rows.forEach(r => (byPlace[r.position] = byPlace[r.position] || []).push(r));
  v.layout.forEach((p, i) => {
    const g = s("g", { class: "ck-place" + (p.position === 1 ? " janma" : "") + " z-" + p.zone, "data-position": p.position, tabindex: 0 }, svg);
    g.style.setProperty("--k", i);
    const x = X(p.x), y = Y(p.y);
    s("circle", { cx: x, cy: y, r: 13 }, g);
    const n = s("text", { x, y, class: "ck-num" }, g); n.textContent = p.position;
    const d = s("text", { x, y: y - 20, class: "ck-dev", "font-size": 10 }, g); d.textContent = p.dev;
    const t = s("text", { x, y: y + 22, class: "ck-tr" }, g); t.textContent = CK_AB[p.index];
    (byPlace[p.position] || []).forEach((r, k) => ckMark(svg, r.id, x + 22 + k * 17, y - 4, 8, i % 9 + k, set === "transit"));
    ckHover(g, () => [`${p.position}. ${p.dev} · ${p.tr}`, [["Zone", CK_ZONE[p.zone]], ["Direction", p.direction], ["Place", p.place === "core" ? "the core" : p.place + " the fort"],
      ["Natal here", ckList((v.natal || []).filter(r => r.position === p.position).map(r => r.id))],
      ["Transit here", ckList((v.transit || []).filter(r => r.position === p.position).map(r => `${r.id}`))],
      ...(byPlace[p.position] || []).map(r => [ckEn(r.id), `${r.nature}, ${r.path}${r.retrograde ? " (retrograde)" : ""}`])], `counted from the Janma nakshatra ${v.janma.tr}`]);
  });
  const sum = set === "natal" ? v.natal_summary : v.transit_summary;
  const where = o => o ? `${o.nakshatra} · ${o.zone}, ${o.path}` : "—";
  const tiles = h("div", { class: "stats" },
    stat("Janma nakshatra", `${v.janma.dev} ${v.janma.tr}`, "place 1, top-left corner (east)"),
    stat("Kota swāmī", ckEn(v.kota_swami.graha), `${v.kota_swami.rule} · ${set}: ${where(v.kota_swami[set])}`, GR[v.kota_swami.graha] ? "#/planet/" + v.kota_swami.graha : null),
    stat("Kota pāla", ckEn(v.kota_pala.graha), `${v.kota_pala.rule} (${v.kota_pala.navamsha}) · ${set}: ${where(v.kota_pala[set])}`, GR[v.kota_pala.graha] ? "#/planet/" + v.kota_pala.graha : null),
    stat("Malefics inside", String(sum.malefics_inside.length), ckList(sum.malefics_inside) || "none"),
    stat("Benefics inside", String(sum.benefics_inside.length), ckList(sum.benefics_inside) || "none"));
  const tbl = table(["Graha", "Nakshatra", "Place", "Zone", "Direction", "Moving", "Nature"],
    rows.map(r => [GR[r.id] ? glink(r.id) : ckEn(r.id), r.nakshatra, r.position, CK_ZONE[r.zone].split(" (")[0], r.direction, r.path + (r.retrograde ? " (retrograde)" : ""), r.nature]));
  const list = h("div", { class: "facts-wrap" }, facts([
    ["Malefics entering", ckList(sum.malefics_entering) || "none"], ["Malefics leaving", ckList(sum.malefics_leaving) || "none"],
    ["Benefics entering", ckList(sum.benefics_entering) || "none"], ["Benefics leaving", ckList(sum.benefics_leaving) || "none"],
    ["In the stambha", ckList(sum.in_stambha) || "none"]]));
  return [tiles,
    card("Kota chakra · कोट चक्र", h("div", { class: "figure ck-fig" }, svg),
      note(label("tradition_rule"), " Seven nakshatras per direction from the Janma nakshatra: each direction enters at its corner through bāhya, prakāra and madhya to the stambha, then leaves along its axis. Hover a place for its zone and who is there.")),
    card(set === "natal" ? "Natal grahas in the fort" : "Transit grahas in the fort", tbl, list, note("Convention: ", v.convention, ".")),
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
