/** Display-only rules. Neither route changes authority, records, or timer targets. */
export const routeSegmentMs = 30 * 60_000;
export const routeSegments = 8;
function nonNegative(value: number) {
  return Number.isFinite(value) ? Math.max(0, value) : 0;
}
export function collectionProgress(total: number) {
  return Math.min(routeSegments, Math.floor(nonNegative(total) / routeSegmentMs)) / routeSegments;
}
export function focusProgress(duration: number, target: number) {
  return target > 0 && Number.isFinite(target) ? Math.min(1, nonNegative(duration) / target) : 0;
}
/** Coordinates on the visible snow path, bottom to summit. */
export const ascentRoute = [
  [240, 1005],
  [640, 826],
  [780, 706],
  [650, 591],
  [799, 448],
  [680, 320],
  [783, 235],
  [733, 179],
  [755, 112],
] as const;
export function routePosition(progress: number) {
  const offset = Math.min(1, nonNegative(progress)) * routeSegments;
  const index = Math.min(routeSegments - 1, Math.floor(offset));
  const weight = offset - index;
  const a = ascentRoute[index],
    b = ascentRoute[index + 1];
  return {
    x: Math.round(a[0] + (b[0] - a[0]) * weight),
    y: Math.round(a[1] + (b[1] - a[1]) * weight),
  };
}

/** Feet follow the snow track in the extracted focus artwork, in asset coordinates. */
export function slopePosition(progress: number) {
  const points = [
    [800, 700],
    [1050, 655],
    [1295, 592],
    [1500, 520],
    [1690, 455],
    [1810, 365],
  ];
  const offset = Math.min(1, nonNegative(progress)) * (points.length - 1);
  const index = Math.min(points.length - 2, Math.floor(offset));
  const weight = offset - index;
  return {
    x: Math.round(points[index][0] + (points[index + 1][0] - points[index][0]) * weight),
    y: Math.round(points[index][1] + (points[index + 1][1] - points[index][1]) * weight),
  };
}
