import assert from 'node:assert/strict';

class Element {
  constructor(id, value = '') {
    this.id = id;
    this.value = value;
    this.textContent = '';
    this.className = 'hidden';
    this.checked = false;
    this.classList = {
      add: () => {},
      remove: className => {
        this.className = this.className.replace(className, '').trim();
      },
    };
  }

  querySelector() {
    return new Element(`${this.id}-child`);
  }
}

const elements = new Map([
  ['export-feedback', new Element('export-feedback')],
  ['export-deck-main', new Element('export-deck-main', 'Hindi')],
  ['export-deck-sub', new Element('export-deck-sub', 'Vocabulary')],
  ['export-sent-deck-main', new Element('export-sent-deck-main', 'Hindi')],
  ['export-sent-deck-sub', new Element('export-sent-deck-sub', 'Sentences01')],
  ['export-override-toggle', new Element('export-override-toggle')],
]);

globalThis.window = {
  __APP_DATA__: {
    allWords: [{
      hindi: 'घर',
      romanisation: 'ghar',
      english: 'house',
    }],
    allSentences: [{
      hindi: 'क्या?',
      romanisation: 'kyā?',
      english: 'What?',
      tokens: [],
    }],
    wordSearchIndex: [{ i: 0 }],
    sentenceSearchIndex: [{ i: 0 }],
    wordGroupTitles: ['Complete Hindi Chapter 01'],
  },
  dispatchEvent() {},
};

globalThis.document = {
  getElementById(id) {
    return elements.get(id) ?? null;
  },
};

globalThis.CustomEvent = class CustomEvent {
  constructor(type) {
    this.type = type;
  }
};

const { initSelection } = await import('../src/scripts/state/selection.js');
const { handleExportClick } = await import('../src/scripts/ui/exportActions.js');

initSelection();
await handleExportClick();

const feedback = elements.get('export-feedback');
assert(feedback.textContent.includes('Review recommended'), 'first export click should warn about selected issues');
assert(feedback.textContent.includes('word audio'), 'warning should mention word audio issues');
assert(feedback.textContent.includes('sentence audio'), 'warning should mention sentence audio issues');
assert(feedback.textContent.includes('sentence token'), 'warning should mention sentence token issues');
assert(!feedback.className.includes('hidden'), 'warning should be visible');

console.log('Export gate checks passed.');
