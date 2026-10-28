import test from 'node:test';
import assert from 'node:assert/strict';
import { phrasePages } from './phrasePages.ts';
const graphemes = (s) =>
  [...new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(s)].map((v) => v.segment);
for (const text of [
  '短句',
  '很長的自訂短句。'.repeat(200),
  'A thoughtful sentence with natural word breaks. '.repeat(50),
  'averylongword'.repeat(200),
  '👨‍👩‍👧‍👦é🇯🇵'.repeat(50),
  'line one\n\nline two',
])
  test(`pagination preserves all text: ${text.slice(0, 16)}`, () => {
    const pages = phrasePages(text, (s) => graphemes(s).length <= 40);
    assert.equal(pages.join(''), text);
    assert.ok(pages.every((s) => graphemes(s).length <= 40 && s.length > 0));
    assert.deepEqual(pages.flatMap(graphemes), graphemes(text));
  });
test('Chinese pages prefer clause boundaries and avoid orphan punctuation', () => {
  const text =
    '把注意力放在眼前的一步。每一段认真投入的时间，都会留下属于自己的积累。慢慢来，先完成今天最重要的一件事。';
  const pages = phrasePages(text, (s) => graphemes(s).length <= 43);
  assert.equal(pages.join(''), text);
  assert.ok(pages.every((p) => !/^\s*[，。！？、；：]/u.test(p)));
});
