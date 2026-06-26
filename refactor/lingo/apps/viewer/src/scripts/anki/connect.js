/**
 * AnkiConnect HTTP client.
 *
 * Responsible for: wrapping the AnkiConnect JSON-RPC API (localhost:8765) and
 * providing a connection health check. All Anki API calls in the project go
 * through ankiRequest so error handling is centralised here.
 *
 * No dependencies on other project modules.
 */
// Responsible for: AnkiConnect HTTP API wrapper and connection status check

/**
 * Sends a JSON-RPC request to AnkiConnect and returns the result.
 *
 * Throws if the HTTP response is not OK or if AnkiConnect returns an error
 * in the response body.
 *
 * @param {string} action - AnkiConnect action name (e.g. 'addNotes', 'modelNames').
 * @param {object} params - Action parameters object.
 * @returns {Promise<*>} The value of json.result from the AnkiConnect response.
 * @throws {Error} On network failure, non-2xx HTTP status, or AnkiConnect error.
 */
export async function ankiRequest(action, params) {
  const resp = await fetch('http://localhost:8765', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ action, version: 6, params }),
  });
  if (!resp.ok) throw new Error(`AnkiConnect HTTP ${resp.status}`);
  const json = await resp.json();
  // Only throw for top-level string errors. Array errors (per-note failures from
  // addNotes in newer AnkiConnect versions) are returned as part of the result
  // and handled by the caller.
  if (json.error && typeof json.error === 'string') throw new Error(json.error);
  return json.result;
}

/**
 * Checks whether AnkiConnect is reachable by requesting its version.
 *
 * @returns {Promise<boolean>} True if AnkiConnect responds successfully, false otherwise.
 */
export async function checkAnkiConnect() {
  try { await ankiRequest('version', {}); return true; } catch { return false; }
}
