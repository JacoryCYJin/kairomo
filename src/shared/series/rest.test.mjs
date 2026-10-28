import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { availableSeries } from './catalog.ts';
import { remainingRestFraction, restThemeStyle } from './rest.ts';

test('rest volume covers ready, partial, paused/reopened and overdue snapshots without mutating time', () => {
  const session = Object.freeze({ duration_ms: 300_000, target_ms: 600_000 });
  assert.equal(remainingRestFraction(0, session.target_ms), 1);
  assert.equal(remainingRestFraction(session.duration_ms, session.target_ms), 0.5);
  assert.ok(Math.abs(remainingRestFraction(599_000, session.target_ms) - 1 / 600) < 1e-12);
  assert.equal(remainingRestFraction(600_000, session.target_ms), 0);
  assert.equal(remainingRestFraction(900_000, session.target_ms), 0);
  assert.equal(session.duration_ms, 300_000);
  assert.equal(remainingRestFraction(10, 0), 0);
});

function luminance(hex) {
  const channels = hex
    .slice(1)
    .match(/../g)
    .map((part) => parseInt(part, 16) / 255)
    .map((value) => (value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4));
  return channels.reduce((sum, value, index) => sum + value * [0.2126, 0.7152, 0.0722][index], 0);
}

test('series text remains readable above and below every liquid surface in both themes', () => {
  const styles = readFileSync(new URL('../styles/tokens.css', import.meta.url), 'utf8');
  const papers = [...styles.matchAll(/--paper:\s*(#[0-9a-f]+);/gi)].map((match) => match[1]);
  assert.equal(papers.length, 2);
  for (const series of availableSeries)
    for (const dark of [false, true]) {
      const palette = dark ? series.rest.dark : series.rest.light;
      const tokens = restThemeStyle(series.id, dark);
      assert.equal(
        Object.hasOwn(tokens, '--paper'),
        false,
        'Rest must inherit the normal background',
      );
      for (const foreground of [tokens['--ink'], tokens['--muted'], tokens['--faint']]) {
        for (const background of [papers[dark ? 1 : 0], ...palette.waves]) {
          const [a, b] = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
          assert.ok((a + 0.05) / (b + 0.05) >= 4.5, `${series.id}: ${foreground} on ${background}`);
        }
      }
    }
});
