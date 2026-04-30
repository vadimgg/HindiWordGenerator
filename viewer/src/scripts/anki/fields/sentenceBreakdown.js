/**
 * Anki field builder — sentence word breakdown table.
 *
 * Responsible for: building the WordBreakdown HTML field for the Hindi Sentence note type.
 *
 * No dependencies on other project modules.
 */
// Responsible for: WordBreakdown Anki field HTML for Hindi Sentence note type

/**
 * Builds the WordBreakdown HTML field for an Anki sentence note.
 * Returns a fully rendered HTML table, or an empty string if no words are provided.
 *
 * @param {object[]|undefined} words - Array of SentenceWord objects with {hindi, roman, meaning, gender?, number?, note?}.
 * @returns {string} HTML string for the WordBreakdown field, or '' if words is empty/missing.
 */
export function buildWordBreakdown(words) {
  if (!words || words.length === 0) return '';

  const rows = words.map((w) => {
    const genderBadge = w.gender === 'feminine'
      ? `<span class="meta-badge" style="background:rgba(131,24,67,.25);color:#f9a8d4;border:1px solid rgba(236,72,153,.25);">f</span>`
      : w.gender === 'masculine'
      ? `<span class="meta-badge" style="background:rgba(30,58,138,.25);color:#93c5fd;border:1px solid rgba(59,130,246,.25);">m</span>`
      : '';

    const numberBadge = w.number
      ? `<span class="meta-badge" style="background:rgba(30,41,59,.6);color:#94a3b8;border:1px solid rgba(51,65,85,.4);">${w.number}</span>`
      : '';

    const badges = (genderBadge || numberBadge)
      ? `<div class="bd-badges">${genderBadge}${numberBadge}</div>`
      : '';

    const note = w.note
      ? `<span class="bd-note">${w.note}</span>`
      : '';

    return `<tr>
      <td style="width:36%;"><span class="bd-word" lang="hi">${w.hindi}</span><span class="bd-roman">${w.roman}</span></td>
      <td><span class="bd-meaning">${w.meaning}</span>${badges}${note}</td>
    </tr>`;
  }).join('');

  return `<table class="breakdown-table">
    <thead><tr><th>Word</th><th>Meaning</th></tr></thead>
    <tbody>${rows}</tbody>
  </table>`;
}
