import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { getSentenceTokenIssue, hasExactSentenceTokens } from '../scripts/quality/sentenceTokens.js';

async function readJsonFiles(dir) {
  try {
    const files = (await readdir(dir)).filter(file => file.endsWith('.json')).sort();
    const loaded = await Promise.all(
      files.map(async (file) => {
        try {
          return {
            file,
            stem: file.replace('.json', ''),
            data: JSON.parse(await readFile(join(dir, file), 'utf-8')),
          };
        } catch (error) {
          console.warn(`[viewer] Skipping invalid JSON file: ${join(dir, file)}`);
          console.warn(error);
          return null;
        }
      })
    );
    return loaded.filter(Boolean);
  } catch (error) {
    console.warn(`[viewer] No readable JSON directory: ${dir}`);
    return [];
  }
}

function metadataFromBatch(data, fallback) {
  const title = typeof data === 'object' && !Array.isArray(data)
    ? data.title
    : undefined;
  const subtitle = typeof data === 'object' && !Array.isArray(data)
    ? data.subtitle
    : undefined;
  if (typeof title !== 'string' || !title.trim()) {
    console.warn(`[viewer] Missing title in generated batch: ${fallback}`);
  }
  const normalizedTitle = title || fallback.replace(/_/g, ' ');
  const normalizedSubtitle = typeof subtitle === 'string' ? subtitle : '';
  return {
    title: normalizedTitle,
    subtitle: normalizedSubtitle,
    label: [normalizedTitle, normalizedSubtitle].filter(Boolean).join(' '),
  };
}

function buildWordFiles(wordFiles) {
  return wordFiles.map(({ stem, data }) => {
    const audioBatch = stem;
    const meta = metadataFromBatch(data, stem);
    const rawWords = Array.isArray(data) ? data : (data.words ?? []);
    const words = rawWords.map(word => ({
      ...word,
      audioBatch,
      title: meta.title,
      subtitle: meta.subtitle,
      groupLabel: meta.label,
    }));
    return { title: meta.label, words };
  });
}

function buildWordGroups(allFiles) {
  let offset = 0;
  const merged = new Map();
  let groupIndex = 0;

  for (const file of allFiles.filter(item => item.words.length > 0)) {
    if (!merged.has(file.title)) {
      merged.set(file.title, { id: String(groupIndex++), title: file.title, words: [] });
    }
    const group = merged.get(file.title);
    for (const word of file.words) {
      group.words.push({ w: word, i: offset++ });
    }
  }

  return [...merged.values()];
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

function normalizeSentenceWord(word) {
  const grammar = Array.isArray(word.grammar) ? word.grammar : [];
  const noteParts = [
    word.note,
    word.kind ? `kind: ${word.kind}` : undefined,
    grammar.length > 0 ? `grammar: ${grammar.join(', ')}` : undefined,
  ].filter(Boolean);
  return {
    ...word,
    hindi: word.hindi ?? word.target ?? '',
    roman: word.roman ?? word.romanisation ?? '',
    meaning: word.meaning ?? '',
    gender: word.gender ?? grammar.find(value => value === 'masculine' || value === 'feminine'),
    number: word.number ?? grammar.find(value => value === 'singular' || value === 'plural'),
    note: noteParts.length > 0 ? noteParts.join(' · ') : undefined,
  };
}

function normalizeLingoCard(card) {
  const audio = typeof card.audio === 'string'
    ? card.audio
    : card.audio?.relative_path;
  return {
    ...card,
    hindi: card.hindi ?? card.target ?? '',
    romanisation: card.romanisation ?? '',
    english: card.english ?? '',
    literal: card.literal ?? '',
    register: card.register,
    tokens: Array.isArray(card.tokens) ? card.tokens.map(normalizeSentenceToken) : [],
    words: Array.isArray(card.words) ? card.words.map(normalizeSentenceWord) : [],
    anki_tags: card.anki_tags ?? card.tags ?? [],
    audio,
  };
}

function normalizeLegacySentence(sentence) {
  return {
    ...sentence,
    tokens: Array.isArray(sentence.tokens) ? sentence.tokens.map(normalizeSentenceToken) : [],
    words: Array.isArray(sentence.words) ? sentence.words.map(normalizeSentenceWord) : [],
    anki_tags: sentence.anki_tags ?? sentence.tags ?? [],
  };
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
      return { groupLabel: meta.label, stem, sentences };
    })
    .filter(batch => batch.sentences.length > 0);
}

