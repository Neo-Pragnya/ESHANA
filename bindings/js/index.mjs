// ESHANA for JavaScript: the engine as a WebAssembly module, in browsers and in Node (BIND-03, BIND-04; ADR-014).
// One entry point, as in every binding: a JSON request in, a JSON envelope out. No rule lives here.
//
//   import { load } from "eshana";
//   const eshana = await load();                       // Node: reads eshana_engine.wasm next to this file
//   const chart = eshana.execute({ op: "chart", system: "vedic", when: {...}, place: {...}, config: {...} });
//
// In a browser, `load()` fetches the .wasm next to this module; pass a URL, a Response, bytes or a compiled
// WebAssembly.Module to load it from elsewhere. Nothing is sent anywhere: the module has no imports.

/** An engine error, with the stable `ESH-*` code of the error envelope. */
export class EshanaError extends Error {
  constructor(code, message) {
    super(`${code}: ${message}`);
    this.name = "EshanaError";
    this.code = code;
  }
}

async function bytesOf(source) {
  if (source instanceof WebAssembly.Module) return source;
  if (source instanceof ArrayBuffer || ArrayBuffer.isView(source)) return source;
  if (typeof Response !== "undefined" && source instanceof Response) return await source.arrayBuffer();
  const url = source ?? new URL("./eshana_engine.wasm", import.meta.url);
  const isNode = typeof process !== "undefined" && process.versions && process.versions.node;
  if (isNode && (url instanceof URL ? url.protocol === "file:" : !/^https?:/.test(String(url)))) {
    const { readFile } = await import("node:fs/promises");
    return await readFile(url instanceof URL ? url : new URL(String(url), "file://" + process.cwd() + "/"));
  }
  const r = await fetch(url);
  if (!r.ok) throw new Error(`cannot load the ESHANA engine from ${url}: HTTP ${r.status}`);
  return await r.arrayBuffer();
}

/**
 * Load the engine. Resolves to `{ execute, executeJson, version }`.
 * @param {string|URL|Response|ArrayBuffer|Uint8Array|WebAssembly.Module} [source]
 */
export async function load(source) {
  const bytes = await bytesOf(source);
  const instance = bytes instanceof WebAssembly.Module
    ? await WebAssembly.instantiate(bytes, {})
    : (await WebAssembly.instantiate(bytes, {})).instance;
  const x = instance.exports, enc = new TextEncoder(), dec = new TextDecoder();

  /** Run a request given as JSON text; returns the envelope as JSON text (an error envelope on failure). */
  function executeJson(text) {
    const b = enc.encode(text);
    const p = x.eshana_alloc(b.length);
    new Uint8Array(x.memory.buffer, p, b.length).set(b);
    const out = x.eshana_execute(p, b.length);
    const n = new DataView(x.memory.buffer).getUint32(out, true);
    const s = dec.decode(new Uint8Array(x.memory.buffer, out + 4, n).slice());
    x.eshana_free(out, 4 + n);
    x.eshana_free(p, b.length);
    return s;
  }

  /** Run a request object; returns the envelope, or throws an EshanaError with the envelope's code. */
  function execute(request) {
    const env = JSON.parse(executeJson(JSON.stringify(request)));
    if (env.kind === "error") throw new EshanaError(env.error.code, env.error.message);
    return env;
  }

  const info = execute({ op: "info" });
  return { execute, executeJson, version: info.engine_version ?? info.versions?.engine ?? null };
}
