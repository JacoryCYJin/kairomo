import test from 'node:test';
import assert from 'node:assert/strict';
import { timerLayout, marqueeTiming } from './timerLayout.ts';
function state(entries = [], selected = entries[0]?.id ?? null) {
  return {
    settings: { phrases: { enabled: true, mode: 'fixed', entries, selected_id: selected } },
  };
}
test('disabled and empty messages use the compact layout without a reserved text row', () => {
  assert.equal(timerLayout(state()).height, 284);
  assert.equal(timerLayout(state([{ id: 'blank', text: '  ', author: 'Author' }])).height, 284);
  const disabled = state([{ id: 'a', text: 'message', author: 'Author' }]);
  disabled.settings.phrases.enabled = false;
  assert.equal(timerLayout(disabled).height, 284);
});
test('body only has a distinct fixed height; either attribution field enables the source row', () => {
  const message = { id: 'a', text: 'message', author: ' ', source_title: ' ' };
  assert.equal(timerLayout(state([message])).height, 302);
  for (const fields of [
    { author: 'Writer' },
    { source_title: 'Book' },
    { author: 'Writer', source_title: 'Book' },
  ]) {
    assert.equal(timerLayout(state([{ ...message, ...fields }])).height, 320);
  }
  for (const height of [284, 302, 320]) {
    const entries =
      height === 284 ? [] : [{ ...message, ...(height === 320 ? { author: 'Writer' } : {}) }];
    const layout = timerLayout(state(entries));
    assert.equal(
      layout.controlsTop + 48 + 73,
      height,
      'the button and supported pile keep the same bottom clearance',
    );
  }
});
test('text length and session snapshots cannot alter the chosen layout or random selection', () => {
  const short = { id: 'short', text: 'Short', author: 'Writer' };
  const long = { id: 'long', text: '长句'.repeat(500) };
  const snapshot = state([short, long]);
  snapshot.settings.phrases.mode = 'random';
  snapshot.phrase_selection = { session_id: 'same-round', entry_id: 'long' };
  assert.equal(timerLayout(snapshot).phrase.id, 'long');
  assert.equal(timerLayout(snapshot).height, 302);
  for (const phase of ['running', 'paused', 'idle']) {
    snapshot.phase = phase;
    assert.equal(timerLayout(snapshot).height, 302);
  }
  snapshot.phrase_selection.entry_id = 'short';
  assert.equal(timerLayout(snapshot).height, 320);
});
test('marquee preserves a 22px/s horizontal speed and pauses at both ends', () => {
  for (const overflow of [22, 220, 2200]) {
    const timing = marqueeTiming(overflow);
    assert.ok(Math.abs(timing.start * timing.duration - 2200) < 0.001);
    assert.ok(Math.abs((1 - timing.end) * timing.duration - 2200) < 0.001);
    assert.ok(
      Math.abs((timing.end - timing.start) * timing.duration - (overflow / 22) * 1000) < 0.001,
    );
  }
});
