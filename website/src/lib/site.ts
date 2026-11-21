import stats from "../data/stats.json";

export const site = {
  name: "ESHANA",
  tagline: "From celestial positions to connected meaning.",
  description:
    "ESHANA is a Rust-first computation layer for celestial positions, civil time and symbolic traditions. It keeps measurement, tradition and interpretation apart, with provenance on every result.",
  repo: "https://github.com/Neo-Pragnya/ESHANA",
  docsDir: "docs",
  expansion: stats.expansion,
  version: stats.version,
  eyebrow: "Symbolic computation infrastructure",
  headline: "Compute precisely.",
  highlight: "Interpret transparently.",
  lede: "Positions are calculated. Tradition rules are labelled as rules. Interpretation is never the engine's job. One JSON entry point serves a Rust CLI and a Python package, and every result carries the versions and assumptions that produced it.",
  logoShapes: "circle hex flower star",
  footer: `ESHANA ${stats.version} · Open source, Apache-2.0 · Not a prediction service: calculation is not validation.`,
  footerLinks: [{ label: "GitHub ↗", href: "https://github.com/Neo-Pragnya/ESHANA" }],
} as const;

export const NAV: { label: string; href: string }[] = [
  { label: "Docs", href: "docs/" },
  { label: "Demos", href: "demos/" },
  { label: "Reports", href: "reports/example-birth-chart.html" },
  { label: "Accuracy", href: "docs/guides/accuracy-suite/" },
  { label: "Roadmap", href: "docs/roadmap/" },
  { label: "GitHub ↗", href: "https://github.com/Neo-Pragnya/ESHANA" },
];

/** Documentation sidebar: section → doc ids (paths under docs/ without .md). */
export const DOC_NAV: { section: string; items: string[]; shapes: string; tint: string }[] = [
  { section: "Overview", items: ["index", "roadmap"], shapes: "blob>star", tint: "coral" },
  {
    section: "Concepts",
    items: [
      "concepts/vision",
      "concepts/epistemic-model",
      "concepts/architecture",
      "concepts/calculation-conventions",
      "concepts/provenance",
      "concepts/time",
      "concepts/ephemeris-providers",
      "concepts/privacy-and-ethics",
      "concepts/licensing",
      "concepts/naming",
    ],
    shapes: "circle>flower",
    tint: "teal",
  },
  {
    section: "Guides",
    items: [
      "guides/getting-started",
      "guides/python",
      "guides/vedic",
      "guides/western",
      "guides/human-design",
      "guides/numerology",
      "guides/uncertain-birth-time",
      "guides/method-comparison",
      "guides/graph",
      "guides/spk-kernel",
      "guides/ways-to-use",
      "guides/html-report",
      "guides/vedic-features",
      "guides/panchanga",
      "guides/matching",
      "guides/accuracy-suite",
      "guides/publishing",
    ],
    shapes: "hex>flower8",
    tint: "lemon",
  },
  {
    section: "Reference",
    items: [
      "reference/cli",
      "reference/request",
      "reference/envelope",
      "reference/errors",
      "reference/rulesets",
      "reference/capabilities",
      "reference/accuracy",
      "reference/conformance",
    ],
    shapes: "triangle>circle",
    tint: "sky",
  },
  {
    section: "Decisions",
    items: [
      "adr/index",
      "adr/001-ephemeris-provider-and-licensing",
      "adr/002-time-representation",
      "adr/003-house-systems",
      "adr/004-rule-packaging",
      "adr/005-ontology-persistence",
      "adr/006-api-shape",
      "adr/007-naming-and-scope",
      "adr/008-licensing-model",
      "adr/009-human-design-rights",
      "adr/010-privacy-and-retention",
      "adr/011-open-source-distribution",
      "adr/012-yaml-rule-packs",
      "design/master-plan",
    ],
    shapes: "star>blob",
    tint: "lilac",
  },
];

/** Titles for docs without `title` frontmatter. Every ESHANA page has one; kept for the template. */
export const DOC_TITLES: Record<string, string> = { index: "ESHANA" };

export function docHref(base: string, id: string): string {
  return id === "index" ? `${base}docs/` : `${base}docs/${id}/`;
}
