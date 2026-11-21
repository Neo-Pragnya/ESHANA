import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Root } from "mdast";
import type { Plugin } from "unified";

// Rewrites relative links between Markdown files in the docs folder (e.g. `../guides/release.md#3-what`) to site
// routes (`/docs/guides/release/#3-what`), so the same Markdown works on GitHub and on the site.
const docsRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../docs");
const siteBase = process.env.SITE_BASE ?? "/";
const basePrefix = siteBase === "/" ? "" : `/${siteBase.replace(/^\/+|\/+$/g, "")}`;

interface LinkNode {
  type: string;
  url?: string;
  children?: unknown[];
}

export function docRoute(filePath: string): string | undefined {
  const relative = path.relative(docsRoot, filePath);
  if (!relative || relative.startsWith("..") || path.isAbsolute(relative)) return undefined;
  if (!relative.toLowerCase().endsWith(".md")) return undefined;
  const route = relative.replace(/\\/g, "/").replace(/(^|\/)index\.md$/i, "$1").replace(/\.md$/i, "");
  return `/docs/${route}${route && !route.endsWith("/") ? "/" : ""}`;
}

const repoRoot = path.resolve(docsRoot, "..");

/** Site URL for a non-Markdown file: docs/** → /docs-assets/**, examples/reports/*.html → /reports/*. */
export function assetRoute(filePath: string): string | undefined {
  const inDocs = path.relative(docsRoot, filePath);
  if (inDocs && !inDocs.startsWith("..") && !path.isAbsolute(inDocs)) return `/docs-assets/${inDocs.replace(/\\/g, "/")}`;
  const inRepo = path.relative(repoRoot, filePath).replace(/\\/g, "/");
  if (inRepo.startsWith("examples/reports/")) return `/reports/${inRepo.slice("examples/reports/".length)}`;
  if (!inRepo.startsWith("..")) return `https://github.com/Neo-Pragnya/ESHANA/blob/main/${inRepo}`;
  return undefined;
}

export const remarkDocLinks: Plugin<[], Root> = () => (tree, file) => {
  const source = file.history[0];
  if (!source || !path.isAbsolute(source) || !docRoute(source)) return;

  const walk = (node: LinkNode) => {
    if (!Array.isArray(node.children)) return;
    for (const child of node.children as LinkNode[]) {
      if (
        (child.type === "link" || child.type === "definition") &&
        typeof child.url === "string" &&
        !/^(?:[a-z][a-z\d+.-]*:|\/|#)/i.test(child.url)
      ) {
        const [pathname, ...fragment] = child.url.split("#");
        const target = path.resolve(path.dirname(source), pathname ?? "");
        const route = docRoute(target);
        const hash = fragment.length ? `#${fragment.join("#")}` : "";
        if (route) child.url = `${basePrefix}${route}${hash}`;
        else {
          const asset = assetRoute(target);
          if (asset) child.url = asset.startsWith("http") ? `${asset}${hash}` : `${basePrefix}${asset}${hash}`;
        }
      } else if (child.type === "image" && typeof child.url === "string" && !/^(?:[a-z][a-z\d+.-]*:|\/|#)/i.test(child.url)) {
        const asset = assetRoute(path.resolve(path.dirname(source), child.url));
        if (asset && !asset.startsWith("http")) child.url = `${basePrefix}${asset}`;
      }
      walk(child);
    }
  };
  walk(tree as unknown as LinkNode);
};
