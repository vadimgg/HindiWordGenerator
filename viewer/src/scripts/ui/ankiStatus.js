/**
 * Deliver AnkiConnect status — polling and status bar rendering.
 */

import { checkAnkiConnect } from '../anki/connect.js';

let pollInterval = null;

export function updateStatusBadge(online) {
  const bar = document.getElementById('deliver-status-bar');
  const dot = document.getElementById('deliver-status-dot');
  const label = document.getElementById('deliver-status-label');
  const exportBtn = document.getElementById('export-btn');

  if (bar) {
    bar.classList.toggle('anki-status-connected', online);
    bar.classList.toggle('anki-status-offline', !online);
  }
  if (dot) {
    dot.classList.toggle('anki-status-dot-connected', online);
    dot.classList.toggle('anki-status-dot-offline', !online);
  }
  if (label) {
    const overrideToggle = document.getElementById('export-override-toggle');
    const state = online
      ? (overrideToggle?.checked ? 'Connected — Replace mode' : 'Connected')
      : 'Offline — open Anki + AnkiConnect';

    const strong = document.createElement('strong');
    strong.textContent = 'AnkiConnect';
    label.replaceChildren(strong, `  ${state}`);
  }
  if (exportBtn) {
    const overrideToggle = document.getElementById('export-override-toggle');
    exportBtn.disabled = !online;
    exportBtn.querySelector('.send-btn-text').textContent =
      online && overrideToggle?.checked ? 'Replace Deck' : 'Send to Anki';
  }
}

export async function pollAnkiStatus() {
  updateStatusBadge(await checkAnkiConnect());
}

export function startPolling() {
  if (pollInterval) return;
  pollAnkiStatus();
  pollInterval = setInterval(pollAnkiStatus, 3000);
}

export function stopPolling() {
  clearInterval(pollInterval);
  pollInterval = null;
}
