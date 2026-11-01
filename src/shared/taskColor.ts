export const taskColors = [
  '#3e63dd',
  '#12a594',
  '#30a46c',
  '#e2a336',
  '#e8654c',
  '#d64545',
  '#8e4ec6',
  '#8c7a6b',
];

function linear(value: number) {
  const channel = value / 255;
  return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
}

function luminance(hex: string) {
  const value = hex.replace('#', '');
  return (
    0.2126 * linear(parseInt(value.slice(0, 2), 16)) +
    0.7152 * linear(parseInt(value.slice(2, 4), 16)) +
    0.0722 * linear(parseInt(value.slice(4, 6), 16))
  );
}

function contrast(a: number, b: number) {
  const light = Math.max(a, b);
  const dark = Math.min(a, b);
  return (light + 0.05) / (dark + 0.05);
}

export interface Hsv {
  h: number;
  s: number;
  v: number;
}

export function hexToHsv(hex: string): Hsv {
  const value = hex.replace('#', '');
  const red = parseInt(value.slice(0, 2), 16) / 255;
  const green = parseInt(value.slice(2, 4), 16) / 255;
  const blue = parseInt(value.slice(4, 6), 16) / 255;
  const max = Math.max(red, green, blue);
  const min = Math.min(red, green, blue);
  const delta = max - min;
  let hue = 0;
  if (delta) {
    if (max === red) hue = ((green - blue) / delta) % 6;
    else if (max === green) hue = (blue - red) / delta + 2;
    else hue = (red - green) / delta + 4;
    hue *= 60;
    if (hue < 0) hue += 360;
  }
  return { h: hue, s: max === 0 ? 0 : delta / max, v: max };
}

export function hsvToHex(h: number, s: number, v: number) {
  const chroma = v * s;
  const second = chroma * (1 - Math.abs(((h / 60) % 2) - 1));
  const offset = v - chroma;
  const [red, green, blue] =
    h < 60
      ? [chroma, second, 0]
      : h < 120
        ? [second, chroma, 0]
        : h < 180
          ? [0, chroma, second]
          : h < 240
            ? [0, second, chroma]
            : h < 300
              ? [second, 0, chroma]
              : [chroma, 0, second];
  return (
    '#' +
    [red, green, blue]
      .map((channel) =>
        Math.round((channel + offset) * 255)
          .toString(16)
          .padStart(2, '0'),
      )
      .join('')
  );
}

export function taskInk(hex: string) {
  const value = luminance(hex);
  return contrast(value, 1) >= contrast(value, 0) ? '#ffffff' : '#000000';
}
