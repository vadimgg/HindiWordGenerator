/**
 * .txt fallback download for manual Anki import.
 *
 * Responsible for: generating a tab-separated .txt file in Anki's import format
 * and triggering a browser file download. Used when AnkiConnect is not available.
 *
 * Dependencies: anki/noteType.js, anki/fields/index.js, anki/tagUtils.js.
 */
// Responsible for: generating and triggering browser download of a .txt file for manual Anki import
import { ANKI_NOTE_TYPE, ANKI_FIELDS } from './noteType.js';
import { wordToAnkiFields }            from './fields/index.js';
import { buildWordTags }               from './tagUtils.js';

/**
 * Builds a tab-separated Anki import file and triggers a browser download.
 *
 * The file includes the standard Anki header directives (#separator, #html, etc.)
 * followed by one row per word. Tab characters inside field values are replaced
 * with spaces to avoid breaking the column structure.
 *
 * @param {object[]} words    - Vocabulary word objects to include in the export.
 * @param {string}   deckName - Anki deck name written into the #deck header line.
 * @returns {void}
 */
export function downloadAnkiTxt(words, deckName) {
  const lines = [
    '#separator:tab',
    '#html:true',
    `#notetype:${ANKI_NOTE_TYPE}`,
    `#deck:${deckName}`,
    '#columns:' + [...ANKI_FIELDS, 'Tags'].join('\t'),
  ];

  for (const word of words) {
    const fields  = wordToAnkiFields(word);
    const tagStr  = buildWordTags(word).join(' ');
    const columns = [...ANKI_FIELDS.map(k => (fields[k] || '').replace(/\t/g, ' ')), tagStr];
    lines.push(columns.join('\t'));
  }

  const blob     = new Blob([lines.join('\n')], { type: 'text/plain;charset=utf-8' });
  const url      = URL.createObjectURL(blob);
  const filename = `hindi_vocab_${new Date().toISOString().split('T')[0]}.txt`;
  const anchor   = Object.assign(document.createElement('a'), { href: url, download: filename });
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  setTimeout(() => URL.revokeObjectURL(url), 5000);
}
