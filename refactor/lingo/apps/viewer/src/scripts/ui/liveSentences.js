import { replaceSentenceData } from '../data.js';
import { replaceSentenceSelection } from '../state/selection.js';
import { refreshSearchCaches, applyFilter } from './search.js';
import { syncInitialSelection } from './cardSelection.js';

let loadedSignature = '';

export function initLiveSentences() {
  window.addEventListener('tabchange', event => {
    if (event.detail?.tab === 'sentences') refreshLiveSentences();
  });
  window.addEventListener('sentencesrefresh', refreshLiveSentences);
}

async function refreshLiveSentences() {
  let payload;
  try {
    const response = await fetch('/api/generated/sentences');
    if (!response.ok) return;
    payload = await response.json();
  } catch {
    return;
  }

  const files = Array.isArray(payload.files) ? payload.files : [];
  const signature = files.map(file => `${file.file}:${JSON.stringify(file.data).length}`).join('|');
  if (signature === loadedSignature) return;
  loadedSignature = signature;

  const sentenceBatches = buildSentenceFiles(files);
  const sentenceGroups = buildSentenceGroups(sentenceBatches);
  const allSentences = sentenceGroups.flatMap(group => group.sentences);
  const sentenceGroupLabels = sentenceGroups.flatMap(group => group.sentences.map(() => group.label));
  const sentenceSearchIndex = allSentences.map((sentence, i) => ({
    i,
    h: sentence.hindi ?? '',
    r: sentence.romanisation ?? '',
    e: sentence.english ?? '',
    d: sentence.date_added ?? '',
    group: sentenceGroupLabels[i] ?? '',
  }));

  replaceSentenceData({
    allSentences,
    sentenceGroups,
    sentenceSearchIndex,
    dataHealth: {
      sentenceFiles: files.length,
      totalSentences: allSentences.length,
      sentenceAudioReady: allSentences.filter(sentence => Boolean(sentence.audio)).length,
      sentenceTokenReady: allSentences.filter(sentence => Array.isArray(sentence.tokens) && sentence.tokens.length > 0).length,
    },
  });

  renderSentences(sentenceGroups, allSentences);
  replaceSentenceSelection(allSentences.length);
  refreshSearchCaches();
  syncInitialSelection();
  applyFilter();
}

function metadataFromBatch(data, fallback) {
  const title = typeof data === 'object' && !Array.isArray(data) ? data.title : undefined;
  const subtitle = typeof data === 'object' && !Array.isArray(data) ? data.subtitle : undefined;
  const normalizedTitle = typeof title === 'string' && title.trim() ? title : fallback.replace(/_/g, ' ');
  const normalizedSubtitle = typeof subtitle === 'string' ? subtitle : '';
  return {
    title: normalizedTitle,
    subtitle: normalizedSubtitle,
    label: [normalizedTitle, normalizedSubtitle].filter(Boolean).join(' '),
  };
}

function normalizeSentenceToken(token) {
  return {
    ...token,
    hindi: token.hindi ?? token.target ?? '',
    roman: token.roman ?? token.romanisation ?? '',
    kind: token.kind ?? 'word',
    word_id: token.word_id,
    word_index: token.word_index,
  };
}

const GENDER_VALUES = new Set(['masculine', 'feminine', 'm', 'f']);
const NUMBER_VALUES = new Set(['singular', 'plural', 'sg', 'pl']);
const TENSE_VALUES = new Set(['present', 'past', 'future']);

function canonGender(value) {
  if (!value) return undefined;
  const s = String(value).toLowerCase();
  if (s.startsWith('m')) return 'masculine';
  if (s.startsWith('f')) return 'feminine';
  return undefined;
}

