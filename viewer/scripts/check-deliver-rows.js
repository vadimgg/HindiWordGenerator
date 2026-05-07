import assert from 'node:assert/strict';

import { renderSentenceRow, renderWordRow } from '../src/scripts/ui/deliverRows.js';

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

class Element {
  constructor(tag) {
    this.tag = tag;
    this.className = '';
    this.textContent = '';
    this.children = [];
    this.attrs = {};
    this.style = {};
    this.classList = {
      add: (...classes) => {
        this.className = [this.className, ...classes].filter(Boolean).join(' ');
      },
    };
  }

  setAttribute(key, value) {
    this.attrs[key] = value;
  }

  append(...children) {
    this.children.push(...children);
  }

  get outerHTML() {
    const attrs = [
      this.className ? `class="${escapeHtml(this.className)}"` : '',
      ...Object.entries(this.attrs).map(([key, value]) => `${key}="${escapeHtml(value)}"`),
    ].filter(Boolean).join(' ');
    const open = attrs ? `<${this.tag} ${attrs}>` : `<${this.tag}>`;
    const content = [
      this.textContent ? escapeHtml(this.textContent) : '',
      ...this.children.map(child => typeof child === 'string' ? escapeHtml(child) : child.outerHTML),
    ].join('');
    return `${open}${content}</${this.tag}>`;
  }
}

globalThis.document = {
  createElement(tag) {
    return new Element(tag);
  },
};

const malicious = '<img src=x onerror=alert(1)>';
const wordHtml = renderWordRow({
  hindi: malicious,
  romanisation: 'khiṛkī',
  english: `${malicious}, window`,
  pos: '<script>alert(1)</script>',
}).outerHTML;

assert(!wordHtml.includes('<img'), 'word row must not inject generated HTML');
assert(!wordHtml.includes('<script>'), 'word row must not inject script tags');
assert(wordHtml.includes('&lt;img'), 'word row should render generated markup as text');
assert(wordHtml.includes('no audio'), 'word row should render missing audio badge when audio is absent');

const wordWithAudioHtml = renderWordRow({
  hindi: 'घर',
  romanisation: 'ghar',
  english: 'house',
  pos: 'noun',
}, true).outerHTML;
assert(wordWithAudioHtml.includes('audio'), 'word row should render audio badge when audio is present');

const sentenceHtml = renderSentenceRow({
  hindi: malicious,
  english: malicious,
  register: 'standard',
  tokens: [{ kind: 'word' }],
}, true).outerHTML;

assert(!sentenceHtml.includes('<img'), 'sentence row must not inject generated HTML');
assert(sentenceHtml.includes('&lt;img'), 'sentence row should render generated markup as text');
assert(sentenceHtml.includes('audio'), 'sentence row should render audio badge when audio is present');
assert(sentenceHtml.includes('1 words'), 'sentence row should render token count badge');

console.log('Deliver row safety checks passed.');
