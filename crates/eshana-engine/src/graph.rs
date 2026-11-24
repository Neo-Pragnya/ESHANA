//! Typed symbolic graph built from a result envelope, with exports to JSON, Mermaid and GraphML.
//!
//! Nodes are inputs, positions, derived attributes, concepts, rules and sources. Edges carry a **relation
//! class**, so causal, correlational, traditional-correspondence, hypothesised and purely computational links are
//! never put in one unlabelled field. Everything generated here is computational or a traditional
//! correspondence; the engine never asserts a causal link between the sky and a person.

use eshana_core::{EshanaError, Result, SCHEMA_VERSION};
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// Edge vocabulary.
pub const EDGE_KINDS: [&str; 8] = [
    "calculated_from",
    "classified_as",
    "uses_rule",
    "associated_with",
    "contrasts_with",
    "derived_from",
    "interpreted_as",
    "cites",
];

/// Relation classes. `causal` and `correlational` exist for researchers' own graphs; the engine emits neither.
pub const RELATION_CLASSES: [&str; 5] = [
    "computational",
    "traditional_correspondence",
    "correlational",
    "causal",
    "hypothesized",
];

fn source_for(rule: &str) -> Option<(&'static str, &'static str)> {
    let r = rule.split('@').next().unwrap_or(rule);
    Some(match r {
        "vedic.parashari" => ("source:parashari", "Parashari conventions as widely published (Brihat Parashara Hora Shastra tradition); implemented independently from standard tables"),
        "western.basic" => ("source:ptolemy", "Ptolemaic aspects and the tropical zodiac; orbs are a tradition-level choice"),
        "human_design.basic" => ("source:hd-structure", "Human Design wheel, channel and centre structure as publicly described; legal review pending (ADR-009)"),
        "numerology.pythagorean" => ("source:pythagorean-table", "Pythagorean letter table as commonly published"),
        "numerology.chaldean" => ("source:chaldean-table", "Chaldean letter table as commonly published"),
        r if r.starts_with("ayanamsha-lahiri") => ("source:crc-1956", "Indian Calendar Reform Committee (1956); anchor value from the Swiss Ephemeris reference"),
        r if r.starts_with("ayanamsha-") => ("source:ayanamsha-ref", "Published ayanamsha anchor value; carried by IAU 2006 precession"),
        _ => return None,
    })
}

struct G {
    nodes: Vec<Value>,
    edges: Vec<Value>,
    seen: BTreeSet<String>,
}

impl G {
    fn node(&mut self, id: &str, kind: &str, label: &str, epistemic: &str, extra: Value) {
        if self.seen.insert(id.to_string()) {
            self.nodes
                .push(json!({"id": id, "type": kind, "label": label, "epistemic": epistemic, "data": extra}));
        }
    }
    fn edge(&mut self, from: &str, to: &str, kind: &str, class: &str, epistemic: &str, rule: Option<&str>) {
        debug_assert!(EDGE_KINDS.contains(&kind) && RELATION_CLASSES.contains(&class));
        self.edges.push(json!({"from": from, "to": to, "kind": kind, "relation_class": class, "epistemic": epistemic, "rule_version": rule}));
    }
}

