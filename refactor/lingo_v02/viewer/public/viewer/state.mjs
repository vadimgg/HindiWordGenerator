export const sentenceStatuses = ['draft', 'enriching', 'enriched', 'active'];

export function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

export const sampleState = {
  workspace: {
    name: 'Complete Hindi',
    language: 'Hindi',
    languageCode: 'hi',
    libraryPath: 'library.db',
  },
  config: {
    display: { lead: 'romanisation', showSecondary: true },
    audio: { backend: 'gtts', voice: '', model: '' },
    anki: { deck: 'Hindi::Sentences', replace: false },
    package: { destination: 'packages/sentences', format: 'json' },
  },
  sentences: [
    {
      id: '01HIN00000000000000000001',
      collection: 'Complete Hindi',
      section: 'Chapter 02',
      order: 1,
      target: 'अध्यापक जी, यहाँ कितने विद्यार्थी हैं?',
      romanisation: 'adhyāpak jī, yahā̃ kitne vidyārthī haĩ?',
      english: 'Teacher ji, how many students are here?',
      literal: 'teacher ji here how-many students are',
      register: 'standard',
      status: 'enriched',
      tags: ['classroom', 'question'],
      authority: { english: 'human' },
      breakdown: [
        { surface: 'अध्यापक', roman: 'adhyāpak', gloss: 'teacher', kind: 'noun' },
        { surface: 'जी', roman: 'jī', gloss: 'honorific (ji)', kind: 'particle' },
        { surface: 'यहाँ', roman: 'yahā̃', gloss: 'here', kind: 'adverb' },
        { surface: 'कितने', roman: 'kitne', gloss: 'how many', kind: 'question' },
        { surface: 'विद्यार्थी', roman: 'vidyārthī', gloss: 'students', kind: 'noun' },
        { surface: 'हैं', roman: 'haĩ', gloss: 'are', kind: 'verb' },
      ],
      audio: { path: '/audio/01HIN00000000000000000001.mp3', backend: 'gtts', voice: null, hash: 'sha256:demo' },
      provenance: { kind: 'imported', package: 'demo-package' },
    },
    {
      id: '01HIN00000000000000000002',
      collection: 'Complete Hindi',
      section: 'Chapter 02',
      order: 2,
      target: 'मैं लड़की हूँ।',
      romanisation: 'maĩ laṛkī hū̃.',
      english: 'I am a girl.',
      literal: 'I girl am',
      register: 'standard',
      status: 'enriched',
      tags: ['identity'],
      authority: { english: 'ai', romanisation: 'ai', literal: 'ai', breakdown: 'ai' },
      breakdown: [
        { surface: 'मैं', roman: 'maĩ', gloss: 'I', kind: 'pronoun' },
        { surface: 'लड़की', roman: 'laṛkī', gloss: 'girl', kind: 'noun' },
        { surface: 'हूँ', roman: 'hū̃', gloss: 'am', kind: 'verb' },
      ],
      audio: { path: '/audio/01HIN00000000000000000002.mp3', backend: 'gtts', voice: null, hash: 'sha256:demo' },
      provenance: { kind: 'generated', run: 'extract-demo' },
    },
    {
      id: '01HIN00000000000000000003',
      collection: 'Complete Hindi',
      section: 'Chapter 02',
      order: 3,
      target: 'मैं आदमी हूँ।',
      romanisation: 'maĩ ādmī hū̃.',
      english: 'I am a man.',
      literal: 'I man am',
      register: 'standard',
      status: 'enriched',
      tags: ['identity'],
      authority: { english: 'ai' },
      breakdown: [
        { surface: 'मैं', roman: 'maĩ', gloss: 'I', kind: 'pronoun' },
        { surface: 'आदमी', roman: 'ādmī', gloss: 'man', kind: 'noun' },
        { surface: 'हूँ', roman: 'hū̃', gloss: 'am', kind: 'verb' },
      ],
      audio: null,
      provenance: { kind: 'generated', run: 'extract-demo' },
    },
    {
      id: '01HIN00000000000000000004',
      collection: 'Complete Hindi',
      section: 'Chapter 03',
      order: 4,
      target: 'यह किताब अच्छी है।',
      romanisation: null,
      english: 'This book is good.',
      literal: null,
      register: null,
      status: 'draft',
      tags: ['object'],
      authority: { english: 'human' },
      breakdown: [],
      audio: null,
      provenance: { kind: 'generated', run: 'extract-demo' },
    },
    {
      id: '01HIN00000000000000000005',
      collection: 'Complete Hindi',
      section: 'Chapter 03',
      order: 5,
      target: 'क्या आप ठीक हैं?',
      romanisation: 'kyā āp ṭhīk haĩ?',
      english: 'Are you okay?',
      literal: null,
      register: null,
      status: 'enriching',
      enrichRun: 'enrich-demo',
      tags: ['health', 'question'],
      authority: { english: 'human', romanisation: 'ai' },
      breakdown: [],
      audio: null,
      provenance: { kind: 'generated', run: 'extract-demo' },
    },
  ],
  words: [],
};

