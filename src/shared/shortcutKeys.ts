import type { AppLocale } from './i18n';
import type { ShortcutBinding, ShortcutBindings, ShortcutModifier } from './ipc/types';

export const defaultShortcutBindings: ShortcutBindings = {
  toggle: { modifiers: ['control', 'alt'], key: 'KeyK' },
  pause: { modifiers: ['control', 'alt'], key: 'Enter' },
  toggle_enabled: true,
  pause_enabled: true,
};

const modifierOrder: ShortcutModifier[] = ['control', 'alt', 'shift', 'meta'];
const specialKeys = new Set([
  'Space',
  'Enter',
  'Tab',
  'Backspace',
  'Delete',
  'Insert',
  'Home',
  'End',
  'PageUp',
  'PageDown',
  'ArrowUp',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'Backquote',
  'Minus',
  'Equal',
  'BracketLeft',
  'BracketRight',
  'Backslash',
  'Semicolon',
  'Quote',
  'Comma',
  'Period',
  'Slash',
  ...Array.from({ length: 10 }, (_, index) => `Numpad${index}`),
  'NumpadAdd',
  'NumpadSubtract',
  'NumpadMultiply',
  'NumpadDivide',
  'NumpadDecimal',
  'NumpadEnter',
  'NumpadEqual',
]);

const modifierCodes = new Set([
  'ControlLeft',
  'ControlRight',
  'AltLeft',
  'AltRight',
  'ShiftLeft',
  'ShiftRight',
  'MetaLeft',
  'MetaRight',
]);

const keyLabels: Record<AppLocale, Record<string, string>> = {
  'zh-CN': {
    Space: '空格',
    Enter: '回车',
    Tab: 'Tab',
    Backspace: '退格',
    Delete: '删除',
    Insert: '插入',
    Home: 'Home',
    End: 'End',
    PageUp: 'Page Up',
    PageDown: 'Page Down',
    ArrowUp: '上方向键',
    ArrowDown: '下方向键',
    ArrowLeft: '左方向键',
    ArrowRight: '右方向键',
    Backquote: '`',
    Minus: '-',
    Equal: '=',
    BracketLeft: '[',
    BracketRight: ']',
    Backslash: '\\',
    Semicolon: ';',
    Quote: "'",
    Comma: ',',
    Period: '.',
    Slash: '/',
    NumpadAdd: '小键盘 +',
    NumpadSubtract: '小键盘 -',
    NumpadMultiply: '小键盘 ×',
    NumpadDivide: '小键盘 ÷',
    NumpadDecimal: '小键盘 .',
    NumpadEnter: '小键盘回车',
    NumpadEqual: '小键盘 =',
  },
  'zh-TW': {
    Space: '空格',
    Enter: '換行',
    Tab: 'Tab',
    Backspace: '退格',
    Delete: '刪除',
    Insert: '插入',
    Home: 'Home',
    End: 'End',
    PageUp: 'Page Up',
    PageDown: 'Page Down',
    ArrowUp: '上方向鍵',
    ArrowDown: '下方向鍵',
    ArrowLeft: '左方向鍵',
    ArrowRight: '右方向鍵',
    Backquote: '`',
    Minus: '-',
    Equal: '=',
    BracketLeft: '[',
    BracketRight: ']',
    Backslash: '\\',
    Semicolon: ';',
    Quote: "'",
    Comma: ',',
    Period: '.',
    Slash: '/',
    NumpadAdd: '小鍵盤 +',
    NumpadSubtract: '小鍵盤 -',
    NumpadMultiply: '小鍵盤 ×',
    NumpadDivide: '小鍵盤 ÷',
    NumpadDecimal: '小鍵盤 .',
    NumpadEnter: '小鍵盤換行',
    NumpadEqual: '小鍵盤 =',
  },
  'en-US': {
    Space: 'Space',
    Enter: 'Enter',
    Tab: 'Tab',
    Backspace: 'Backspace',
    Delete: 'Delete',
    Insert: 'Insert',
    Home: 'Home',
    End: 'End',
    PageUp: 'Page Up',
    PageDown: 'Page Down',
    ArrowUp: 'Up Arrow',
    ArrowDown: 'Down Arrow',
    ArrowLeft: 'Left Arrow',
    ArrowRight: 'Right Arrow',
    Backquote: '`',
    Minus: '-',
    Equal: '=',
    BracketLeft: '[',
    BracketRight: ']',
    Backslash: '\\',
    Semicolon: ';',
    Quote: "'",
    Comma: ',',
    Period: '.',
    Slash: '/',
    NumpadAdd: 'Numpad +',
    NumpadSubtract: 'Numpad -',
    NumpadMultiply: 'Numpad ×',
    NumpadDivide: 'Numpad ÷',
    NumpadDecimal: 'Numpad .',
    NumpadEnter: 'Numpad Enter',
    NumpadEqual: 'Numpad =',
  },
  'ja-JP': {
    Space: 'スペース',
    Enter: 'リターン',
    Tab: 'Tab',
    Backspace: 'バックスペース',
    Delete: 'デリート',
    Insert: 'インサート',
    Home: 'Home',
    End: 'End',
    PageUp: 'Page Up',
    PageDown: 'Page Down',
    ArrowUp: '上矢印',
    ArrowDown: '下矢印',
    ArrowLeft: '左矢印',
    ArrowRight: '右矢印',
    Backquote: '`',
    Minus: '-',
    Equal: '=',
    BracketLeft: '[',
    BracketRight: ']',
    Backslash: '\\',
    Semicolon: ';',
    Quote: "'",
    Comma: ',',
    Period: '.',
    Slash: '/',
    NumpadAdd: 'テンキー +',
    NumpadSubtract: 'テンキー -',
    NumpadMultiply: 'テンキー ×',
    NumpadDivide: 'テンキー ÷',
    NumpadDecimal: 'テンキー .',
    NumpadEnter: 'テンキー Enter',
    NumpadEqual: 'テンキー =',
  },
};

