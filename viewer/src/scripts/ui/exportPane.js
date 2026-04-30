/**
 * Export pane controller — selected-word list, sentence list, AnkiConnect status polling, export actions.
 *
 * Responsible for: keeping the Deliver page's UI in sync with the current word
 * and sentence selection, polling AnkiConnect every 3 seconds while the Deliver
 * tab is open, handling the "Send to Anki" and "Replace Deck" button actions, and
 * triggering the .txt fallback download.
 *
 * Does NOT perform the actual Anki API calls (anki/export.js) or selection
 * state management (state/selection.js) — it coordinates them.
 *
 * Dependencies: anki/connect.js, anki/export.js (sendToAnki, overrideDeck,
 *               ensureSentenceNoteType, sentenceToAnkiFields, uploadSentenceAudio),
 *               anki/txtFallback.js, state/selection.js, data.js.
 */
// Responsible for: deliver page controller — word list, sentence list, AnkiConnect polling, export button

import { checkAnkiConnect, ankiRequest }                  from '../anki/connect.js';
import { sendToAnki, overrideDeck, ensureSentenceNoteType, sentenceToAnkiFields, uploadSentenceAudio } from '../anki/export.js';
import { downloadAnkiTxt }                               from '../anki/txtFallback.js';
import { getSelectedWordObjects, getSelectedSentenceIndices } from '../state/selection.js';
import { getAllSentences }                                from '../data.js';

/** @type {number|null} setInterval handle; non-null while polling is active. */
let pollInterval = null;

/** Returns 's' when n !== 1 — used for pluralising feedback messages. */
const plural = n => (n !== 1 ? 's' : '');

/**
 * Returns the current words deck name from the deck inputs (main::sub).
 *
 * @returns {string} Deck name in the form 'Main::Sub'.
 */
function getDeckName() {
  const main = document.getElementById('export-deck-main')?.value.trim() || 'Hindi';
  const sub  = document.getElementById('export-deck-sub')?.value.trim() || 'Vocabulary';
  return sub ? `${main}::${sub}` : main;
}

/**
 * Returns the current sentences deck name from the sentences deck inputs (main::sub).
 *
 * @returns {string} Deck name in the form 'Main::Sub'.
 */
function getSentenceDeckName() {
  const main = document.getElementById('export-sent-deck-main')?.value.trim() || 'Hindi';
  const sub  = document.getElementById('export-sent-deck-sub')?.value.trim() || 'Sentences01';
  return sub ? `${main}::${sub}` : main;
}

/**
 * Syncs the words deck preview text and confirm-deck label to the current input values.
 *
 * @returns {void}
 */
function syncDeckPreview() {
  const name = getDeckName();
  const preview = document.getElementById('export-deck-preview');
  if (preview) preview.textContent = name;
  const confirmDeck = document.getElementById('deliver-confirm-deck');
  if (confirmDeck) confirmDeck.textContent = name;
}

/**
 * Syncs the sentences deck preview text and confirm-deck label to the current input values.
 *
 * @returns {void}
 */
function syncSentenceDeckPreview() {
  const name = getSentenceDeckName();
  const preview = document.getElementById('export-sent-deck-preview');
  if (preview) preview.textContent = name;
  const confirmDeck = document.getElementById('deliver-confirm-sent-deck');
  if (confirmDeck) confirmDeck.textContent = name;
}

/**
 * Updates the AnkiConnect status bar and export button state.
 *
 * @param {boolean} online - Whether AnkiConnect is reachable.
 * @returns {void}
 */