export function normalizeText(value) {
  return String(value ?? '')
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .trim();
}

export function hasAudio(sentence) {
  return Boolean(sentence?.audio?.path || sentence?.audio === true);
}

export function deriveSummary(state) {
  const sentences = state.sentences ?? [];
  const summary = {
    sentences: sentences.length,
    draft: 0,
    enriching: 0,
    enriched: 0,
    active: 0,
    audioReady: 0,
    words: (state.words ?? []).length,
  };
  for (const sentence of sentences) {
    if (sentence.status === 'draft') summary.draft += 1;
    if (sentence.status === 'enriching') summary.enriching += 1;
    if (sentence.status === 'enriched') summary.enriched += 1;
    if (sentence.status === 'active') summary.active += 1;
    if (hasAudio(sentence)) summary.audioReady += 1;
  }
  return summary;
}

export function statusClass(status) {
  if (status === 'active') return 'ready';
  if (status === 'enriched') return 'review';
  if (status === 'enriching') return 'wait';
  if (status === 'draft') return 'draft';
  return 'neutral';
}

export function audioClass(sentence) {
  return hasAudio(sentence) ? 'ready' : 'missing';
}

export function groupSentences(sentences) {
  const groups = new Map();
  const sorted = [...(sentences ?? [])].sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
  for (const sentence of sorted) {
    const section = sentence.section || 'Unsectioned';
    if (!groups.has(section)) groups.set(section, []);
    groups.get(section).push(sentence);
  }
  return [...groups.entries()].map(([section, rows]) => ({ section, rows }));
}

export function filterSentences(sentences, { search = '', status = 'all' } = {}) {
  const needle = normalizeText(search);
  return (sentences ?? []).filter((sentence) => {
    if (status === 'missing-audio' && hasAudio(sentence)) return false;
    if (status !== 'all' && status !== 'missing-audio' && sentence.status !== status) return false;
    if (!needle) return true;
    const haystack = [
      sentence.target,
      sentence.romanisation,
      sentence.english,
      sentence.literal,
      sentence.section,
      ...(sentence.tags ?? []),
    ].map(normalizeText).join(' ');
    return haystack.includes(needle);
  });
}

export function buildWordsFromSentences(sentences) {
  const byKey = new Map();
  for (const sentence of sentences ?? []) {
    for (const item of sentence.breakdown ?? []) {
      const form = item.surface || '';
      const key = normalizeText(form);
      if (!key) continue;
      if (!byKey.has(key)) {
        byKey.set(key, {
          key,
          form,
          roman: item.roman ?? '',
          kind: item.kind ?? '',
          count: 0,
          meanings: [],
          sentenceIds: [],
        });
      }
      const row = byKey.get(key);
      if (!row.sentenceIds.includes(sentence.id)) {
        row.count += 1;
        row.sentenceIds.push(sentence.id);
      }
      if (item.gloss && !row.meanings.includes(item.gloss)) row.meanings.push(item.gloss);
    }
  }
  return [...byKey.values()].sort((a, b) => b.count - a.count || a.form.localeCompare(b.form));
}

export function readySentences(sentences) {
  return (sentences ?? []).filter((sentence) => (sentence.status === 'enriched' || sentence.status === 'active') && hasAudio(sentence));
}

export function selectedOrAllReady(state, selectedIds) {
  const selected = (state.sentences ?? []).filter((sentence) => selectedIds?.has(sentence.id));
  if (selected.length) return selected;
  return readySentences(state.sentences);
}

export function safeState(input) {
  const state = clone(input || sampleState);
  state.workspace = { ...sampleState.workspace, ...(state.workspace || {}) };
  state.config = {
    display: { ...sampleState.config.display, ...(state.config?.display || {}) },
    audio: { ...sampleState.config.audio, ...(state.config?.audio || {}) },
    anki: { ...sampleState.config.anki, ...(state.config?.anki || {}) },
    package: { ...sampleState.config.package, ...(state.config?.package || {}) },
  };
  state.sentences = Array.isArray(state.sentences) ? state.sentences : [];
  state.words = Array.isArray(state.words) && state.words.length ? state.words : buildWordsFromSentences(state.sentences);
  return state;
}
