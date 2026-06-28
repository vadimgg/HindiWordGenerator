/**
 * Studio API client.
 *
 * Responsible for: the single fetch boundary between the Studio UI and the Rust
 * viewer server's `/api/studio/*` endpoints (see crates/lingo-cli/src/studio.rs).
 * Every call returns parsed JSON or throws an `ApiError` carrying the server's
 * HTTP status, human message, and (for 422 validation failures) the `problems`
 * list the packet-exchange panel renders inline.
 *
 * No dependencies on other project modules.
 */
// Responsible for: typed fetch wrapper over the local /api/studio endpoints

const BASE = '/api/studio';

/** Error thrown for any non-2xx response. */
export class ApiError extends Error {
  constructor(message, status, problems) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.problems = problems || [];
  }
}

async function request(method, path, body) {
  let res;
  try {
    res = await fetch(BASE + path, {
      method,
      headers: body !== undefined ? { 'Content-Type': 'application/json' } : undefined,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
  } catch (networkError) {
    throw new ApiError(`could not reach the viewer server: ${networkError.message}`, 0, []);
  }
  let data = {};
  try { data = await res.json(); } catch { /* empty / non-JSON body */ }
  if (!res.ok) {
    throw new ApiError(data.message || `HTTP ${res.status}`, res.status, data.problems || []);
  }
  return data;
}

export const api = {
  status:        ()        => request('GET',  '/status'),
  batch:         (id)      => request('GET',  `/batch?batch=${encodeURIComponent(id)}`),
  listRaw:       ()        => request('GET',  '/raw'),
  rawDetail:     (id)      => request('GET',  `/raw?raw=${encodeURIComponent(id)}`),
  saveRaw:       (payload) => request('POST', '/raw', payload),
  updateMeta:    (payload) => request('POST', '/meta', payload),
  prepareImport: (payload) => request('POST', '/import/prepare', payload),
  applyImport:   (payload) => request('POST', '/import/apply', payload),
  prepareBuild:  (payload) => request('POST', '/build/prepare', payload),
  applyBuild:    (payload) => request('POST', '/build/apply', payload),
  check:         (payload) => request('POST', '/check', payload),
  voices:        ()        => request('GET',  '/voices'),
  audio:         (payload) => request('POST', '/audio', payload),
  package:       (payload) => request('POST', '/package', payload),
  pickFolder:    ()        => request('POST', '/package/browse', {}),
  getConfig:     ()        => request('GET',  '/config'),
  setConfig:     (payload) => request('POST', '/config', payload),
};