function updateStatusBadge(online) {
  const bar       = document.getElementById('deliver-status-bar');
  const dot       = document.getElementById('deliver-status-dot');
  const label     = document.getElementById('deliver-status-label');
  const exportBtn = document.getElementById('export-btn');

  if (bar) {
    bar.classList.toggle('anki-status-connected', online);
    bar.classList.toggle('anki-status-offline',   !online);
  }
  if (dot) {
    dot.classList.toggle('anki-status-dot-connected', online);
    dot.classList.toggle('anki-status-dot-offline',   !online);
  }
  if (label) {
    const overrideToggle = document.getElementById('export-override-toggle');
    const state = online
      ? (overrideToggle?.checked ? 'Connected — Replace mode' : 'Connected')
      : 'Offline — open Anki + AnkiConnect';
    label.innerHTML = `<strong>AnkiConnect</strong> &nbsp;${state}`;
  }
  if (exportBtn) {
    const overrideToggle = document.getElementById('export-override-toggle');
    exportBtn.disabled = !online;
    exportBtn.querySelector('.send-btn-text').textContent =
      online && overrideToggle?.checked ? 'Replace Deck' : 'Send to Anki';
  }
}

/**
 * Checks AnkiConnect status and updates the badge with the result.
 *
 * @returns {Promise<void>}
 */
async function pollAnkiStatus() {
  updateStatusBadge(await checkAnkiConnect());
}

/**
 * Starts polling AnkiConnect every 3 seconds. No-op if already polling.
 *
 * @returns {void}
 */
function startPolling() {
  if (pollInterval) return;
  pollAnkiStatus();
  pollInterval = setInterval(pollAnkiStatus, 3000);
}

/**
 * Stops the AnkiConnect poll interval.
 *
 * @returns {void}
 */
function stopPolling() {
  clearInterval(pollInterval);
  pollInterval = null;
}

/**
 * Rebuilds the selected-word list in the deliver panel.
 * Shows one compact row per selected word.
 *
 * @returns {void}
 */
function populateWordTable() {
  const words        = getSelectedWordObjects();
  const sentIndices  = getSelectedSentenceIndices();
  const allSents     = getAllSentences();
  const sentences    = sentIndices.map(i => allSents[i]).filter(Boolean);

  const wordRows     = document.getElementById('export-word-rows');
  const sentRows     = document.getElementById('export-sent-rows');
  const wordCountEl  = document.getElementById('export-word-count');
  const sentCountEl  = document.getElementById('export-sent-count');
  const metaEl       = document.getElementById('deliver-action-meta');
  const confirmWordCount = document.getElementById('deliver-confirm-word-count');
  const confirmSentLine  = document.getElementById('deliver-confirm-sent-line');
  const confirmSentCount = document.getElementById('deliver-confirm-sent-count');
  const deliverMain  = document.getElementById('deliver-main');
  const deliverEmpty = document.getElementById('deliver-empty');

  // Update counts
  if (wordCountEl)  wordCountEl.textContent  = String(words.length);
  if (sentCountEl)  sentCountEl.textContent  = String(sentences.length);
  if (confirmWordCount) confirmWordCount.textContent = `${words.length} word card${plural(words.length)}`;
  if (confirmSentCount) confirmSentCount.textContent = `${sentences.length} sentence card${plural(sentences.length)}`;

  // Show/hide sentences confirm line
  if (confirmSentLine) confirmSentLine.style.display = sentences.length > 0 ? '' : 'none';

  // Update meta line
  const total = words.length + sentences.length;
  if (metaEl) {
    if (total === 0) {
      metaEl.textContent = 'Select words or sentences to begin';
    } else if (words.length > 0 && sentences.length > 0) {
      metaEl.innerHTML = `<strong>${total} cards</strong> across 2 decks`;
    } else {
      metaEl.textContent = `${total} card${plural(total)} ready to export`;
    }
  }

  // Show/hide empty state
  if (deliverMain && deliverEmpty) {
    const isEmpty = total === 0;
    deliverMain.style.display  = isEmpty ? 'none' : '';
    deliverEmpty.style.display = isEmpty ? ''     : 'none';
  }

  syncDeckPreview();
  syncSentenceDeckPreview();

  // Populate word rows
  if (wordRows) {
    wordRows.innerHTML = words.map(w => `
      <div class="deliver-row">
        <span class="deliver-row-hindi" lang="hi">${w.hindi || ''}</span>
        <span class="deliver-row-sep">·</span>
        <span class="deliver-row-roman">${w.romanisation || ''}</span>
        <span class="deliver-row-sep">·</span>
        <span class="deliver-row-english">${(w.english || '').split(',')[0].trim()}</span>
        <span class="deliver-row-meta">${[w.pos, w.gender ? (w.gender === 'masculine' ? 'masc.' : 'fem.') : ''].filter(Boolean).join(' · ')}</span>
      </div>`).join('');
  }

  // Populate sentence rows
  if (sentRows) {
    sentRows.innerHTML = sentences.map(s => {
      const badge = s.register
        ? `<span class="deliver-row-badge"><span class="reg reg-${s.register}" style="font-size:11px;padding:0.1rem 0.45rem;">${s.register}</span></span>`
        : '';
      return `
        <div class="deliver-row">
          <span class="deliver-row-hindi" lang="hi">${s.hindi || ''}</span>
          <span class="deliver-row-sep">·</span>
          <span class="deliver-row-english">${s.english || ''}</span>
          ${badge}
        </div>`;
    }).join('');
  }
}

