/**
 * Anki Etymology field builder.
 *
 * Responsible for: building the Anki Etymology HTML section field from
 * the etymology_journey array and optional origin_note string.
 *
 * Dependencies: anki/fields/utils.js.
 */
// Responsible for: building the Anki Etymology HTML field
import { esc, aSection } from './utils.js';

/**
 * Renders a single etymology stage as a centred vertical card block.
 *
 * @param {object}  stage   - Stage object with { stage, form, roman, meaning }.
 * @param {boolean} isLast  - Whether this is the final (current Hindi) stage.
 * @returns {string} HTML string for one .etym-stage block.
 */
function renderStage(stage, isLast) {
  const langStyle = isLast ? ' style="color:#fbbf24;"' : '';
  const wordStyle = isLast
    ? ' style="background:linear-gradient(135deg,#fbbf24,#f97316);-webkit-background-clip:text;-webkit-text-fill-color:transparent;background-clip:text;"'
    : '';
  const meaningPart = stage.meaning
    ? `<span class="etym-sep">—</span><span class="etym-meaning">${esc(stage.meaning)}</span>`
    : '';
  return (
    `<div class="etym-stage">` +
    `<span class="etym-lang"${langStyle}>${esc(stage.stage)}</span>` +
    `<span class="etym-word" lang="hi"${wordStyle}>${esc(stage.form)}</span>` +
    `<div class="etym-sub">` +
    `<span class="etym-rom">${esc(stage.roman)}</span>` +
    meaningPart +
    `</div>` +
    `</div>`
  );
}

/**
 * Builds the Anki Etymology field HTML for a word.
 *
 * Renders a vertical chain of etymology stages with arrow connectors
 * and an optional origin note. Each stage is a centred flex-column card.
 *
 * @param {object}   word                    - Vocabulary word object.
 * @param {object[]} [word.etymology_journey] - Array of { stage, form, roman, meaning } objects.
 * @param {string}   [word.origin_note]       - Optional prose note about the word's origin.
 * @returns {string} HTML section, or empty string if no etymology data.
 */
export function buildAnkiEtymology(word) {
  const journey = word.etymology_journey || [];
  if (!journey.length) return '';

  const stagesHtml = journey.map((stage, i) => {
    const block = renderStage(stage, i === journey.length - 1);
    const arrow = i < journey.length - 1 ? `<div class="etym-arrow">↓</div>` : '';
    return block + arrow;
  }).join('');

  const chain = `<div class="etym-chain">${stagesHtml}</div>`;
  const note = word.origin_note
    ? `<p style="font-size:1rem;color:#64748b;font-style:italic;line-height:1.65;margin-top:.75rem;padding-top:.75rem;border-top:1px solid rgba(30,41,59,.8);">${esc(word.origin_note)}</p>`
    : '';
  return aSection('Etymology', chain + note, '#334155');
}
