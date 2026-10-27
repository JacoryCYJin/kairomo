export interface Point {
  x: number;
  y: number;
}
const quadratic = (a: Point, control: Point, b: Point, steps: number): Point[] =>
  Array.from({ length: steps }, (_, index) => {
    const t = (index + 1) / steps;
    return {
      x: (1 - t) ** 2 * a.x + 2 * (1 - t) * t * control.x + t ** 2 * b.x,
      y: (1 - t) ** 2 * a.y + 2 * (1 - t) * t * control.y + t ** 2 * b.y,
    };
  });
// The visible glass and collision segments share the same open contour.
export const jarContour: readonly Point[] = [
  { x: 176, y: 40 },
  { x: 176, y: 55 },
  ...quadratic({ x: 176, y: 55 }, { x: 176, y: 62 }, { x: 172, y: 70 }, 4),
  ...quadratic({ x: 172, y: 70 }, { x: 170, y: 76 }, { x: 170, y: 85 }, 4),
  { x: 170, y: 204 },
  ...quadratic({ x: 170, y: 204 }, { x: 170, y: 220 }, { x: 186, y: 226 }, 8),
  ...quadratic({ x: 186, y: 226 }, { x: 240, y: 240 }, { x: 294, y: 226 }, 20),
  ...quadratic({ x: 294, y: 226 }, { x: 310, y: 220 }, { x: 310, y: 204 }, 8),
  { x: 310, y: 85 },
  ...quadratic({ x: 310, y: 85 }, { x: 310, y: 76 }, { x: 308, y: 70 }, 4),
  ...quadratic({ x: 308, y: 70 }, { x: 304, y: 62 }, { x: 304, y: 55 }, 4),
  { x: 304, y: 40 },
];
export const jarPath = jarContour.map((p, i) => `${i ? 'L' : 'M'}${p.x} ${p.y}`).join('');
