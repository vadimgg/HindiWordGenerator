/**
 * Deliver export actions — button handling, Anki export calls, and feedback.
 */

import { overrideDeck, sendSentencesToAnki, sendToAnki } from '../anki/exportService.js';
import { downloadAnkiTxt } from '../anki/txtFallback.js';
import { getDeckName, getSentenceDeckName } from './deckControls.js';
import { getSelectedDeliverItems } from './deliverSelectionPreview.js';

const plural = n => (n !== 1 ? 's' : '');

function showFeedback(message, isError) {
  const el = document.getElementById('export-feedback');
  if (!el) return;
  el.textContent = message;
  el.className = `rounded-xl px-4 py-3 text-sm leading-relaxed ${
    isError
      ? 'bg-red-950/60 border border-red-800/40 text-red-300'
      : 'bg-emerald-950/60 border border-emerald-800/40 text-emerald-300'
  }`;
  el.classList.remove('hidden');
}

function buildSendMessage(added, skipped, deckName) {
  if (added === 0 && skipped > 0) return `All ${skipped} card${plural(skipped)} already exist in "${deckName}".`;
  if (skipped > 0) return `Done! ${added} card${plural(added)} added · ${skipped} already existed.`;
  return `Done! ${added} card${plural(added)} added to "${deckName}".`;
}

function buildSentenceMessage(added, skipped, deckName) {
  if (skipped > 0 && added === 0) return `Sentences: all ${skipped} already exist in "${deckName}".`;
  if (skipped > 0) return `Sentences: ${added} added · ${skipped} already existed in "${deckName}".`;
  return `Sentences: ${added} card${plural(added)} added to "${deckName}".`;
}

export async function handleExportClick(onComplete) {
  const { words, sentences } = getSelectedDeliverItems();
  const deckName = getDeckName();
  const sentDeckName = getSentenceDeckName();
  const isOverride = document.getElementById('export-override-toggle')?.checked ?? false;

  if (!words.length && !sentences.length) {
    showFeedback('No words or sentences selected.', true);
    return;
  }

  const exportBtn = document.getElementById('export-btn');
  if (exportBtn) {
    exportBtn.disabled = true;
    exportBtn.querySelector('.send-btn-text').textContent = isOverride ? 'Replacing…' : 'Sending…';
  }
  document.getElementById('export-feedback')?.classList.add('hidden');

  try {
    const messages = [];

    if (words.length > 0) {
      if (isOverride) {
        const { added, deleted } = await overrideDeck(words, deckName);
        messages.push(`Words: removed ${deleted} old, added ${added} new to "${deckName}".`);
      } else {
        const { added, skipped } = await sendToAnki(words, deckName);
        messages.push(buildSendMessage(added, skipped, deckName));
      }
    }

    if (sentences.length > 0) {
      const { added, skipped } = await sendSentencesToAnki(sentences, sentDeckName);
      messages.push(buildSentenceMessage(added, skipped, sentDeckName));
    }

    showFeedback(messages.join(' '), false);
  } catch (err) {
    showFeedback(`Error: ${err.message}`, true);
  } finally {
    if (exportBtn) exportBtn.querySelector('.send-btn-text').textContent = 'Send to Anki';
    onComplete?.();
  }
}

export function handleTxtDownload() {
  const { words } = getSelectedDeliverItems();
  downloadAnkiTxt(words, getDeckName());
  showFeedback('Downloading .txt file for manual Anki import…', false);
}

export function wireExportActions(onExportComplete) {
  document.getElementById('export-btn')?.addEventListener('click', () => handleExportClick(onExportComplete));
  document.getElementById('export-txt-btn')?.addEventListener('click', handleTxtDownload);
}
