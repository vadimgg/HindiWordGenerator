/**
 * Rich text highlighting for vocabulary card content.
 *
 * Responsible for: wrapping recognised patterns in text with HTML span elements
 * that carry CSS classes for visual highlighting — Devanagari script, Cyrillic,
 * Hebrew, language names, country adjectives, grammatical gender labels, and
 * romanised terms in single quotes.
 *
 * Also exports parseCrossLangLabel() which parses legacy string-format
 * cross-language entries (before they were migrated to structured objects).
 *
 * No dependencies on other project modules.
 */
// Responsible for: rich text highlighting for vocabulary card text content

const LANGUAGE_NAMES = [
  'Proto-Indo-European', 'Sanskrit', 'English', 'Russian', 'Hebrew',
  'Hindi', 'Urdu', 'Arabic', 'Persian', 'Latin', 'Greek', 'Prakrit',
  'Apabhraṃśa', 'PIE', 'French', 'German', 'Italian', 'Spanish',
];

const LANG_COLOR_CLASS: Record<string, string> = {
  'Proto-Indo-European': 'hl-lang-pie',
  'PIE':                 'hl-lang-pie',
  'Sanskrit':            'hl-lang-sanskrit',
  'Prakrit':             'hl-lang-prakrit',
  'Apabhraṃśa':          'hl-lang-prakrit',
  'Hindi':               'hl-lang-hindi',
  'Urdu':                'hl-lang-urdu',
  'Arabic':              'hl-lang-arabic',
  'Persian':             'hl-lang-persian',
  'English':             'hl-lang-english',
  'Russian':             'hl-lang-russian',
  'Hebrew':              'hl-lang-hebrew',
  'Latin':               'hl-lang-latin',
  'Greek':               'hl-lang-greek',
  'French':              'hl-lang-french',
  'German':              'hl-lang-german',
  'Italian':             'hl-lang-italian',
  'Spanish':             'hl-lang-spanish',
};

const COUNTRY_NAMES = [
  'American', 'Indian', 'Gangetic', 'Iranian', 'Pakistani',
  'Afghan', 'Israeli', 'British', 'European', 'Indic',
];

// Diacritics found in romanised Hindi/Sanskrit
const ROMANISED_RE = /'([^']*[āīūṭḍṇśṣṃḥṛṝḷ√][^']*)'/g;

const LANG_RE    = new RegExp(`\\b(${LANGUAGE_NAMES.join('|')})\\b`, 'g');
const COUNTRY_RE = new RegExp(`\\b(${COUNTRY_NAMES.join('|')})\\b`, 'g');

// Matches "masculine", "masculine singular/plural/oblique", and feminine variants
const MASC_RE = /\b(masculine(?:\s+(?:singular|plural|oblique|pl\.))?)\b/gi;
const FEM_RE  = /\b(feminine(?:\s+(?:singular|plural|oblique|pl\.))?)\b/gi;

/**
 * Applies inline HTML highlighting to a plain-text vocabulary description.
 *
 * Processes in order: HTML-escape → Devanagari → Cyrillic → Hebrew →
 * language names → country adjectives → gender labels → romanised terms.
 *
 * @param text - Raw text content from a vocabulary field.
 * @returns HTML string with recognised patterns wrapped in styled spans.
 */
export function highlight(text: string): string {
  // 1. HTML-escape first
  let h = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  // 2. Non-Latin scripts
  h = h.replace(/[\u0900-\u097F]+/g,
    '<span class="hl-deva">$&</span>');
  h = h.replace(/[\u0400-\u04FF]+/g,
    '<span class="hl-cyrillic">$&</span>');
  h = h.replace(/[\u0590-\u05FF\uFB1D-\uFB4E]+/g,
    '<span class="hl-hebrew">$&</span>');

  // 3. Linguistic labels (language names and country adjectives)
  h = h.replace(LANG_RE, (_, lang) => {
    const cls = LANG_COLOR_CLASS[lang] ?? 'hl-lang';
    return `<span class="${cls}">${lang}</span>`;
  });
  h = h.replace(COUNTRY_RE, '<span class="hl-country">$1</span>');

  // 4. Grammatical gender labels
  h = h.replace(MASC_RE, '<span class="hl-masc">$1</span>');
  h = h.replace(FEM_RE,  '<span class="hl-fem">$1</span>');

  // 5. Romanised terms in single quotes containing diacritics
  h = h.replace(ROMANISED_RE, `<span class="hl-roman">'$1'</span>`);

  return h;
}

/**
 * Parses a legacy string-format cross-language entry into a label + text pair.
 *
 * Accepts both colon and em-dash separators:
 *   "TRUE RELATIVE: …"  or  "TRUE RELATIVE — …"
 *
 * @param item - Raw cross-language entry string.
 * @returns Object with a label ('TRUE RELATIVE' | 'USEFUL COINCIDENCE' | null)
 *          and the remaining text after the separator.
 */
export function parseCrossLangLabel(item: string): {
  label: 'TRUE RELATIVE' | 'USEFUL COINCIDENCE' | null;
  text: string;
} {
  // Accept both "TRUE RELATIVE: text" and "TRUE RELATIVE — text" separators
  const match = item.match(/^(TRUE RELATIVE|USEFUL COINCIDENCE)(?::\s*|\s*—\s*)/);
  if (!match) return { label: null, text: item };
  return {
    label: match[1] as 'TRUE RELATIVE' | 'USEFUL COINCIDENCE',
    text: item.slice(match[0].length),
  };
}