/// Build the graph from a chart envelope.
#[must_use]
pub fn build(env: &Value) -> Value {
    let mut g = G {
        nodes: vec![],
        edges: vec![],
        seen: BTreeSet::new(),
    };
    g.node(
        "input:moment",
        "input",
        env["input"]["instant_utc"].as_str().unwrap_or("input"),
        "calculated",
        env["input"].clone(),
    );
    if let Some(ps) = env["positions"].as_array() {
        for p in ps {
            let id = p["id"].as_str().unwrap_or("?");
            let label = format!(
                "{} {} {}",
                id,
                p["sign"]["dms"].as_str().unwrap_or(""),
                p["sign"]["name"].as_str().unwrap_or("")
            );
            g.node(
                &format!("pos:{id}"),
                "position",
                label.trim(),
                "calculated",
                json!({"longitude_deg": p["longitude_deg"]}),
            );
            g.edge(
                &format!("pos:{id}"),
                "input:moment",
                "calculated_from",
                "computational",
                "calculated",
                None,
            );
        }
    }
    if let Some(ds) = env["derived_attributes"].as_array() {
        for d in ds {
            let id = d["id"].as_str().unwrap_or("?");
            let rule = d["rule_version"].as_str().unwrap_or("");
            let ep = d["epistemic"].as_str().unwrap_or("tradition_rule");
            // a name, a sign or an aspect; a bhāva by its number; otherwise the id alone
            let house = d["value"]["house"].as_u64().map(|h| format!("house {h}"));
            let label = d["value"]["name"]
                .as_str()
                .or_else(|| d["value"]["rashi"].as_str())
                .or_else(|| d["value"]["aspect"].as_str())
                .map(str::to_string)
                .or(house)
                .unwrap_or_else(|| id.to_string());
            g.node(
                &format!("attr:{id}"),
                "derived_attribute",
                &format!("{id} = {label}"),
                ep,
                d["value"].clone(),
            );
            if !rule.is_empty() {
                g.node(
                    &format!("rule:{rule}"),
                    "rule",
                    rule,
                    "tradition_rule",
                    json!({"tradition_id": d["tradition_id"]}),
                );
                g.edge(
                    &format!("attr:{id}"),
                    &format!("rule:{rule}"),
                    "uses_rule",
                    "computational",
                    ep,
                    Some(rule),
                );
                if let Some((sid, text)) = source_for(rule) {
                    g.node(sid, "source", text, "tradition_rule", Value::Null);
                    g.edge(
                        &format!("rule:{rule}"),
                        sid,
                        "cites",
                        "computational",
                        "tradition_rule",
                        Some(rule),
                    );
                }
            }
            for f in d["derived_from"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                let target = if g.seen.contains(&format!("pos:{f}")) {
                    format!("pos:{f}")
                } else {
                    "input:moment".to_string()
                };
                g.edge(
                    &format!("attr:{id}"),
                    &target,
                    "derived_from",
                    "computational",
                    ep,
                    Some(rule),
                );
            }
            if let Some(name) = d["value"]["name"]
                .as_str()
                .or_else(|| d["value"]["rashi"].as_str())
            {
                let cid = format!(
                    "concept:{}:{}",
                    d["tradition_id"].as_str().unwrap_or(""),
                    name.to_lowercase().replace(' ', "-")
                );
                g.node(&cid, "concept", name, "tradition_rule", Value::Null);
                g.edge(
                    &format!("attr:{id}"),
                    &cid,
                    "classified_as",
                    "traditional_correspondence",
                    ep,
                    Some(rule),
                );
            }
        }
    }
    json!({
        "schema_version": SCHEMA_VERSION,
        "kind": "symbolic_graph",
        "nodes": g.nodes,
        "edges": g.edges,
        "relation_classes": RELATION_CLASSES,
        "note": "No edge asserts a causal link between celestial positions and a person.",
    })
}

fn esc_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn esc_mermaid(s: &str) -> String {
    s.replace('"', "'")
        .replace(['[', ']', '(', ')', '{', '}', '|', '<', '>'], " ")
}

/// A deterministic layout: one column per node type (in order of first appearance), nodes stacked in each column.
/// Returns (x, y) per node index, and the canvas width and height.
fn layout(nodes: &[Value]) -> (Vec<(f64, f64)>, f64, f64) {
    let (col_w, row_h, w, h0) = (280.0, 46.0, 240.0, 60.0);
    let mut types: Vec<String> = vec![];
    let mut rows: Vec<usize> = vec![];
    let mut pos = vec![];
    for n in nodes {
        let t = n["type"].as_str().unwrap_or("").to_string();
        let c = types.iter().position(|x| *x == t).unwrap_or_else(|| {
            types.push(t);
            rows.push(0);
            types.len() - 1
        });
        pos.push((20.0 + c as f64 * col_w, h0 + rows[c] as f64 * row_h));
        rows[c] += 1;
    }
    let width = 40.0 + types.len() as f64 * col_w - (col_w - w);
    let height = h0 + rows.iter().copied().max().unwrap_or(1) as f64 * row_h + 20.0;
    (pos, width, height)
}

/// A label that fits a 240-px box at 12 px (about 36 characters), with an ellipsis; the full text is in the tooltip.
fn short(label: &str) -> String {
    if label.chars().count() <= 36 {
        label.to_string()
    } else {
        label.chars().take(35).collect::<String>() + "…"
    }
}

/// Fill colour and border of a node by its epistemic layer (as in the Mermaid export).
fn colours(epistemic: &str) -> (&'static str, &'static str) {
    match epistemic {
        "tradition_rule" => ("#fff1d6", "#b07a12"),
        "speculative_interpretation" => ("#f6dcdc", "#aa3333"),
        _ => ("#dff3e3", "#2f7d4a"),
    }
}