function buildSentenceGroups(sentenceBatchFiles) {
  const merged = new Map();

  for (const batch of sentenceBatchFiles) {
    if (!merged.has(batch.groupLabel)) {
      merged.set(batch.groupLabel, { label: batch.groupLabel, sentences: [], batches: [] });
    }
    const group = merged.get(batch.groupLabel);
    group.batches.push(batch.stem);
    group.sentences.push(...batch.sentences);
  }

  return [...merged.values()];
}

function buildQaIssues(allWords, allSentences) {
  const wordIssues = allWords.flatMap((word, index) => {
    if (word.audio) return [];
    return [{
      type: 'audio',
      cardType: 'word',
      wordIndex: index,
      label: 'missing audio path',
      hindi: word.hindi ?? '',
      english: word.english ?? '',
      groupLabel: word.groupLabel ?? '',
    }];
  });

  const sentenceIssues = allSentences.flatMap((sentence, index) => {
    const issues = [];
    const tokenIssue = getSentenceTokenIssue(sentence);
    if (tokenIssue) {
      issues.push({
        type: 'tokens',
        cardType: 'sentence',
        sentenceIndex: index,
        label: tokenIssue.reason,
        hindi: sentence.hindi ?? '',
        english: sentence.english ?? '',
        groupLabel: sentence.groupLabel ?? '',
        expected: tokenIssue.expected,
        actual: tokenIssue.actual,
      });
    }
    if (!sentence.audio) {
      issues.push({
        type: 'audio',
        cardType: 'sentence',
        sentenceIndex: index,
        label: 'missing audio path',
        hindi: sentence.hindi ?? '',
        english: sentence.english ?? '',
        groupLabel: sentence.groupLabel ?? '',
      });
    }
    return issues;
  });

  return [...wordIssues, ...sentenceIssues];
}

export async function loadGeneratedData(projectRoot) {
  const wordFiles = await readJsonFiles(join(projectRoot, 'output', 'words'));
  const allFiles = buildWordFiles(wordFiles);
  const allWords = allFiles.flatMap(file => file.words);
  const wordGroups = buildWordGroups(allFiles);
  const wordGroupTitles = new Array(allWords.length).fill('Hindi Vocabulary');
  wordGroups.forEach(group => group.words.forEach(({ i }) => { wordGroupTitles[i] = group.title; }));

  const sentenceFiles = await readJsonFiles(join(projectRoot, 'output', 'sentences'));
  const sentenceBatchFiles = buildSentenceFiles(sentenceFiles);
  const sentenceGroups = buildSentenceGroups(sentenceBatchFiles);
  const allSentences = sentenceGroups.flatMap(group => group.sentences);
  const sentenceGroupLabels = sentenceGroups.flatMap(group =>
    group.sentences.map(() => group.label)
  );

  const wordSearchIndex = allWords.map((word, i) => ({
    i,
    h: word.hindi,
    r: word.romanisation,
    e: word.english,
    d: word.date_added ?? '',
  }));
  const sentenceSearchIndex = allSentences.map((sentence, i) => ({
    i,
    h: sentence.hindi ?? '',
    r: sentence.romanisation ?? '',
    e: sentence.english ?? '',
    d: sentence.date_added ?? '',
    group: sentenceGroupLabels[i] ?? '',
  }));
  const hoverData = allWords.map((word, i) => ({
    i,
    hindi: word.hindi,
    roman: word.romanisation,
    english: String(word.english ?? '').split(',')[0].trim(),
    forms: (word.forms || []).map(form => ({ h: form.hindi, r: form.roman })),
  }));

  const dataHealth = {
    wordFiles: wordFiles.length,
    sentenceFiles: sentenceFiles.length,
    totalWords: allWords.length,
    totalSentences: allSentences.length,
    wordAudioReady: allWords.filter(word => Boolean(word.audio)).length,
    sentenceAudioReady: allSentences.filter(sentence => Boolean(sentence.audio)).length,
    sentenceTokenReady: allSentences.filter(hasExactSentenceTokens).length,
  };

  return {
    wordFiles,
    sentenceFiles,
    allWords,
    wordGroups,
    wordGroupTitles,
    wordSearchIndex,
    sentenceSearchIndex,
    hoverData,
    allSentences,
    sentenceGroups,
    dataHealth,
    qaIssues: buildQaIssues(allWords, allSentences),
  };
}
