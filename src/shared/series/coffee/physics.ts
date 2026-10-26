import Matter from 'matter-js';
import { coffeeSeries } from './definition.ts';

export const SCENE_WIDTH = 480;
export const SCENE_HEIGHT = 260;
export const BEAN_SIZE = 28;
const silhouette = Array.from({ length: 14 }, (_, i) => {
  const a = (i / 14) * Math.PI * 2,
    x = Math.cos(a) * 7.6,
    y = Math.sin(a) * 10.3;
  return { x: x * Math.cos(0.5) - y * Math.sin(0.5), y: x * Math.sin(0.5) + y * Math.cos(0.5) };
});
function randomFor(id: string) {
  let seed = 2166136261;
  for (const c of id) seed = Math.imul(seed ^ c.charCodeAt(0), 16777619);
  return () => {
    seed = (Math.imul(seed, 1664525) + 1013904223) | 0;
    return (seed >>> 0) / 4294967296;
  };
}
function wall(x1: number, y1: number, x2: number, y2: number) {
  return Matter.Bodies.rectangle((x1 + x2) / 2, (y1 + y2) / 2, Math.hypot(x2 - x1, y2 - y1), 4, {
    isStatic: true,
    angle: Math.atan2(y2 - y1, x2 - x1),
    friction: 0.8,
    slop: 0.01,
  });
}
export function createBeanWorld(ids: readonly string[]) {
  const engine = Matter.Engine.create({
    enableSleeping: true,
    positionIterations: 16,
    velocityIterations: 8,
  });
  // A wide-mouth jar on an open tabletop. Exterior space has no vertical walls:
  // overflow must find its own angle of repose, rather than fill side channels.
  const walls = [
    wall(176, 40, 176, 58),
    wall(176, 58, 170, 78),
    wall(170, 78, 170, 214),
    wall(170, 214, 182, 228),
    Matter.Bodies.rectangle(240, 239, 116, 26, { isStatic: true, friction: 0.45 }),
    wall(298, 228, 310, 214),
    wall(310, 214, 310, 78),
    wall(310, 78, 304, 58),
    wall(304, 58, 304, 40),
    Matter.Bodies.rectangle(240, 252, 1200, 44, { isStatic: true, friction: 0.6 }),
    // A second depth plane lets foreground beans settle in front of the jar,
    // rather than collide with its side-view silhouette. It is never drawn.
    Matter.Bodies.rectangle(240, 268, 1200, 44, {
      isStatic: true,
      friction: 0.6,
      collisionFilter: { category: 2, mask: 2 },
    }),
  ];
  // Fixed bean area and vessel geometry target roughly 80 beans inside the jar.
  const scale = Math.sqrt(80 / coffeeSeries.collection.insideCapacity);
  const bodies = ids.slice(0, coffeeSeries.collection.displayLimit).map((id, index) => {
    const random = randomFor(id);
    const body = Matter.Bodies.fromVertices(
      240 + (random() - 0.5) * 72,
      -20 - index * 28,
      [silhouette],
      {
        label: id,
        friction: 0.55,
        frictionStatic: 0.7,
        restitution: 0.05,
        frictionAir: 0.015,
        sleepThreshold: 45,
        collisionFilter: { category: 1, mask: 1 },
      },
    );
    Matter.Body.scale(body, scale, scale);
    Matter.Body.setAngle(body, (random() - 0.5) * Math.PI);
    return body;
  });
  // Capacity is only a warm-start estimate. Settled coverage at the mouth
  // decides when overflow is permitted; a record index never does.
  const initialCount = Math.min(bodies.length, coffeeSeries.collection.insideCapacity);
  const active = bodies.slice(0, initialCount);
  Matter.Composite.add(engine.world, [...walls, ...active]);
  const mouthLanes = [188, 208, 228, 248, 268, 288];
  const foregroundDepth = new Map<string, number>();
  let released = initialCount,
    overflowCount = 0,
    releaseTicks = 0,
    spilling = false,
    disposed = false;
  function surface() {
    return mouthLanes.map((x) => ({
      x,
      top: Math.min(
        226,
        ...active
          .filter(
            (body) =>
              body.position.x >= 176 &&
              body.position.x <= 304 &&
              body.bounds.min.x <= x + 5 &&
              body.bounds.max.x >= x - 5,
          )
          .map((body) => body.bounds.min.y),
      ),
    }));
  }
  return {
    bodies,
    foregroundDepth,
    step() {
      if (!disposed) {
        if (released < bodies.length) {
          const settled = active.every((body) => body.isSleeping);
          if (!spilling && settled) {
            const lanes = surface();
            const lowest = lanes.reduce((a, b) => (a.top > b.top ? a : b));
            spilling = lowest.top <= 40;
            if (!spilling) {
              const next = bodies[released++]!;
              Matter.Body.setPosition(next, { x: lowest.x, y: 18 });
              active.push(next);
              Matter.Composite.add(engine.world, next);
            }
          }
          if (spilling && releaseTicks++ % 10 === 0) {
            const next = bodies[released++]!;
            const random = randomFor(next.label);
            const direction = overflowCount++ % 3;
            const front = direction === 2;
            Matter.Body.setPosition(next, {
              x: front
                ? 240 + (random() - 0.5) * 100
                : (direction === 0 ? 148 : 332) + (random() - 0.5) * 44,
              y: front ? 154 : 26,
            });
            if (front) {
              next.collisionFilter = { category: 2, mask: 2, group: 0 };
              foregroundDepth.set(next.label, random() * 8);
            }
            next.friction = 0.35;
            next.frictionAir = 0.06;
            active.push(next);
            Matter.Composite.add(engine.world, next);
          }
        }
        Matter.Engine.update(engine, 1000 / 60);
      }
      return released === bodies.length && active.every((body) => body.isSleeping);
    },
    dispose() {
      disposed = true;
      Matter.Composite.clear(engine.world, false);
      Matter.Engine.clear(engine);
    },
  };
}