// Split a word's annotations into structured fields so the UI can render each as
// its own element instead of one run-on string. Handles both the rich shape
// (kind=part-of-speech, grammar=[…]) and the sparse shape (kind="word", no grammar).
function normalizeSentenceWord(word) {
  const grammar = Array.isArray(word.grammar) ? word.grammar.map(String) : [];
  const findIn = (set) => grammar.find(value => set.has(value.toLowerCase()));

  const gender = canonGender(word.gender ?? findIn(GENDER_VALUES));
  const number = word.number ?? findIn(NUMBER_VALUES);
  const tense = word.tense ?? findIn(TENSE_VALUES);

  const claimed = new Set(
    [word.gender ?? findIn(GENDER_VALUES), number, tense]
      .filter(Boolean)
      .map(value => String(value).toLowerCase())
  );
  const extra = grammar.filter(value => !claimed.has(value.toLowerCase()));

  // `kind` is "word" in sparse data (token kind, not POS); only treat it as a
  // part of speech when it carries a real label.
  const rawPos = word.part_of_speech ?? word.pos ?? word.kind;
  const pos = rawPos && String(rawPos).toLowerCase() !== 'word' ? rawPos : undefined;

  const notes = Array.isArray(word.notes)
    ? word.notes.filter(Boolean)
    : word.note
      ? [word.note]
      : [];

  // Many meanings pack a short gloss + a longer explanation into one string
  // joined by a dash ("I — the speaker referring to themselves"). Split them so
  // each gets its own visual treatment instead of relying on the dash.
  const { gloss, detail } = splitMeaning(word.meaning ?? '');

  return {
    ...word,
    hindi: word.hindi ?? word.target ?? '',
    roman: word.roman ?? word.romanisation ?? '',
    meaning: word.meaning ?? '',
    gloss,
    detail,
    pos,
    gender,
    number,
    tense,
    extra,
    note: notes.join(' ') || undefined,
  };
}

// Split "gloss — explanation" on the first em/en dash (or " - ") into two parts.
function splitMeaning(meaning) {
  const text = String(meaning ?? '').trim();
  // em/en dash with optional spaces, OR a plain hyphen that must be spaced
  // (so hyphenated words like "well-known" are never split).
  const match = text.match(/\s*[—–]\s*|\s+-\s+/);
  if (!match) return { gloss: text, detail: '' };
  const at = match.index ?? -1;
  return {
    gloss: text.slice(0, at).trim(),
    detail: text.slice(at + match[0].length).trim(),
  };
}

function normalizeLingoCard(card) {
  const audio = typeof card.audio === 'string'
    ? card.audio
    : card.audio?.relative_path;
  const tokens = Array.isArray(card.tokens) ? card.tokens.map(normalizeSentenceToken) : [];
  const words = Array.isArray(card.words) ? card.words.map(normalizeSentenceWord) : [];
  return {
    ...card,
    hindi: card.hindi ?? card.target ?? '',
    romanisation: card.romanisation ?? '',
    english: card.english ?? '',
    literal: card.literal ?? '',
    register: card.register,
    tokens,
    words,
    breakdown: buildBreakdown(tokens, words),
    anki_tags: card.anki_tags ?? card.tags ?? [],
    audio,
  };
}

function normalizeLegacySentence(sentence) {
  const tokens = Array.isArray(sentence.tokens) ? sentence.tokens.map(normalizeSentenceToken) : [];
  const words = Array.isArray(sentence.words) ? sentence.words.map(normalizeSentenceWord) : [];
  return {
    ...sentence,
    tokens,
    words,
    breakdown: buildBreakdown(tokens, words),
    anki_tags: sentence.anki_tags ?? sentence.tags ?? [],
  };
}

function buildBreakdown(tokens, words) {
  if (!Array.isArray(words) || words.length === 0) return [];
  if (!Array.isArray(tokens) || tokens.length === 0) return words;

  const byId = new Map(words.filter(word => word.id).map(word => [word.id, word]));
  const byIndex = new Map(words.map((word, index) => [index, word]));
  const rows = [];

  tokens.forEach((token, tokenIndex) => {
    const kind = token.kind ?? 'word';
    if (kind !== 'word') return;
    const linked = token.word_id
      ? byId.get(token.word_id)
      : typeof token.word_index === 'number'
        ? byIndex.get(token.word_index)
        : byIndex.get(tokenIndex);
    rows.push({
      ...(linked || {}),
      hindi: token.hindi || linked?.hindi || '',
      roman: token.roman || linked?.roman || '',
      meaning: linked?.meaning || '',
      gloss: linked?.gloss || linked?.meaning || '',
      detail: linked?.detail || '',
      missingWord: !linked,
    });
  });

  return rows.length > 0 ? rows : words;
}