/**
 * Displays a success or error message below the export button.
 *
 * @param {string}  message - The message to display.
 * @param {boolean} isError - When true, renders in red; when false, in green.
 * @returns {void}
 */
function showFeedback(message, isError) {
  const el = document.getElementById('export-feedback');
  if (!el) return;
  el.textContent = message;
  el.className   = `rounded-xl px-4 py-3 text-sm leading-relaxed ${
    isError
      ? 'bg-red-950/60 border border-red-800/40 text-red-300'
      : 'bg-emerald-950/60 border border-emerald-800/40 text-emerald-300'
  }`;
  el.classList.remove('hidden');
}

/**
 * Builds the success feedback message for an incremental Anki export.
 *
 * @param {number} added   - Number of cards actually added.
 * @param {number} skipped - Number of cards skipped (already exist).
 * @param {string} deckName - Target deck name for display in the message.
 * @returns {string} Human-readable result message.
 */
function buildSendMessage(added, skipped, deckName) {
  if (added === 0 && skipped > 0) return `All ${skipped} card${plural(skipped)} already exist in "${deckName}".`;
  if (skipped > 0) return `Done! ${added} card${plural(added)} added · ${skipped} already existed.`;
  return `Done! ${added} card${plural(added)} added to "${deckName}".`;
}

/**
 * Exports selected sentences to Anki incrementally (skipping duplicates).
 *
 * @param {object[]} sentences - Sentence objects to export.
 * @param {string}   deckName  - Target Anki deck name.
 * @returns {Promise<{ added: number, skipped: number }>}
 */
async function sendSentencesToAnki(sentences, deckName) {
  await ankiRequest('createDeck', { deck: deckName });
  await ensureSentenceNoteType();
  // Upload audio files before adding notes (failures are silently ignored)
  await Promise.all(sentences.map(s => uploadSentenceAudio(s)));
  const notes  = sentences.map(s => ({
    deckName,
    modelName: 'Hindi Sentence',
    fields:    sentenceToAnkiFields(s, s.chapter ?? ''),
    tags:      s.anki_tags ?? [],
    options:   { allowDuplicate: false, duplicateScope: 'deck' },
  }));
  const canAdd  = await ankiRequest('canAddNotes', { notes });
  const toAdd   = notes.filter((_, i) => canAdd[i]);
  const skipped = notes.length - toAdd.length;
  let added = 0;
  if (toAdd.length > 0) {
    const results = await ankiRequest('addNotes', { notes: toAdd });
    added = Array.isArray(results) ? results.filter(r => typeof r === 'number').length : 0;
  }
  return { added, skipped };
}

/**
 * Handles the export button click: validates, runs the export/override API call
 * for both words and sentences, and shows the result feedback.
 *
 * @returns {Promise<void>}
 */
