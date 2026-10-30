import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { coffeeSeries } from './coffee/definition.ts';
import { availableSeries } from './catalog.ts';
import {
  dialogConfirmStyle,
  remainingRestFraction,
  restThemeStyle,
  visualThemeStyle,
} from './rest.ts';

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

test('every kit provides readable dialog confirmations in light and dark appearances', () => {
  for (const series of availableSeries) {
    for (const dark of [false, true]) {
      const tokens = dialogConfirmStyle(series.id, dark);
      const [a, b] = [tokens['--dialog-confirm'], tokens['--dialog-confirm-ink']]
        .map(luminance)
        .sort((a, b) => b - a);
      assert.ok((a + 0.05) / (b + 0.05) >= 4.5, `${series.id}, dark=${dark}`);
    }
  }
});

test('series text remains readable above and below every liquid surface in both themes', () => {
  const styles = readFileSync(new URL('../styles/tokens.css', import.meta.url), 'utf8');
  const papers = [...styles.matchAll(/--paper:\s*(#[0-9a-f]+);/gi)].map((match) => match[1]);
  assert.equal(papers.length, 2);
  for (const series of [coffeeSeries])
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

test('optional focus palette is local, independent of rest and needs no coffee rendering', () => {
  const light = {
    ink: '#123456',
    muted: '#234567',
    selected: '#345678',
    hover: '#456789',
    line: '#567890',
    background: '#ffffff',
  };
  const dark = { ...light, ink: '#abcdef' };
  const fixture = {
    id: 'fixture',
    name: {},
    focus: { light, dark },
    rest: { light: dark, dark: light },
  };
  assert.equal(visualThemeStyle(fixture, false, false)['--ink'], '#123456');
  assert.equal(visualThemeStyle(fixture, true, false)['--ink'], '#abcdef');
  assert.equal(visualThemeStyle(fixture, false, true)['--ink'], '#abcdef');
  assert.equal(visualThemeStyle(coffeeSeries, false, false), undefined);
});
