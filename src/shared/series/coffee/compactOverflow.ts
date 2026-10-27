import type { BeanPose } from './layouts';

// Angles describe the visible major axis, rather than the diagonal source icon.
interface Slot {
  x: number;
  y: number;
  angle: number;
  size: number;
  thickness: number;
  order: number;
}
const slots: Slot[] = [
  { x: 155, y: 225, angle: -12, size: 1.03, thickness: 1.18, order: 1 },
  { x: 165, y: 221, angle: 48, size: 1.02, thickness: 1.18, order: 2 },
  { x: 177, y: 234, angle: -16, size: 1.12, thickness: 1.28, order: 12 },
  { x: 320, y: 216, angle: -68, size: 1.02, thickness: 1.16, order: 3 },
  { x: 339, y: 222, angle: 20, size: 1.05, thickness: 1.18, order: 4 },
  { x: 353, y: 234, angle: 22, size: 1.03, thickness: 1.2, order: 8 },
  { x: 333, y: 237, angle: 18, size: 1.08, thickness: 1.25, order: 9 },
  { x: 310, y: 237, angle: -34, size: 1.2, thickness: 1.35, order: 15 },
  { x: 285, y: 237, angle: -4, size: 1.08, thickness: 1.18, order: 10 },
  { x: 271, y: 226, angle: 38, size: 1.02, thickness: 1.15, order: 6 },
  { x: 246, y: 230, angle: 38, size: 1.04, thickness: 1.2, order: 11 },
  { x: 220, y: 228, angle: 0, size: 1.07, thickness: 1.18, order: 7 },
  { x: 202, y: 220, angle: -18, size: 1.0, thickness: 1.15, order: 5 },
];
export function compactOverflow(poses: readonly BeanPose[]) {
  const exterior = poses.filter((p) => !p.inside).sort((a, b) => a.id.localeCompare(b.id));
  // Fill the connected right cluster first, then its foreground and the small left cluster.
  const fillOrder = [0, 1, 3, 4, 2, 6, 11, 7, 5, 10, 8, 9, 12];
  const composed: BeanPose[] = exterior.map((pose, i) => {
    const slot = slots[fillOrder[i % fillOrder.length]!]!;
    const tier = Math.floor(i / fillOrder.length);
    return {
      ...pose,
      x: slot.x + (tier % 2 ? 5 : -3) * tier,
      y: slot.y - tier * 10,
      angle: (slot.angle * Math.PI) / 180,
      size: slot.size,
      squash: slot.thickness,
      depth: 1000 + slot.order - tier * 20,
      layer: 'front',
    };
  });
  const shadows = composed
    .filter((p) => p.y >= 224)
    .map((p) => ({ id: p.id, x: p.x, y: p.y + 9, rx: 14 * p.size, ry: 2.2 }));
  return { poses: [...poses.filter((p) => p.inside), ...composed], shadows };
}
