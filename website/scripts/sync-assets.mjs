// Copies non-Markdown files from docs/ (diagrams, the interactive architecture page, draw.io files) to
// public/docs-assets/, and the example HTML reports to public/reports/, so the same relative links work on GitHub
// and on the site. remark-doc-links.ts rewrites the links. Runs before `astro dev` and `astro build`.
import { cpSync, existsSync, mkdirSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../..");
const pub = path.resolve(here, "../public");

function copyTree(src, dst, keep) {
  for (const name of readdirSync(src)) {
    const s = path.join(src, name);
    const d = path.join(dst, name);
    if (statSync(s).isDirectory()) copyTree(s, d, keep);
    else if (keep(name)) {
      mkdirSync(path.dirname(d), { recursive: true });
      cpSync(s, d);
    }
  }
}

copyTree(path.join(repo, "docs"), path.join(pub, "docs-assets"), (n) => !n.toLowerCase().endsWith(".md"));
const reports = path.join(repo, "examples", "reports");
if (existsSync(reports)) copyTree(reports, path.join(pub, "reports"), (n) => n.endsWith(".html"));
const gallery = path.join(repo, "examples", "gallery");
if (existsSync(gallery)) copyTree(gallery, path.join(pub, "gallery"), (n) => n.endsWith(".webp"));
console.log("synced docs assets and example reports");