/// The graph as a CHITKALA living-graph document (GRA-03): its fixture schema (`metadata`, `universes`, `nodes`,
/// `edges`, `sources`; see CHITKALA's `Ideation/sample-graph.json`). Each node type is a universe placed on a ring;
/// calculated nodes are tagged `verified`, tradition rules `traditional`, the rest `context`; every edge is in the
/// `astrology` lane, approved when calculated and `proposed` when it is a traditional correspondence. Positions are
/// deterministic, so the same chart always draws the same way.
fn chitkala(g: &Value) -> Value {
    let nodes = g["nodes"].as_array().cloned().unwrap_or_default();
    let mut types: Vec<String> = vec![];
    for n in &nodes {
        let t = n["type"].as_str().unwrap_or("").to_string();
        if !types.contains(&t) {
            types.push(t);
        }
    }
    let centre = |k: usize| {
        let a = k as f64 / types.len().max(1) as f64 * std::f64::consts::TAU;
        [
            (300.0 * a.cos()).round(),
            (220.0 * a.sin()).round(),
            (40.0 * (2.0 * a).sin()).round(),
        ]
    };
    let universes: Vec<Value> = types
        .iter()
        .enumerate()
        .map(|(k, t)| {
            json!({"id": t, "label": t.replace('_', " "), "symbol": "✦",
        "desc": format!("ESHANA {} nodes", t.replace('_', " ")), "center": centre(k)})
        })
        .collect();
    let mut count: Vec<usize> = vec![0; types.len()];
    let out_nodes: Vec<Value> = nodes.iter().map(|n| {
        let t = n["type"].as_str().unwrap_or("");
        let k = types.iter().position(|x| x == t).unwrap_or(0);
        let i = count[k];
        count[k] += 1;
        let c = centre(k);
        let (s, q) = ((i as f64 * 2.399).sin(), (i as f64 * 2.399).cos()); // the golden angle spreads them evenly
        let r = 18.0 + 9.0 * (i as f64).sqrt();
        let tag = match n["epistemic"].as_str().unwrap_or("") {
            "calculated" => "verified",
            "tradition_rule" => "traditional",
            _ => "context",
        };
        json!({"id": n["id"], "label": n["label"], "cluster": t, "summary": format!("{} ({})", n["label"].as_str().unwrap_or(""), n["epistemic"].as_str().unwrap_or("")),
               "tag": tag, "x": (c[0] + r * s).round(), "y": (c[1] + r * q).round(), "z": c[2]})
    }).collect();
    let edges: Vec<Value> = g["edges"].as_array().into_iter().flatten().enumerate().map(|(i, e)| {
        let traditional = e["relation_class"] == "traditional_correspondence";
        json!({"id": format!("edge{i}"), "from": e["from"], "to": e["to"], "label": e["kind"],
               "status": if traditional { "proposed" } else { "approved" }, "epistemic_lane": "astrology",
               "review_status": if traditional { "candidate" } else { "engine_computed" },
               "claim_id": format!("claim:eshana:edge{i}"), "evidence_source_ids": ["source:eshana"],
               "evidence_grade": if traditional { "tradition_rule" } else { "computed" }, "rule_version": e["rule_version"]})
    }).collect();
    json!({
        "metadata": {"name": "ESHANA chart graph", "demoOnly": false, "schemaStatus": "chitkala_fixture_schema",
                     "epistemicNotice": "Calculated positions and named tradition rules; no edge is a causal or predictive claim",
                     "proposedEdgesNotEvidence": true, "sourceAcknowledgements": ["ESHANA engine"]},
        "universes": universes, "nodes": out_nodes, "edges": edges,
        "sources": [{"id": "source:eshana", "title": "ESHANA symbolic graph", "uri": "https://neo-pragnya.github.io/ESHANA/", "role": "computed chart and named rules"}],
    })
}

