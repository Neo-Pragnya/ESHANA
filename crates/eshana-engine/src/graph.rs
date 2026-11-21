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
            let label = d["value"]["name"]
                .as_str()
                .or_else(|| d["value"]["rashi"].as_str())
                .or_else(|| d["value"]["aspect"].as_str())
                .unwrap_or(id);
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
        o => Err(EshanaError::Input(format!(
            "unknown graph format '{o}' (json, mermaid, graphml)"
        ))),
    }
}