async function handleExportClick() {
  const words        = getSelectedWordObjects();
  const sentIndices  = getSelectedSentenceIndices();
  const allSents     = getAllSentences();
  const sentences    = sentIndices.map(i => allSents[i]).filter(Boolean);
  const deckName     = getDeckName();
  const sentDeckName = getSentenceDeckName();
  const isOverride   = document.getElementById('export-override-toggle')?.checked ?? false;

  if (!words.length && !sentences.length) { showFeedback('No words or sentences selected.', true); return; }

  const exportBtn = document.getElementById('export-btn');
  if (exportBtn) {
    exportBtn.disabled = true;
    exportBtn.querySelector('.send-btn-text').textContent = isOverride ? 'Replacing…' : 'Sending…';
  }
  document.getElementById('export-feedback')?.classList.add('hidden');

  try {
    const messages = [];

    // Export words
    if (words.length > 0) {
      if (isOverride) {
        const { added, deleted } = await overrideDeck(words, deckName);
        messages.push(`Words: removed ${deleted} old, added ${added} new to "${deckName}".`);
      } else {
        const { added, skipped } = await sendToAnki(words, deckName);
        messages.push(buildSendMessage(added, skipped, deckName));
      }
    }

    // Export sentences
    if (sentences.length > 0) {
      const { added, skipped } = await sendSentencesToAnki(sentences, sentDeckName);
      if (skipped > 0 && added === 0) {
        messages.push(`Sentences: all ${skipped} already exist in "${sentDeckName}".`);
      } else if (skipped > 0) {
        messages.push(`Sentences: ${added} added · ${skipped} already existed in "${sentDeckName}".`);
      } else {
        messages.push(`Sentences: ${added} card${plural(added)} added to "${sentDeckName}".`);
      }
    }

    showFeedback(messages.join(' '), false);
  } catch (err) {
    showFeedback(`Error: ${err.message}`, true);
  } finally {
    if (exportBtn) exportBtn.querySelector('.send-btn-text').textContent = 'Send to Anki';
    pollAnkiStatus();
  }
}

/**
 * Wires window-level event listeners for tab changes and selection changes.
 *
 * @returns {void}
 */
function wireWindowListeners() {
  window.addEventListener('tabchange', e => {
    if (e.detail.tab === 'deliver') { populateWordTable(); startPolling(); }
    else stopPolling();
  });

  window.addEventListener('selectionchange', () => {
    const deliverPage = document.getElementById('page-deliver');
    if (!deliverPage?.classList.contains('is-active')) return;
    populateWordTable();
    syncDeckPreview();
  });
}

/**
 * Wires DOM element listeners for the deck inputs, override toggle, export button,
 * and .txt download button.
 *
 * @returns {void}
 */
function wireControlListeners() {
  document.getElementById('export-deck-main')?.addEventListener('input', syncDeckPreview);
  document.getElementById('export-deck-sub')?.addEventListener('input', syncDeckPreview);
  document.getElementById('export-sent-deck-main')?.addEventListener('input', syncSentenceDeckPreview);
  document.getElementById('export-sent-deck-sub')?.addEventListener('input', syncSentenceDeckPreview);

  document.getElementById('export-override-toggle')?.addEventListener('change', e => {
    const exportBtn = document.getElementById('export-btn');
    if (exportBtn && !exportBtn.disabled) {
      exportBtn.querySelector('.send-btn-text').textContent =
        e.target.checked ? 'Replace Deck' : 'Send to Anki';
    }
    // Update status label to reflect override mode
    pollAnkiStatus();
  });

  document.getElementById('export-btn')?.addEventListener('click', handleExportClick);

  document.getElementById('export-txt-btn')?.addEventListener('click', () => {
    const words    = getSelectedWordObjects();
    const deckName = getDeckName();
    downloadAnkiTxt(words, deckName);
    showFeedback('Downloading .txt file for manual Anki import…', false);
  });
}

/**
 * Initialises the export/deliver pane.
 *
 * @returns {void}
 */
export function initExportPane() {
  wireWindowListeners();
  wireControlListeners();
}
