import assert from 'node:assert/strict';

const elements = new Map();
const listeners = new Map();
const storage = new Map();

function input(id, value = '') {
  const element = {
    id,
    value,
    textContent: '',
    addEventListener(type, handler) {
      listeners.set(`${id}:${type}`, handler);
    },
  };
  elements.set(id, element);
  return element;
}

function text(id) {
  const element = { id, textContent: '' };
  elements.set(id, element);
  return element;
}

input('export-deck-main', 'Hindi');
input('export-deck-sub', 'Vocabulary');
input('export-sent-deck-main', 'Hindi');
input('export-sent-deck-sub', 'Sentences01');
text('export-deck-preview');
text('export-sent-deck-preview');
text('deliver-confirm-deck');
text('deliver-confirm-sent-deck');

const presetButton = {
  dataset: {
    deckPreset: 'words',
    deckMain: 'Hindi',
    deckSub: 'Review',
  },
  addEventListener(type, handler) {
    listeners.set(`preset:${type}`, handler);
  },
};

globalThis.document = {
  getElementById(id) {
    return elements.get(id) ?? null;
  },
  querySelectorAll(selector) {
    return selector === '[data-deck-preset]' ? [presetButton] : [];
  },
};

globalThis.window = {
  localStorage: {
    getItem(key) {
      return storage.get(key) ?? null;
    },
    setItem(key, value) {
      storage.set(key, value);
    },
  },
};

const { getDeckName, getSentenceDeckName, wireDeckControls } = await import('../src/scripts/ui/deckControls.js');

wireDeckControls();
assert.equal(getDeckName(), 'Hindi::Vocabulary');
assert.equal(getSentenceDeckName(), 'Hindi::Sentences01');

elements.get('export-deck-sub').value = 'Custom Words';
listeners.get('export-deck-sub:input')();
assert.equal(getDeckName(), 'Hindi::Custom Words');
assert.equal(JSON.parse(storage.get('hindiweb.deliverDecks.v1')).words.sub, 'Custom Words');

listeners.get('preset:click')();
assert.equal(getDeckName(), 'Hindi::Review');
assert.equal(elements.get('export-deck-preview').textContent, 'Hindi::Review');
assert.equal(elements.get('deliver-confirm-deck').textContent, 'Hindi::Review');
assert.equal(JSON.parse(storage.get('hindiweb.deliverDecks.v1')).words.sub, 'Review');

console.log('Deck control persistence checks passed.');
