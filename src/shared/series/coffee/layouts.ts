import { createBeanWorld } from './physics.ts';
import { coffeeSeries } from './definition.ts';
export interface BeanPose {
  readonly id: string;
  readonly x: number;
  readonly y: number;
  readonly angle: number;
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
      for (let i = 0; i < budget; i++) {
        if (!world.step()) continue;
        result = Object.freeze({
          poses: Object.freeze(
            world.bodies.map((body) => {
              const depth = world!.foregroundDepth.get(body.label);
              const foreground = depth !== undefined;
              // Project the foreground onto a shallow tabletop. Interior
              // bottom rows follow the jar's curved base, keeping one camera.
              const baseCurve =
                Math.max(0, 1 - ((body.position.x - 240) / 66) ** 2) *
                Math.max(0, Math.min(1, (body.position.y - 180) / 35)) *
                6;
              return Object.freeze({
                id: body.label,
                x: body.position.x,
                y: body.position.y + (foreground ? depth : baseCurve),
                angle: body.angle,
                layer: foreground ? ('front' as const) : ('body' as const),
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
