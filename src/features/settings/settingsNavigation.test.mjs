import test from 'node:test';
import assert from 'node:assert/strict';
import { settingsNavigation } from './settingsNavigation.ts';
test('nested phrase pages restore their own parent positions without overwriting the settings position', () => {
  const nav = settingsNavigation();
  assert.deepEqual(nav.navigate('phrase-library', 360), { page: 'phrase-library', scrollTop: 0 });
  assert.equal(nav.navigate('phrase-editor', 840).scrollTop, 0);
  assert.equal(nav.navigate('phrase-library', 150).scrollTop, 840);
  assert.equal(nav.navigate('settings', 780).scrollTop, 360);
  assert.equal(nav.navigate('task-manager', 410).scrollTop, 0);
  assert.equal(nav.navigate('settings', 230).scrollTop, 410);
  assert.equal(nav.navigate('shortcut-manager', 510).scrollTop, 0);
  assert.equal(nav.navigate('settings', 240).scrollTop, 510);
});
test('new child visits start at the top and repeated notifications do not reset the current page', () => {
  const nav = settingsNavigation();
  nav.navigate('phrase-library', 120);
  nav.navigate('phrase-editor', 500);
  nav.navigate('phrase-library', 220);
  assert.equal(nav.navigate('phrase-editor', 460).scrollTop, 0);
  assert.equal(nav.navigate('phrase-editor', 90).scrollTop, 90);
  assert.equal(nav.navigate('phrase-library', 100).scrollTop, 460);
  nav.navigate('settings', 460);
  assert.equal(nav.navigate('phrase-library', 120).scrollTop, 0);
});
