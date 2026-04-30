/**
 * Anki Example Sentence field builder.
 *
 * Responsible for: building the Anki ExampleSentence HTML field showing
 * a Hindi sentence with romanisation, English translation, and word breakdown.
 *
 * Dependencies: anki/fields/utils.js.
 */
// Responsible for: building the Anki ExampleSentence HTML field
import { esc, aSection } from './utils.js';

/**
 * Renders a single breakdown token as an HTML row.
 *
 * @param {object} t          - Token object with { hindi, roman, meaning }.
 * @returns {string} HTML div string for the row.
 */
function renderToken(t) {
  return `<div style="display:flex;align-items:baseline;gap:.5rem;padding:.45rem 0;border-bottom:1px solid rgba(51,65,85,.22);">` +
    `<span style="font-family:'Tiro Devanagari Hindi',serif;font-size:1rem;color:#fbbf24;min-width:3.5rem;flex-shrink:0;" lang="hi">${esc(t.hindi)}</span>` +
    `<span style="font-family:'DM Mono',monospace;font-size:.8125rem;color:rgba(94,234,212,.7);min-width:3.5rem;flex-shrink:0;">${esc(t.roman)}</span>` +
    `<span style="font-size:.875rem;color:#64748b;">— ${esc(t.meaning)}</span>` +
    `</div>`;
}

/**
 * Builds the Anki ExampleSentence field HTML for a word.
 *
 * @param {object}  word                    - Vocabulary word object.
 * @param {object}  [word.example_sentence] - Object with { hindi, roman, english, breakdown? }.
 * @returns {string} HTML section, or empty string if no example sentence.
 */
export function buildAnkiExampleSentence(word) {
  const ex = word.example_sentence;
  if (!ex) return '';

  const hindiP = `<p style="font-family:'Tiro Devanagari Hindi',serif;font-size:1.05rem;color:#fbbf24;margin:0 0 .3rem 0;" lang="hi">${esc(ex.hindi)}</p>`;
  const romanP = `<p style="font-family:'DM Mono',monospace;font-size:.875rem;color:#5eead4;margin:0 0 .3rem 0;">${esc(ex.roman)}</p>`;
  const englishP = `<p style="font-size:.9rem;color:#cbd5e1;font-style:italic;margin:0 0 .5rem 0;">${esc(ex.english)}</p>`;

  const breakdown = ex.breakdown || [];
  const breakdownHtml = breakdown.length
    ? `<div style="border-top:1px solid rgba(51,65,85,.3);margin-top:.5rem;">${breakdown.map(renderToken).join('')}</div>`
    : '';

  const content = hindiP + romanP + englishP + breakdownHtml;
  return aSection('Example Sentence', content, '#6366f1');
}
