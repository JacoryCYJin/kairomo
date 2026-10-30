import test from 'node:test';
import assert from 'node:assert/strict';
import { createSSRApp, h } from 'vue';
import { renderToString } from '@vue/server-renderer';
import { themeResolver } from './registry.ts';
import { availableSeries } from './catalog.ts';
import { readFileSync, existsSync } from 'node:fs';
import manifest from '../../../theme-kits/catalog.json' with { type: 'json' };

// Deliberately no beans, containers, liquid, roast tiers or coffee fields.
const makeKit = (id) =>
  Object.fromEntries(
    ['focus', 'rest', 'restIcon', 'collection', 'reward', 'settings'].map((region) => [
      region,
      {
        props: ['state', 'records', 'completedRecords', 'total', 'duration', 'settings'],
        setup(props) {
          return () =>
            h(
              'div',
              { 'data-region': region },
              `${id}:${region}:${props.total ?? props.duration ?? props.records?.length ?? props.state?.phase ?? props.settings?.locale ?? ''}`,
            );
        },
      },
    ]),
  );
test('unregistered test kits independently replace every region and receive real context', async () => {
  const resolve = themeResolver(
    { original: makeKit('original'), scene: makeKit('scene') },
    'original',
  );
  const kit = resolve('scene');
  const records = [
    { id: 'first', duration_ms: 1234 },
    { id: 'second', duration_ms: 5678 },
  ];
  const output = await renderToString(
    createSSRApp({
      render: () =>
        h('main', [
          h(kit.focus, { state: { phase: 'paused' } }),
          h(kit.rest, { state: { phase: 'running' } }),
          h(kit.restIcon),
          h(kit.collection, { records, total: 6912 }),
          h(kit.reward, { duration: 1234 }),
          h(kit.settings, { settings: { locale: 'ja-JP' } }),
        ]),
    }),
  );
  for (const expected of [
    'scene:focus:paused',
    'scene:rest:running',
    'scene:restIcon:',
    'scene:collection:6912',
    'scene:reward:1234',
    'scene:settings:ja-JP',
  ])
    assert.ok(output.includes(expected), expected);
  assert.strictEqual(resolve(undefined), resolve('original'));
  assert.strictEqual(resolve('missing'), resolve('original'));
  assert.strictEqual(resolve('__proto__'), resolve('original'));
  assert.equal(
    availableSeries.some((s) => s.id === 'scene'),
    false,
    'fixture must not leak into product selection',
  );
  assert.equal(records[0].duration_ms, 1234);
});
test('shared catalog has complete names, valid palettes and packaged native assets', () => {
  assert.ok(manifest.themes[manifest.default]);
  assert.deepEqual(
    availableSeries.map((s) => s.id),
    Object.keys(manifest.themes),
  );
  for (const theme of Object.values(manifest.themes)) {
    for (const language of ['zh-CN', 'zh-TW', 'en-US', 'ja-JP']) assert.ok(theme.name[language]);
    for (const mode of ['light', 'dark']) {
      for (const field of ['ink', 'muted', 'selected', 'hover', 'line', 'background'])
        assert.match(theme.rest[mode][field], /^#[0-9a-f]{6}$/i);
      for (const state of ['focus', 'rest', 'overtime']) {
        if (!theme.menu[state]) continue;
        for (const field of ['ink', 'background'])
          assert.match(theme.menu[state][mode][field], /^#[0-9a-f]{6}$/i);
      }
    }
    for (const field of ['focusIcon', 'restIcon', 'overtimeIcon']) {
      if (!theme.menu[field]) continue;
      const icon = new URL('../../../theme-kits/' + theme.menu[field], import.meta.url);
      assert.ok(existsSync(icon));
      assert.equal(readFileSync(icon).subarray(1, 4).toString(), 'PNG');
    }
  }
});
test('feature assembly cannot import coffee geometry, workers or per-theme settings', () => {
  for (const file of [
    '../../features/timer/TimerView.vue',
    '../../features/settings/ThemeSettings.vue',
    '../../features/history/HistoryView.vue',
  ]) {
    const source = readFileSync(new URL(file, import.meta.url), 'utf8');
    assert.doesNotMatch(
      source,
      /from ['"][^'"]*(?:coffee\/|PeriodBeans|RestBackdrop|CoffeeCupIcon)/,
    );
    assert.doesNotMatch(source, /===? ['"]coffee['"]/);
  }
  const native = readFileSync(
    new URL('../../../src-tauri/src/platform/macos/tray.m', import.meta.url),
    'utf8',
  );
  assert.doesNotMatch(native, /191\.0|141\.0|coffeeSeries|tray-rest/);
});

test('coffee options preserve beta.3 storage and use declared defaults', async () => {
  const { themeOptions } = await import('./options.ts');
  assert.deepEqual(themeOptions({ visual_series: 'coffee', bean_range: 'week' }), {
    range: 'week',
  });
  assert.deepEqual(themeOptions({ visual_series: 'coffee' }), { range: 'day' });
});
