/**
 * Anki Sound Alikes field builder.
 *
 * Responsible for: rendering sound-alike mnemonic entries showing which Hindi
 * syllable sounds like a foreign word, with an optional explanatory note.
 *
 * Dependencies: anki/fields/utils.js.
 */
// Responsible for: building the Anki Sound Alikes HTML field
import { esc, langHex, aSection } from './utils.js';

/**
 * Renders a single sound-alike entry as a centred vertical .clang-item card.
 *
 * @param {object} s - Sound-alike object with { part, association, roman?, language?, note? }.
 * @returns {string} HTML div string for the item.
 */
function renderSoundAlike(s) {
  const col = s.language ? langHex(s.language) : '#94a3b8';

  const langLabel = s.language
    ? `<span class="lang-label" style="color:${col};">${esc(s.language)}</span>`
    : '';

  const soundAlikeRow =
    `<div class="sound-alike-row">` +
    `<span class="sound-alike-part">${esc(s.part || '')}</span>` +
    `<span class="sound-alike-arrow">→</span>` +
    `<span class="sound-alike-assoc">${esc(s.association)}</span>` +
    `</div>`;

  const noteP = s.note
    ? `<p class="sound-alike-note">${esc(s.note)}</p>`
    : '';

  return `<div class="clang-item">` +
    langLabel +
    soundAlikeRow +
    noteP +
    `</div>`;
}

/**
 * Builds the Anki SoundAlikes field HTML for a word.
 *
 * @param {object}   word               - Vocabulary word object.
 * @param {object[]} [word.sound_alikes] - Array of { part, association, roman?, language?, note? }.
 * @returns {string} HTML section, or empty string if no sound-alikes.
 */
export function buildAnkiSoundAlikes(word) {
  const sa = word.sound_alikes || [];
  if (!sa.length) return '';
  const items = sa.map(renderSoundAlike).join('');
  return aSection('Sound Alikes', items, '#e11d48');
}