export function normalizeBinding(binding: ShortcutBinding): ShortcutBinding {
  const modifiers = [...new Set(binding.modifiers)].sort(
    (a, b) => modifierOrder.indexOf(a) - modifierOrder.indexOf(b),
  );
  return { modifiers, key: binding.key };
}

export function bindingEquals(a: ShortcutBinding, b: ShortcutBinding): boolean {
  const left = normalizeBinding(a);
  const right = normalizeBinding(b);
  return (
    left.key === right.key &&
    left.modifiers.length === right.modifiers.length &&
    left.modifiers.every((modifier, index) => modifier === right.modifiers[index])
  );
}

export function isModifierCode(code: string): boolean {
  return modifierCodes.has(code);
}

export function modifiersFromEvent(event: KeyboardEvent): ShortcutModifier[] {
  const modifiers: ShortcutModifier[] = [];
  if (event.ctrlKey) modifiers.push('control');
  if (event.altKey) modifiers.push('alt');
  if (event.shiftKey) modifiers.push('shift');
  if (event.metaKey) modifiers.push('meta');
  return modifiers;
}

export function isSupportedKeyCode(code: string): boolean {
  return (
    /^Key[A-Z]$/.test(code) ||
    /^Digit[0-9]$/.test(code) ||
    /^F([1-9]|1[0-9]|2[0-4])$/.test(code) ||
    specialKeys.has(code)
  );
}

export function bindingFromKeyboardEvent(
  event: KeyboardEvent,
):
  | { binding: ShortcutBinding; error?: undefined }
  | { binding?: undefined; error: 'modifier_required' | 'unsupported_key' } {
  const modifiers = modifiersFromEvent(event);
  if (!modifiers.some((modifier) => modifier !== 'shift')) {
    return { error: 'modifier_required' };
  }
  if (!isSupportedKeyCode(event.code)) {
    return { error: 'unsupported_key' };
  }
  return {
    binding: normalizeBinding({
      modifiers,
      key: event.code,
    }),
  };
}

export function modifierLabel(modifier: ShortcutModifier, platform: 'macos' | 'other'): string {
  switch (modifier) {
    case 'control':
      return platform === 'macos' ? 'Control' : 'Ctrl';
    case 'alt':
      return platform === 'macos' ? 'Option' : 'Alt';
    case 'shift':
      return 'Shift';
    case 'meta':
      return platform === 'macos' ? 'Command' : 'Win';
  }
}

export function keyLabel(code: string, locale: AppLocale): string {
  const named = keyLabels[locale][code];
  if (named) return named;
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  const numpad = /^Numpad([0-9])$/.exec(code);
  if (numpad)
    return `${locale === 'zh-CN' || locale === 'zh-TW' ? '小键盘' : 'Numpad'} ${numpad[1]}`;
  return code;
}

export function bindingKeycaps(
  binding: ShortcutBinding,
  platform: 'macos' | 'other',
  locale: AppLocale,
): string[] {
  const normalized = normalizeBinding(binding);
  return [
    ...normalized.modifiers.map((modifier) => modifierLabel(modifier, platform)),
    ...(normalized.key ? [keyLabel(normalized.key, locale)] : []),
  ];
}

export function bindingText(
  binding: ShortcutBinding,
  platform: 'macos' | 'other',
  locale: AppLocale,
): string {
  return bindingKeycaps(binding, platform, locale).join(' + ');
}
