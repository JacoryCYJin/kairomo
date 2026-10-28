import test from 'node:test';
import assert from 'node:assert/strict';
import { matchesPhrase, phraseAttribution } from './phrases.ts';
test('legacy and personal messages have no invented attribution', () => {
  assert.equal(phraseAttribution({ id: 'a', text: 'hello' }), '');
  assert.equal(phraseAttribution(undefined), '');
});
test('author and work can each be supplied independently', () => {
  const p = { id: 'a', text: '明月松间照', author: ' 王维 ', source_title: ' 山居秋暝 ' };
  assert.equal(phraseAttribution(p), '王维《山居秋暝》');
  assert.equal(phraseAttribution({ ...p, author: '' }), '《山居秋暝》');
  assert.equal(phraseAttribution({ ...p, source_title: '' }), '王维');
});
test('search includes text, author and work with case-insensitive multiword matches', () => {
  const p = {
    id: 'a',
    text: 'One step at a time',
    author: 'Jane Writer',
    source_title: 'A Quiet Book',
  };
  assert.equal(matchesPhrase(p, 'JANE quiet'), true);
  assert.equal(matchesPhrase(p, 'step'), true);
  assert.equal(matchesPhrase(p, 'missing'), false);
  assert.equal(matchesPhrase({ id: 'old', text: '你好' }, '你好'), true);
});
