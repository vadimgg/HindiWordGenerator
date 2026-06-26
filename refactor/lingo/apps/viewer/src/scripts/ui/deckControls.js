/**
 * Deliver deck controls — reads deck inputs and keeps preview labels synced.
 */

import { composeDeckName } from '../anki/deckNames.js';

const STORAGE_KEY = 'hindiweb.deliverDecks.v1';
const DEFAULT_DECKS = {
  words: { main: 'Hindi', sub: 'Vocabulary' },
  sentences: { main: 'Hindi', sub: 'Sentences01' },
};

function deckInputs(target) {
  const isSentences = target === 'sentences';
  return {
    main: document.getElementById(isSentences ? 'export-sent-deck-main' : 'export-deck-main'),
    sub: document.getElementById(isSentences ? 'export-sent-deck-sub' : 'export-deck-sub'),
  };
}

function readSavedDecks() {
  try {
    const raw = window.localStorage?.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw);
    return typeof parsed === 'object' && parsed ? parsed : {};
  } catch {
    return {};
  }
}

function saveDecks() {
  try {
    const words = deckInputs('words');
    const sentences = deckInputs('sentences');
    window.localStorage?.setItem(STORAGE_KEY, JSON.stringify({
      words: {
        main: words.main?.value ?? DEFAULT_DECKS.words.main,
        sub: words.sub?.value ?? DEFAULT_DECKS.words.sub,
      },
      sentences: {
        main: sentences.main?.value ?? DEFAULT_DECKS.sentences.main,
        sub: sentences.sub?.value ?? DEFAULT_DECKS.sentences.sub,
      },
    }));
  } catch {
    // Storage is optional; export controls still work without persistence.
  }
}

function restoreDeckInputs() {
  const saved = readSavedDecks();
  for (const target of ['words', 'sentences']) {
    const inputs = deckInputs(target);
    const defaults = DEFAULT_DECKS[target];
    const values = saved[target] ?? defaults;
    if (inputs.main) inputs.main.value = typeof values.main === 'string' ? values.main : defaults.main;
    if (inputs.sub) inputs.sub.value = typeof values.sub === 'string' ? values.sub : defaults.sub;
  }
}

export function getDeckName() {
  const inputs = deckInputs('words');
  const main = inputs.main?.value.trim() || DEFAULT_DECKS.words.main;
  const sub = inputs.sub?.value.trim() || DEFAULT_DECKS.words.sub;
  return composeDeckName(main, sub);
}

export function getSentenceDeckName() {
  const inputs = deckInputs('sentences');
  const main = inputs.main?.value.trim() || DEFAULT_DECKS.sentences.main;
  const sub = inputs.sub?.value.trim() || DEFAULT_DECKS.sentences.sub;
  return composeDeckName(main, sub);
}

export function syncDeckPreview() {
  const name = getDeckName();
  const preview = document.getElementById('export-deck-preview');
  if (preview) preview.textContent = name;

  const confirmDeck = document.getElementById('deliver-confirm-deck');
  if (confirmDeck) confirmDeck.textContent = name;
}

export function syncSentenceDeckPreview() {
  const name = getSentenceDeckName();
  const preview = document.getElementById('export-sent-deck-preview');
  if (preview) preview.textContent = name;

  const confirmDeck = document.getElementById('deliver-confirm-sent-deck');
  if (confirmDeck) confirmDeck.textContent = name;
}

export function syncAllDeckPreviews() {
  syncDeckPreview();
  syncSentenceDeckPreview();
}

function applyDeckPreset(button) {
  const target = button.dataset.deckPreset;
  const main = button.dataset.deckMain ?? 'Hindi';
  const sub = button.dataset.deckSub ?? '';
  const inputs = deckInputs(target);

  if (inputs.main) inputs.main.value = main;
  if (inputs.sub) inputs.sub.value = sub;
  saveDecks();
  syncAllDeckPreviews();
}

function handleDeckInput(syncPreview) {
  syncPreview();
  saveDecks();
}

export function wireDeckControls(onOverrideModeChange) {
  restoreDeckInputs();
  syncAllDeckPreviews();

  document.getElementById('export-deck-main')?.addEventListener('input', () => handleDeckInput(syncDeckPreview));
  document.getElementById('export-deck-sub')?.addEventListener('input', () => handleDeckInput(syncDeckPreview));
  document.getElementById('export-sent-deck-main')?.addEventListener('input', () => handleDeckInput(syncSentenceDeckPreview));
  document.getElementById('export-sent-deck-sub')?.addEventListener('input', () => handleDeckInput(syncSentenceDeckPreview));

  document.querySelectorAll('[data-deck-preset]').forEach(button => {
    button.addEventListener('click', () => applyDeckPreset(button));
  });

  document.getElementById('export-override-toggle')?.addEventListener('change', event => {
    const exportBtn = document.getElementById('export-btn');
    if (exportBtn && !exportBtn.disabled) {
      exportBtn.querySelector('.send-btn-text').textContent =
        event.target.checked ? 'Replace Deck' : 'Send to Anki';
    }
    onOverrideModeChange?.();
  });
}
