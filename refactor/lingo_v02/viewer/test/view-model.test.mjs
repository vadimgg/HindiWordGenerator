import test from 'node:test';
import assert from 'node:assert/strict';
import {
  buildWordsFromSentences,
  deriveSummary,
  filterSentences,
  groupSentences,
  hasAudio,
  safeState,
  sampleState,
  selectedOrAllReady,
  statusClass,
} from '../public/viewer/state.mjs';
import { commandForPage, commandLibrary, privateForTests, slug } from '../public/viewer/commands.mjs';

test('deriveSummary counts sentence states and audio from canonical rows', () => {
  const state = safeState(sampleState);
  const summary = deriveSummary(state);
  assert.equal(summary.sentences, 5);
  assert.equal(summary.draft, 1);
  assert.equal(summary.enriching, 1);
  assert.equal(summary.enriched, 3);
  assert.equal(summary.audioReady, 2);
});

test('filterSentences supports status, missing audio, and text search', () => {
  const state = safeState(sampleState);
  assert.equal(filterSentences(state.sentences, { status: 'draft' }).length, 1);
  assert.equal(filterSentences(state.sentences, { status: 'missing-audio' }).length, 3);
  assert.equal(filterSentences(state.sentences, { search: 'teacher ji' }).length, 1);
  assert.equal(filterSentences(state.sentences, { search: 'identity' }).length, 2);
});

test('groupSentences is ordered by library order and section', () => {
  const groups = groupSentences(sampleState.sentences);
  assert.deepEqual(groups.map((g) => g.section), ['Chapter 02', 'Chapter 03']);
  assert.deepEqual(groups[0].rows.map((row) => row.order), [1, 2, 3]);
});

test('word projection uses normalized surface form identity', () => {
  const words = buildWordsFromSentences(sampleState.sentences);
  const main = words.find((word) => word.form === 'मैं');
  assert.equal(main.count, 2);
  assert.deepEqual(main.meanings, ['I']);
});

test('selectedOrAllReady defaults to ready-to-publish rows', () => {
  const state = safeState(sampleState);
  assert.equal(selectedOrAllReady(state, new Set()).length, 2);
  assert.equal(selectedOrAllReady(state, new Set(['01HIN00000000000000000003'])).length, 1);
  assert.equal(hasAudio(state.sentences[0]), true);
  assert.equal(statusClass('enriching'), 'wait');
});

test('CLI commands mirror new command names', () => {
  const state = safeState(sampleState);
  const commands = commandLibrary(state, { selectedIds: new Set(['01HIN00000000000000000001']) });
  assert.match(commands['generate.extract'].command, /^lingo extract /);
  assert.match(commands['generate.enrich'].command, /^lingo enrich --limit /);
  assert.match(commands.import.command, /^lingo import --from /);
  assert.match(commands.package.command, /^lingo package --id /);
  assert.match(commandForPage('anki', state, { selectedIds: new Set() }), /^lingo export --all /);
});

test('command quoting and slugging are stable', () => {
  assert.equal(slug('Chapter 02'), 'chapter-02');
  assert.equal(privateForTests.sectionArg('Chapter 02'), '--section "Chapter 02"');
});