function sentenceRowsFromBatch(data) {
  if (typeof data === 'object' && !Array.isArray(data) && data?.format === 'lingo.cards/v1') {
    return Array.isArray(data.cards) ? data.cards.map(normalizeLingoCard) : [];
  }
  const rawSentences = (typeof data === 'object' && !Array.isArray(data) && data.sentences)
    ? data.sentences
    : Array.isArray(data) ? data : [];
  return rawSentences.map(normalizeLegacySentence);
}

function buildSentenceFiles(sentenceFiles) {
  return sentenceFiles
    .map(({ stem, data }) => {
      const audioBatch = stem;
      const meta = metadataFromBatch(data, stem);
      const sentences = sentenceRowsFromBatch(data).map(sentence => ({
        ...sentence,
        audioBatch,
        title: meta.title,
        subtitle: meta.subtitle,
        groupLabel: meta.label,
      }));
      return { groupLabel: meta.label, title: meta.title, subtitle: meta.subtitle, stem, sentences };
    })
    .filter(batch => batch.sentences.length > 0);
}

function buildSentenceGroups(sentenceBatchFiles) {
  const merged = new Map();
  for (const batch of sentenceBatchFiles) {
    if (!merged.has(batch.groupLabel)) {
      merged.set(batch.groupLabel, { label: batch.groupLabel, title: batch.title, subtitle: batch.subtitle, sentences: [], batches: [] });
    }
    const group = merged.get(batch.groupLabel);
    group.batches.push(batch.stem);
    group.sentences.push(...batch.sentences);
  }
  return [...merged.values()];
}

function renderSentences(sentenceGroups, allSentences) {
  const subtitle = document.getElementById('ps-subtitle');
  if (subtitle) {
    subtitle.textContent = allSentences.length === 0
      ? 'No sentences loaded yet'
      : `${allSentences.length} ${allSentences.length === 1 ? 'sentence' : 'sentences'}`;
  }

  const chips = document.getElementById('ps-filter-chips');
  if (chips) {
    chips.classList.toggle('hidden', sentenceGroups.length === 0);
    chips.innerHTML = [
      '<button class="pw-filter-chip is-active" data-group-chip="all">All</button>',
      ...sentenceGroups.map(group => `<button class="pw-filter-chip" data-group-chip="${esc(group.label)}">${esc(group.label)}</button>`),
    ].join('');
  }

  const body = document.getElementById('ps-live-body');
  if (!body) return;
  if (allSentences.length === 0) {
    body.innerHTML = `<div class="py-24 text-center">
      <p class="text-zinc-600 text-sm mb-2">No sentences loaded yet.</p>
      <p class="text-zinc-700 text-xs">Generate sentence batches in Studio, then open this tab.</p>
    </div>`;
    return;
  }

  let index = 0;
  body.innerHTML = `<p class="drag-select-hint">Drag to select multiple</p>
    ${sentenceGroups.map(group => renderGroup(group, () => index++)).join('')}
    <div id="sentences-no-results" class="hidden py-20 text-center">
      <p class="text-zinc-600 text-sm">No sentences match your search.</p>
    </div>`;
}

function renderGroup(group, nextIndex) {
  const groupId = `cgw-s-${slug(group.label)}`;
  return `<div class="card-group-wrapper" id="${esc(groupId)}">
      <div class="card-group-divider">
        <div class="card-group-header" data-group-toggle="${esc(groupId)}">
          <span class="card-group-checkbox" role="checkbox" aria-checked="false" aria-label="Select all in ${esc(group.label)}" data-group-checkbox="${esc(groupId)}"></span>
          <span class="card-group-label">
            <span class="card-group-title">${esc(group.title || group.label)}</span>
            ${group.subtitle ? `<span class="card-group-subtitle">${esc(group.subtitle)}</span>` : ''}
          </span>
          <span class="card-group-line"></span>
          <span class="card-group-count">${group.sentences.length} ${group.sentences.length === 1 ? 'sentence' : 'sentences'}${group.batches?.length > 1 ? ` · ${group.batches.length} batches` : ''}</span>
          <button class="quick-export-btn" type="button" data-quick-export="sentences" data-quick-export-title="${esc(group.label)}">Quick export</button>
          <span class="card-group-chevron">▾</span>
        </div>
      </div>
    </div>
    <div class="card-list" data-group="${esc(groupId)}">
      ${group.sentences.map(sentence => renderCard(sentence, nextIndex())).join('')}
    </div>`;
}

