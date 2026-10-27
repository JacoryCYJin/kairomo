import { Vec3 } from 'cannon-es';
import { createBeanWorld } from './physics.ts';
import { coffeeSeries } from './definition.ts';
export interface BeanPose {
  readonly id: string;
  readonly x: number;
  readonly y: number;
  readonly angle: number;
  readonly depth: number;
  readonly size: number;
  readonly squash: number;
  readonly inside: boolean;
  readonly layer: 'body' | 'front';
}
export interface BeanLayout {
  readonly poses: readonly BeanPose[];
}
const layouts = new Map<string, BeanLayout>();
const MAX_LAYOUTS = 8;
function remember(key: string, result: BeanLayout) {
  layouts.delete(key);
  layouts.set(key, result);
  while (layouts.size > MAX_LAYOUTS) layouts.delete(layouts.keys().next().value!);
}
export function cachedBeanLayout(ids: readonly string[]) {
  return layouts.get(JSON.stringify(ids.slice(0, coffeeSeries.collection.displayLimit)));
}
export function cacheBeanLayout(ids: readonly string[], result: BeanLayout) {
  remember(JSON.stringify(ids.slice(0, coffeeSeries.collection.displayLimit)), result);
}
/** Publish only the settled result. Nothing in this solver updates the visible drawing. */
export function createBeanLayoutJob(ids: readonly string[]) {
  const selected = ids.slice(0, coffeeSeries.collection.displayLimit);
  const key = JSON.stringify(selected);
  let result = layouts.get(key);
  if (result) remember(key, result);
  let world = result ? undefined : createBeanWorld(selected);
  let disposed = false;
  return {
    get result() {
      return result;
    },
    step(budget = 24): BeanLayout | undefined {
      if (disposed || result || !world) return result;
      const deadline = performance.now() + 6;
      for (let i = 0; i < budget; i++) {
        if (!world.step()) {
          if (performance.now() >= deadline) break;
          continue;
        }
        result = Object.freeze({
          poses: Object.freeze(
            world.bodies.map(({ id, body, size }) => {
              const { x, y, z } = body.position;
              const radiusX = y < 22 ? 54 + (Math.max(0, y) / 22) * 16 : y > 142 ? 64 : 70;
              const radiusZ = y < 22 ? 28 + (Math.max(0, y) / 22) * 8 : y > 142 ? 32 : 36;
              const inside = (x / radiusX) ** 2 + (z / radiusZ) ** 2 <= 1.04;
              const axis = body.quaternion.vmult(new Vec3(0.55, 0.83, 0));
              const normal = body.quaternion.vmult(new Vec3(0, 0, 1));
              return Object.freeze({
                id,
                x: 240 + x,
                y: 226 - y + z * 0.2,
                angle: Math.atan2(-axis.y + axis.z * 0.2, axis.x) - Math.atan2(-0.83, 0.55),
                depth: z,
                size: size * Math.max(0.86, Math.min(1.12, 1 + z / 600)),
                squash: 0.68 + 0.32 * Math.min(1, Math.abs(normal.z + normal.y * 0.2)),
                inside,
                layer: !inside && z > 0 ? ('front' as const) : ('body' as const),
              });
            }),
          ),
        });
        world.dispose();
        world = undefined;
        remember(key, result);
        return result;
      }
      return undefined;
    },
    dispose() {
      disposed = true;
      world?.dispose();
      world = undefined;
    },
  };
}
