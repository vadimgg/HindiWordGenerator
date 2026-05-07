/**
 * Deliver export actions — button handling, Anki export calls, and feedback.
 */

import { overrideDeck, sendSentencesToAnki, sendToAnki } from '../anki/exportService.js';
import { downloadAnkiTxt } from '../anki/txtFallback.js';
import { getSentenceTokenIssue } from '../quality/sentenceTokens.js';
import { resolveSentenceAudioSrc, resolveWordAudioSrc } from '../../utils/audioHelpers.ts';
import { getDeckName, getSentenceDeckName } from './deckControls.js';
import { getSelectedDeliverItems } from './deliverSelectionPreview.js';

const plural = n => (n !== 1 ? 's' : '');
let pendingIssueOverride = '';

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

function selectionSignature(words, sentences, deckName, sentDeckName, isOverride) {
  return JSON.stringify({
    words: words.map(word => [word.hindi, word.romanisation, word.english]),
    sentences: sentences.map(sentence => [sentence.hindi, sentence.romanisation, sentence.english]),
    deckName,
    sentDeckName,
    isOverride,
  });
}

function selectedIssueSummary(words, sentences) {
  const wordAudio = words.filter(word => !resolveWordAudioSrc(word)).length;
  const sentenceAudio = sentences.filter(sentence => !resolveSentenceAudioSrc(sentence)).length;
  const sentenceTokens = sentences.filter(sentence => getSentenceTokenIssue(sentence)).length;
  const total = wordAudio + sentenceAudio + sentenceTokens;
  return { total, wordAudio, sentenceAudio, sentenceTokens };
}

function buildIssueWarning(summary) {
  const parts = [
    summary.wordAudio ? `${summary.wordAudio} word audio` : '',
    summary.sentenceAudio ? `${summary.sentenceAudio} sentence audio` : '',
    summary.sentenceTokens ? `${summary.sentenceTokens} sentence token` : '',
  ].filter(Boolean);
  return `Review recommended: selected cards have ${parts.join(', ')} issue${plural(summary.total)}. Click Send to Anki again to export anyway.`;
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

  const signature = selectionSignature(words, sentences, deckName, sentDeckName, isOverride);
  const issueSummary = selectedIssueSummary(words, sentences);
  if (issueSummary.total > 0 && pendingIssueOverride !== signature) {
    pendingIssueOverride = signature;
    showFeedback(buildIssueWarning(issueSummary), true);
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
    pendingIssueOverride = '';
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