function renderCard(sentence, index) {
  const audioSrc = audioUrl(sentence.audio);
  const tokenCount = Array.isArray(sentence.tokens)
    ? sentence.tokens.filter(token => (token.kind ?? 'word') === 'word').length
    : 0;
  return `<article id="sentence-card-${index}" data-sentence-card="${index}" data-sentence-index="${index}"
      class="sc-card border border-slate-700/40 rounded-2xl overflow-hidden hover:border-slate-600/60 transition-colors is-collapsed"
      style="background: linear-gradient(145deg, #131f35 0%, #0f172a 100%); box-shadow: 0 0 0 1px rgba(51,65,85,0.5), 0 4px 24px rgba(0,0,0,0.4);">
      <div class="card-header flex px-4 sm:px-5 py-3 sm:py-4 gap-2 cursor-pointer hover:bg-slate-800/20 transition-colors select-none">
        <div class="flex-1 min-w-0 flex flex-col gap-2 py-1">
          <h2 class="text-xl sm:text-2xl font-bold leading-snug" lang="hi" style="background: linear-gradient(135deg, #fbbf24, #f97316); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">${esc(sentence.hindi)}</h2>
          <p class="word-roman text-sm sm:text-base leading-snug" style="text-shadow: 0 0 20px rgba(94,234,212,0.2);">${esc(sentence.romanisation)}</p>
          ${audioSrc ? `<div class="wc-audio-row"><button class="wc-audio-btn" data-src="${esc(audioSrc)}" data-label="Play">🔊 Play</button></div>` : ''}
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-slate-200 text-sm leading-none">${esc(sentence.english)}</span>
            ${sentence.register ? `<span class="text-xs font-title font-semibold uppercase tracking-wider px-2.5 py-1 rounded-lg ${registerStyle(sentence.register)}">${esc(sentence.register)}</span>` : ''}
            <span class="text-[10px] font-title font-semibold uppercase tracking-wider px-2 py-0.5 rounded-md border ${tokenCount > 0 ? 'bg-teal-900/20 text-teal-300/80 border-teal-700/30' : 'bg-amber-900/20 text-amber-300/80 border-amber-700/30'}">
              ${tokenCount > 0 ? `${tokenCount} ${tokenCount === 1 ? 'word' : 'words'}` : 'legacy'}
            </span>
          </div>
        </div>
        <button class="sel-circle" aria-label="Select sentence" data-sel-circle="${index}"></button>
      </div>
      <div class="card-expandable">
        ${sentence.literal ? `<div class="px-4 sm:px-6 pt-3 pb-2 border-t border-slate-700/40">
          <p class="text-[13px] text-slate-500 italic leading-relaxed"><span class="font-title text-[9px] font-semibold uppercase tracking-widest text-zinc-600 mr-2 not-italic">Literal</span>${esc(sentence.literal)}</p>
        </div>` : ''}
        ${renderWords(sentence.breakdown)}
      </div>
    </article>`;
}

function badge(label, style) {
  return `<span class="${BADGE} ${style}">${esc(label)}</span>`;
}

function wordChips(word) {
  const chips = [];
  if (word.pos) chips.push(badge(word.pos, categoryStyle(word.pos)));
  if (word.gender === 'masculine') chips.push(badge('m', 'bg-blue-900/20 text-blue-300/80 border border-blue-700/30'));
  if (word.gender === 'feminine') chips.push(badge('f', 'bg-pink-900/20 text-pink-300/80 border border-pink-700/30'));
  if (word.number) chips.push(badge(word.number, NEUTRAL_BADGE));
  if (word.tense) chips.push(badge(word.tense, NEUTRAL_BADGE));
  if (Array.isArray(word.extra)) {
    for (const value of word.extra) chips.push(badge(value, NEUTRAL_BADGE));
  }
  return chips.join('');
}

