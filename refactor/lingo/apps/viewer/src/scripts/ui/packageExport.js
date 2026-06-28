/**
 * Deliver tab — portable package export.
 *
 * Wires the "Export package" action on the Deliver page to the Rust viewer
 * server's POST /api/studio/package endpoint — the same operation as
 * `lingo package --dest <DEST>`. It produces a Grasp-compatible
 * `lingo.package/v1` folder; there is no separate Grasp export format.
 *
 * Only works when the page is served by `lingo viewer` (a static build has no
 * backend); in that case the call surfaces a clear, readable error.
 *
 * Dependencies: studio/api.js. Wired from main.js via initPackageExport().
 */

import { api } from '../studio/api.js';

export function initPackageExport() {
  const btn = document.getElementById('package-export-btn');
  if (!btn) return;
  btn.addEventListener('click', runPackageExport);
  document.getElementById('package-browse-btn')?.addEventListener('click', browseFolder);
  document.getElementById('package-dest')?.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') { event.preventDefault(); runPackageExport(); }
  });
}

async function browseFolder() {
  const input = document.getElementById('package-dest');
  const browseBtn = document.getElementById('package-browse-btn');
  const feedback = document.getElementById('package-export-feedback');
  if (browseBtn) browseBtn.disabled = true;
  try {
    const res = await api.pickFolder();
    if (res?.path && input) {
      input.value = res.path;
      input.focus();
      if (feedback) feedback.classList.add('hidden');
    }
  } catch (error) {
    showFeedback(feedback, `Could not open the folder picker: ${error?.message || error}`, 'error');
  } finally {
    if (browseBtn) browseBtn.disabled = false;
  }
}

async function runPackageExport() {
  const input = document.getElementById('package-dest');
  const btn = document.getElementById('package-export-btn');
  const feedback = document.getElementById('package-export-feedback');
  const dest = (input?.value || '').trim();
  if (!dest) {
    showFeedback(feedback, 'Enter a destination folder.', 'error');
    input?.focus();
    return;
  }

  const original = btn ? btn.textContent : '';
  if (btn) { btn.disabled = true; btn.textContent = 'Exporting…'; }
  showFeedback(feedback, 'Exporting package…', 'running');

  try {
    const res = await api.package({ destination: dest });
    const counts = [];
    if (typeof res.batches === 'number') counts.push(`${res.batches} batch${res.batches === 1 ? '' : 'es'}`);
    if (typeof res.cards === 'number') counts.push(`${res.cards} card${res.cards === 1 ? '' : 's'}`);
    if (typeof res.files === 'number') counts.push(`${res.files} file${res.files === 1 ? '' : 's'}`);
    if (typeof res.bytes === 'number') counts.push(formatBytes(res.bytes));
    const detail = counts.length ? `  ·  ${counts.join(' · ')}` : '';
    showFeedback(feedback, `Package exported → ${res.path || dest}${detail}`, 'ok');
  } catch (error) {
    showFeedback(feedback, `Export failed: ${error?.message || error}`, 'error');
  } finally {
    if (btn) { btn.disabled = false; btn.textContent = original || 'Export package'; }
  }
}

function showFeedback(el, message, kind) {
  if (!el) return;
  el.textContent = message;
  el.className = `package-export-feedback is-${kind}`;
}

function formatBytes(bytes) {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) { value /= 1024; unit += 1; }
  return `${value >= 10 || unit === 0 ? Math.round(value) : value.toFixed(1)} ${units[unit]}`;
}
