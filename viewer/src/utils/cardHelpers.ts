/**
 * Pure helper functions shared across WordCard section Astro components.
 *
 * Responsible for: date formatting, CSS class lookups for form chips,
 * category badges, register badges, and per-language text colours used
 * in the website's card UI.
 *
 * These are build-time utilities (used in Astro component frontmatter).
 * They have no runtime dependencies and no side effects.
 *
 * No dependencies on other project modules.
 */
// Responsible for: pure helper functions shared across WordCard section components

/**
 * Formats an ISO date string (YYYY-MM-DD) as a human-readable label.
 * @example formatDate('2026-03-28') → 'Mar 28, 2026'
 */
export function formatDate(iso: string): string {
  const months = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
  const [y, m, d] = iso.split('-');
  return `${months[parseInt(m) - 1]} ${parseInt(d)}, ${y}`;
}

/** Capitalises the first character of a string. */
export const cap = (s: string) => s ? s.charAt(0).toUpperCase() + s.slice(1) : '';

/** Returns the Tailwind CSS class string for a form chip based on its label. */
export function chipClass(label: string): string {
  const l = label.toLowerCase();
  if (l.includes('feminine'))   return 'form-chip form-chip-fem';
  if (l.includes('masculine'))  return 'form-chip form-chip-masc';
  if (l.includes('oblique'))    return 'form-chip form-chip-indigo';
  if (l.includes('invariable')) return 'form-chip form-chip-teal';
  if (l.includes('adverb'))     return 'form-chip form-chip-sky';
  if (l.includes('adjective'))  return 'form-chip form-chip-amber';
  if (l.includes('noun'))       return 'form-chip form-chip-violet';
  if (l.includes('number') || l.includes('cardinal') || l.includes('article')) return 'form-chip form-chip-emerald';
  if (l.includes('compound'))   return 'form-chip form-chip-rose';
  return 'form-chip';
}

/**
 * Returns the Tailwind CSS class string for a word-category badge.
 *
 * @param {string} pos - The part-of-speech string (e.g. "noun", "verb").
 * @returns {string} Tailwind class string for the badge.
 */
export function categoryStyle(pos: string): string {
  const c = pos.toLowerCase();
  if (c.includes('conjunction'))  return 'bg-emerald-900/30 text-emerald-400 border border-emerald-700/40';
  if (c.includes('adverb'))       return 'bg-sky-900/30 text-sky-400 border border-sky-700/40';
  if (c.includes('noun'))         return 'bg-violet-900/30 text-violet-400 border border-violet-700/40';
  if (c.includes('adjective'))    return 'bg-amber-900/30 text-amber-500 border border-amber-700/40';
  if (c.includes('verb'))         return 'bg-orange-900/30 text-orange-400 border border-orange-700/40';
  if (c.includes('preposition') || c.includes('postposition')) return 'bg-rose-900/30 text-rose-400 border border-rose-700/40';
  if (c.includes('particle'))     return 'bg-teal-900/30 text-teal-400 border border-teal-700/40';
  if (c.includes('pronoun'))      return 'bg-indigo-900/30 text-indigo-400 border border-indigo-700/40';
  return 'bg-slate-800/60 text-slate-400 border border-slate-600/40';
}

/**
 * Returns the Tailwind CSS class string for a sentence register badge.
 * @param register - Register string from a Sentence object.
 * @returns Tailwind class string for the badge.
 */
export function registerStyle(register: string): string {
  const map: Record<string, string> = {
    formal:     'bg-violet-900/30 text-violet-400 border border-violet-700/40',
    neutral:    'bg-cyan-900/30 text-cyan-300 border border-cyan-700/40',
    standard:   'bg-sky-900/30 text-sky-400 border border-sky-700/40',
    casual:     'bg-amber-500/20 text-amber-400 border border-amber-500/30',
    colloquial: 'bg-rose-900/30 text-rose-400 border border-rose-700/40',
  };
  return map[register] ?? 'bg-slate-800/60 text-slate-400 border border-slate-700/40';
}

/**
 * Extracts the Devanagari text from a `part` string like "लड़ (laṛ-)".
 * Returns everything before the first space or opening parenthesis.
 *
 * The equivalent for the browser-side Anki pipeline lives in stringUtils.js.
 * These cannot share an import: cardHelpers.ts is build-time TS; stringUtils.js
 * is browser JS. The different names prevent arch-script false-positive flags.
 *
 * @param part - Free-form part string from a SoundAlike entry.
 * @returns The Devanagari portion of the string.
 */
export function devanagariFromPart(part: string): string {
  const idx = part.search(/[\s(]/);
  return idx === -1 ? part : part.slice(0, idx).trim();
}

/**
 * Extracts the parenthetical label from a `part` string like "घर (full word)".
 * Returns the content inside the first pair of parentheses, or an empty string.
 *
 * @param part - Free-form part string from a SoundAlike entry.
 * @returns The label inside parentheses, or '' if none found.
 */
export function partLabel(part: string): string {
  const m = part.match(/\(([^)]+)\)/);
  return m ? m[1] : '';
}

/** Returns a Tailwind text-colour class for a given language name. */
export function langColor(lang: string): string {
  const map: Record<string, string> = {
    'Proto-Indo-European': 'text-teal-200/70', 'PIE': 'text-teal-200/70',
    'Sanskrit': 'text-amber-300/80', 'Prakrit': 'text-amber-200/65', 'Apabhraṃśa': 'text-amber-200/65',
    'Hindi': 'text-amber-400/80', 'Urdu': 'text-orange-400/80', 'Punjabi': 'text-yellow-400/75',
    'Arabic': 'text-orange-400/75', 'Persian': 'text-pink-400/70',
    'English': 'text-sky-400/75', 'Russian': 'text-cyan-300/75', 'Hebrew': 'text-emerald-300/75',
    'Latin': 'text-violet-300/70', 'Greek': 'text-indigo-400/70',
    'French': 'text-violet-400/70', 'German': 'text-slate-400/70',
    'Italian': 'text-green-300/70', 'Spanish': 'text-red-300/70',
  };
  return map[lang] ?? 'text-slate-400/70';
}
