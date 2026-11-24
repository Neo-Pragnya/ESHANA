/** An engine error, with the stable `ESH-*` code of the error envelope. */
export declare class EshanaError extends Error {
  readonly code: string;
  constructor(code: string, message: string);
}

/** A request: `{ op: "...", ... }`, as in docs/reference/request.md. */
export type Request = { op: string; [field: string]: unknown };

/** A result envelope: `{ schema_version, kind, ... }`, as in schemas/envelope.schema.json. */
export type Envelope = { schema_version: string; kind: string; [field: string]: unknown };

export interface Engine {
  /** Run a request; throws EshanaError for an error envelope. */
  execute(request: Request): Envelope;
  /** Run a request given as JSON text; returns JSON text (an error envelope on failure, never a throw). */
  executeJson(request: string): string;
  /** The engine version. */
  readonly version: string | null;
}

/** Load the WebAssembly engine (Node: the .wasm next to this module; browsers: fetched next to it). */
export declare function load(source?: string | URL | Response | ArrayBuffer | Uint8Array | WebAssembly.Module): Promise<Engine>;
