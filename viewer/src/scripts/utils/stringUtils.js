/**
 * Shared string utilities for diacritic-insensitive matching and script parsing.
 *
 * Responsible for: norm() for diacritic-stripping, and extractDevanagari() /
 * extractPartLabel() for parsing sound-alike part strings.
 *
 * Used by: ui/search.js, ui/tooltip.js, anki/fields/soundAlikes.js.
 * No dependencies on other project modules.
 */
// Responsible for: shared string utilities — norm(), extractDevanagari(), extractPartLabel()

/**
 * Strips diacritical marks and lowercases a string for diacritic-insensitive comparison.
 *
 * Decomposes the string to NFD (so ā → a + combining macron), strips the combining
 * characters (U+0300–U+036F), then lowercases and trims whitespace.
 *
 * @param {string} str - Input string, may contain diacritics such as ā, ī, ṭ, ṇ, etc.
 * @returns {string} Normalised lowercase string with all diacritics removed.
 *
 * @example
 * norm('Baṛā') // → 'bara'
 * norm('adhyāpak') // → 'adhyapak'
 */
export function norm(str) {
  return str.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase().trim();
}

/**
 * Extracts the Devanagari text from a sound-alike `part` string.
 * Returns the text before the first space or '(', e.g. 'घर (full word)' → 'घर'.
 *
 * @param {string} part - Part string from a SoundAlike object.
 * @returns {string} Devanagari portion, or the full string if no delimiter found.
 */
export function extractDevanagari(part) {
  const idx = part.search(/[\s(]/);
  return idx === -1 ? part : part.slice(0, idx).trim();
}

/**
 * Extracts the parenthetical label from a sound-alike `part` string.
 * E.g. 'घर (full word)' → 'full word'. Returns '' if no parenthetical found.
 *
 * @param {string} part - Part string from a SoundAlike object.
 * @returns {string} Content inside parentheses, or empty string.
 */
export function extractPartLabel(part) {
  const m = part.match(/\(([^)]+)\)/);
  return m ? m[1] : '';
}