/// Export a chart envelope's graph in a given format.
///
/// # Errors
/// [`EshanaError::Input`] for an unknown format.
pub fn export(env: &Value, format: &str) -> Result<Value> {
    let g = build(env);
    match format {
        "json" => Ok(g),
        "mermaid" => {
            let nodes = g["nodes"].as_array().cloned().unwrap_or_default();
            let idx = |id: &str| nodes.iter().position(|n| n["id"] == id).unwrap_or(0);
            let mut s = String::from("flowchart LR\n");
            for (i, n) in nodes.iter().enumerate() {
                s.push_str(&format!(
                    "  n{i}[\"{}\"]:::{}\n",
                    esc_mermaid(n["label"].as_str().unwrap_or("")),
                    n["epistemic"].as_str().unwrap_or("calculated")
                ));
            }
            for e in g["edges"].as_array().into_iter().flatten() {
                let arrow = if e["relation_class"] == "traditional_correspondence" {
                    "-.->"
                } else {
                    "-->"
                };
                s.push_str(&format!(
                    "  n{} {arrow}|{}| n{}\n",
                    idx(e["from"].as_str().unwrap_or("")),
                    e["kind"].as_str().unwrap_or(""),
                    idx(e["to"].as_str().unwrap_or(""))
                ));
            }
            s.push_str("  classDef calculated fill:#dff3e3,stroke:#2f7d4a;\n  classDef tradition_rule fill:#fff1d6,stroke:#b07a12;\n  classDef speculative_interpretation fill:#f6dcdc,stroke:#a33;\n");
            Ok(
                json!({"schema_version": SCHEMA_VERSION, "kind": "symbolic_graph_export", "format": "mermaid", "content": s}),
            )
        }
        "graphml" => {
            let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n");
            s.push_str("  <key id=\"label\" for=\"node\" attr.name=\"label\" attr.type=\"string\"/>\n  <key id=\"type\" for=\"node\" attr.name=\"type\" attr.type=\"string\"/>\n  <key id=\"epistemic\" for=\"node\" attr.name=\"epistemic\" attr.type=\"string\"/>\n");
            s.push_str("  <key id=\"kind\" for=\"edge\" attr.name=\"kind\" attr.type=\"string\"/>\n  <key id=\"class\" for=\"edge\" attr.name=\"relation_class\" attr.type=\"string\"/>\n  <graph id=\"eshana\" edgedefault=\"directed\">\n");
            for n in g["nodes"].as_array().into_iter().flatten() {
                s.push_str(&format!("    <node id=\"{}\"><data key=\"label\">{}</data><data key=\"type\">{}</data><data key=\"epistemic\">{}</data></node>\n",
                    esc_xml(n["id"].as_str().unwrap_or("")), esc_xml(n["label"].as_str().unwrap_or("")), esc_xml(n["type"].as_str().unwrap_or("")), esc_xml(n["epistemic"].as_str().unwrap_or(""))));
            }
            for (i, e) in g["edges"].as_array().into_iter().flatten().enumerate() {
                s.push_str(&format!("    <edge id=\"e{i}\" source=\"{}\" target=\"{}\"><data key=\"kind\">{}</data><data key=\"class\">{}</data></edge>\n",
                    esc_xml(e["from"].as_str().unwrap_or("")), esc_xml(e["to"].as_str().unwrap_or("")), esc_xml(e["kind"].as_str().unwrap_or("")), esc_xml(e["relation_class"].as_str().unwrap_or(""))));
            }
            s.push_str("  </graph>\n</graphml>\n");
            Ok(
                json!({"schema_version": SCHEMA_VERSION, "kind": "symbolic_graph_export", "format": "graphml", "content": s}),
            )
        }
        "svg" => {
            let nodes = g["nodes"].as_array().cloned().unwrap_or_default();
            let (pos, width, height) = layout(&nodes);
            let idx = |id: &str| nodes.iter().position(|n| n["id"] == id);
            let mut s = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.0} {height:.0}\" width=\"{width:.0}\" height=\"{height:.0}\" role=\"img\" aria-label=\"ESHANA symbolic graph\">\n<title>ESHANA symbolic graph</title>\n<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n<g fill=\"none\" stroke-width=\"1.2\">\n");
            for e in g["edges"].as_array().into_iter().flatten() {
                let (Some(a), Some(b)) = (
                    idx(e["from"].as_str().unwrap_or("")),
                    idx(e["to"].as_str().unwrap_or("")),
                ) else {
                    continue;
                };
                let ((x1, y1), (x2, y2)) = ((pos[a].0 + 240.0, pos[a].1 + 15.0), (pos[b].0, pos[b].1 + 15.0));
                let dash = if e["relation_class"] == "traditional_correspondence" {
                    " stroke-dasharray=\"5 4\""
                } else {
                    ""
                };
                s.push_str(&format!("  <path d=\"M{x1:.0} {y1:.0} C{:.0} {y1:.0} {:.0} {y2:.0} {x2:.0} {y2:.0}\" stroke=\"#8a8fa8\"{dash}><title>{}</title></path>\n",
                    (x1 + x2) / 2.0, (x1 + x2) / 2.0, esc_xml(e["kind"].as_str().unwrap_or(""))));
            }
            s.push_str("</g>\n<g font-family=\"system-ui, sans-serif\" font-size=\"12\">\n");
            for (i, n) in nodes.iter().enumerate() {
                let (fill, stroke) = colours(n["epistemic"].as_str().unwrap_or(""));
                let (x, y) = pos[i];
                s.push_str(&format!("  <g><rect x=\"{x:.0}\" y=\"{y:.0}\" width=\"240\" height=\"30\" rx=\"6\" fill=\"{fill}\" stroke=\"{stroke}\"/><text x=\"{:.0}\" y=\"{:.0}\" fill=\"#1d1f2a\">{}</text><title>{} ({})</title></g>\n",
                    x + 8.0, y + 19.0, esc_xml(&short(n["label"].as_str().unwrap_or(""))), esc_xml(n["label"].as_str().unwrap_or("")), esc_xml(n["type"].as_str().unwrap_or(""))));
            }
            s.push_str("</g>\n</svg>\n");
            Ok(
                json!({"schema_version": SCHEMA_VERSION, "kind": "symbolic_graph_export", "format": "svg", "content": s}),
            )
        }
        "drawio" => {
            let nodes = g["nodes"].as_array().cloned().unwrap_or_default();
            let (pos, _, _) = layout(&nodes);
            let idx = |id: &str| nodes.iter().position(|n| n["id"] == id);
            let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<mxfile host=\"ESHANA\">\n  <diagram name=\"ESHANA symbolic graph\" id=\"eshana\">\n    <mxGraphModel grid=\"1\" gridSize=\"10\">\n      <root>\n        <mxCell id=\"0\"/>\n        <mxCell id=\"1\" parent=\"0\"/>\n");
            for (i, n) in nodes.iter().enumerate() {
                let (fill, stroke) = colours(n["epistemic"].as_str().unwrap_or(""));
                let (x, y) = pos[i];
                s.push_str(&format!("        <mxCell id=\"n{i}\" value=\"{}\" tooltip=\"{} ({})\" style=\"rounded=1;whiteSpace=wrap;fillColor={fill};strokeColor={stroke};\" vertex=\"1\" parent=\"1\">\n          <mxGeometry x=\"{x:.0}\" y=\"{y:.0}\" width=\"240\" height=\"30\" as=\"geometry\"/>\n        </mxCell>\n",
                    esc_xml(n["label"].as_str().unwrap_or("")), esc_xml(n["id"].as_str().unwrap_or("")), esc_xml(n["type"].as_str().unwrap_or(""))));
            }
            for (k, e) in g["edges"].as_array().into_iter().flatten().enumerate() {
                let (Some(a), Some(b)) = (
                    idx(e["from"].as_str().unwrap_or("")),
                    idx(e["to"].as_str().unwrap_or("")),
                ) else {
                    continue;
                };
                let dash = if e["relation_class"] == "traditional_correspondence" {
                    "dashed=1;"
                } else {
                    ""
                };
                s.push_str(&format!("        <mxCell id=\"e{k}\" value=\"{}\" style=\"endArrow=block;{dash}\" edge=\"1\" parent=\"1\" source=\"n{a}\" target=\"n{b}\">\n          <mxGeometry relative=\"1\" as=\"geometry\"/>\n        </mxCell>\n",
                    esc_xml(e["kind"].as_str().unwrap_or(""))));
            }
            s.push_str("      </root>\n    </mxGraphModel>\n  </diagram>\n</mxfile>\n");
            Ok(
                json!({"schema_version": SCHEMA_VERSION, "kind": "symbolic_graph_export", "format": "drawio", "content": s}),
            )
        }
        "chitkala" => Ok(
            json!({"schema_version": SCHEMA_VERSION, "kind": "symbolic_graph_export", "format": "chitkala",
                                "content": serde_json::to_string_pretty(&chitkala(&g)).expect("json")}),
        ),
        o => Err(EshanaError::Input(format!(
            "unknown graph format '{o}' (json, mermaid, graphml, svg, drawio, chitkala)"
        ))),
    }
}