function renderWord(word) {
  const chips = wordChips(word);
  return `<div class="sc-word">
    <div class="sc-word-lex">
      <span class="sc-word-deva" lang="hi">${esc(word.hindi)}</span>
      ${word.roman ? `<span class="sc-word-roman">${esc(word.roman)}</span>` : ''}
    </div>
    <div class="sc-word-body">
      <div class="sc-word-head">
        <span class="sc-word-meaning">${esc(word.gloss || word.meaning)}</span>
        ${chips ? `<span class="sc-word-chips">${chips}</span>` : ''}
      </div>
      ${word.detail ? `<p class="sc-word-detail">${esc(word.detail)}</p>` : ''}
      ${word.note ? `<p class="sc-word-note">${esc(word.note)}</p>` : ''}
    </div>
  </div>`;
}

function renderWords(words) {
  if (!Array.isArray(words) || words.length === 0) return '';
  return `<div class="px-4 sm:px-6 py-3 sm:py-4 border-t border-slate-700/40">
    <p class="font-title text-xs sm:text-sm font-bold tracking-[0.2em] uppercase text-slate-400 mb-3">Word by word</p>
    <div class="sc-word-list">
      ${words.map(renderWord).join('')}
    </div>
  </div>`;
}

function audioUrl(audio) {
  if (typeof audio !== 'string') return '';
  const value = audio.trim();
  if (!value) return '';
  if (value.startsWith('/audio/')) return value;
  if (value.startsWith('audio/')) return `/${value}`;
  return '';
}

function registerStyle(register) {
  const map = {
    formal: 'bg-violet-900/30 text-violet-400 border border-violet-700/40',
    neutral: 'bg-cyan-900/30 text-cyan-300 border border-cyan-700/40',
    standard: 'bg-sky-900/30 text-sky-400 border border-sky-700/40',
    casual: 'bg-amber-500/20 text-amber-400 border border-amber-500/30',
    colloquial: 'bg-rose-900/30 text-rose-400 border border-rose-700/40',
  };
  return map[register] ?? 'bg-slate-800/60 text-slate-400 border border-slate-700/40';
}

// Shared badge geometry — identical to the WordCard grammar badges so the two
// card types speak the same design language.
const BADGE = 'text-xs font-title font-semibold uppercase tracking-wider px-2.5 py-1 rounded-lg';
const NEUTRAL_BADGE = 'bg-slate-800/60 text-slate-400 border border-slate-700/40';

// Mirror of categoryStyle() in utils/cardHelpers.ts — part-of-speech colours.
function categoryStyle(pos) {
  const c = String(pos).toLowerCase();
  if (c.includes('conjunction')) return 'bg-emerald-900/30 text-emerald-400 border border-emerald-700/40';
  if (c.includes('adverb')) return 'bg-sky-900/30 text-sky-400 border border-sky-700/40';
  if (c.includes('noun')) return 'bg-violet-900/30 text-violet-400 border border-violet-700/40';
  if (c.includes('adjective')) return 'bg-amber-900/30 text-amber-500 border border-amber-700/40';
  if (c.includes('verb')) return 'bg-orange-900/30 text-orange-400 border border-orange-700/40';
  if (c.includes('preposition') || c.includes('postposition')) return 'bg-rose-900/30 text-rose-400 border border-rose-700/40';
  if (c.includes('particle')) return 'bg-teal-900/30 text-teal-400 border border-teal-700/40';
  if (c.includes('pronoun')) return 'bg-indigo-900/30 text-indigo-400 border border-indigo-700/40';
  return NEUTRAL_BADGE;
}

function slug(value) {
  return String(value).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'sentences';
}

function esc(value) {
  return String(value ?? '').replace(/[&<>"]/g, char => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[char]));
}
