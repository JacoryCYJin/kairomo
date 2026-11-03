import test from 'node:test';
import assert from 'node:assert/strict';
import {
  bindingEquals,
  bindingFromKeyboardEvent,
  bindingKeycaps,
  bindingText,
  defaultShortcutBindings,
  isSupportedKeyCode,
  keyLabel,
  modifierLabel,
  normalizeBinding,
} from './shortcutKeys.ts';

function event(code, modifiers = {}, repeat = false) {
  return {
    code,
    repeat,
    ctrlKey: !!modifiers.control,
    altKey: !!modifiers.alt,
    shiftKey: !!modifiers.shift,
    metaKey: !!modifiers.meta,
  };
}

test('macOS keycaps use readable modifier and key names instead of symbols', () => {
  assert.deepEqual(bindingKeycaps(defaultShortcutBindings.toggle, 'macos', 'zh-CN'), [
    'Control',
    'Option',
    'K',
  ]);
  assert.deepEqual(bindingKeycaps(defaultShortcutBindings.pause, 'macos', 'zh-CN'), [
    'Control',
    'Option',
    '回车',
  ]);
  assert.equal(
    bindingText({ modifiers: ['meta', 'shift'], key: 'KeyK' }, 'macos', 'en-US'),
    'Shift + Command + K',
  );
  assert.equal(modifierLabel('alt', 'macos'), 'Option');
  assert.equal(modifierLabel('alt', 'other'), 'Alt');
  assert.equal(keyLabel('ArrowUp', 'ja-JP'), '上矢印');
});

test('recorded bindings require a non-Shift modifier and a supported key', () => {
  assert.deepEqual(bindingFromKeyboardEvent(event('KeyK', { control: true, shift: true })), {
    binding: {
      modifiers: ['control', 'shift'],
      key: 'KeyK',
    },
  });
  assert.deepEqual(bindingFromKeyboardEvent(event('KeyK', { shift: true })), {
    error: 'modifier_required',
  });
  assert.deepEqual(bindingFromKeyboardEvent(event('Escape', { control: true })), {
    error: 'unsupported_key',
  });
  assert.equal(isSupportedKeyCode('KeyA'), true);
  assert.equal(isSupportedKeyCode('Digit9'), true);
  assert.equal(isSupportedKeyCode('F24'), true);
  assert.equal(isSupportedKeyCode('Escape'), false);
  assert.equal(isSupportedKeyCode('Numpad5'), true);
});

test('binding comparison ignores modifier order and normalized keycaps stay stable', () => {
  const first = { modifiers: ['alt', 'control'], key: 'Space' };
  const second = { modifiers: ['control', 'alt'], key: 'Space' };
  assert.equal(bindingEquals(first, second), true);
  assert.deepEqual(normalizeBinding(first).modifiers, ['control', 'alt']);
  assert.equal(bindingEquals(second, { modifiers: ['control'], key: 'Space' }), false);
});
