import { Body, GSSolver, Plane, Quaternion, SAPBroadphase, Sphere, Vec3, World } from 'cannon-es';
export const PILE_MAX_HEIGHT = 62;
export interface PileBean {
  id: string;
  x: number;
  y: number;
  size: number;
  angle: number;
  depth: number;
  squash: number;
  shadowY: number;
}
export interface PileLayout {
  seed: string;
  poses: PileBean[];
  evaluatedIds: string[];
  capped: boolean;
}
interface SpatialBean {
  id: string;
  body: Body;
  size: number;
}
function randomFor(id: string) {
  let seed = [...id].reduce((h, c) => Math.imul(h ^ c.charCodeAt(0), 16777619), 2166136261);
  return () => {
    seed = Math.imul(seed, 1664525) + 1013904223;
    return (seed >>> 0) / 4294967296;
  };
}
/** Return the last whole, settled pile below the play-button ceiling. */
export function createPileJob(ids: readonly string[], seed: string, width = 360) {
  const world = new World({ gravity: new Vec3(0, -160, 0), allowSleep: true });
  const broadphase = new SAPBroadphase(world);
  broadphase.axisIndex = 1;
  world.broadphase = broadphase;
  const solver = new GSSolver();
  solver.iterations = 24;
  world.solver = solver;
  world.defaultContactMaterial.friction = 0.65;
  world.defaultContactMaterial.restitution = 0;
  const depth = 32;
  function plane(position: Vec3, rotation: Vec3) {
    const body = new Body({ mass: 0, shape: new Plane(), position });
    body.quaternion.setFromEuler(rotation.x, rotation.y, rotation.z);
    world.addBody(body);
  }
  plane(new Vec3(), new Vec3(-Math.PI / 2, 0, 0));
  plane(new Vec3(-width / 2 + 5, 0, 0), new Vec3(0, Math.PI / 2, 0));
  plane(new Vec3(width / 2 - 5, 0, 0), new Vec3(0, -Math.PI / 2, 0));
  plane(new Vec3(0, 0, -depth / 2), new Vec3());
  plane(new Vec3(0, 0, depth / 2), new Vec3(0, Math.PI, 0));
  const bodies: SpatialBean[] = [];
  let result: PileLayout | undefined;
  let disposed = false;
  let quietTicks = 0;
  let accepted: PileBean[] = [];
  let transforms: { position: Vec3; quaternion: Quaternion }[] = [];
  function release() {
    const id = ids[bodies.length];
    if (id === undefined) return;
    const random = randomFor(JSON.stringify([seed, id]));
    const size = 0.97 + random() * 0.06;
    const body = new Body({
      mass: 1,
      linearDamping: 0.8,
      angularDamping: 0.85,
      allowSleep: true,
      sleepSpeedLimit: 3,
      sleepTimeLimit: 0.5,
    });
    const shape = new Sphere(8.2 * size);
    body.addShape(shape, new Vec3(-3 * size, -4.5 * size, 0));
    body.addShape(shape, new Vec3(3 * size, 4.5 * size, 0));
    body.quaternion.copy(
      new Quaternion().setFromEuler(
        random() * Math.PI * 2,
        random() * Math.PI * 2,
        random() * Math.PI * 2,
      ),
    );
    const top = Math.max(0, ...bodies.map(({ body }) => body.position.y + 13));
    body.position.set(
      (random() - 0.5) * (width - 46),
      top + 24 + random() * 3,
      (random() - 0.5) * 6,
    );
    body.velocity.set((random() - 0.5) * 8, -(1 + random() * 3), (random() - 0.5) * 3);
    world.addBody(body);
    bodies.push({ id, body, size });
    quietTicks = 0;
  }
  function project(): PileBean[] {
    return bodies
      .map(({ id, body, size }) => {
        const { x, y, z } = body.position;
        const normalizedDepth = (z + depth / 2) / depth;
        const perspective = 0.9 + normalizedDepth * 0.1;
        const axis = body.quaternion.vmult(new Vec3(0.55, 0.83, 0));
        const normal = body.quaternion.vmult(new Vec3(0, 0, 1));
        const shadowY = 5 + (1 - normalizedDepth) * 7;
        return {
          id,
          x: width / 2 + x * perspective,
          y: shadowY + y * 0.88,
          size: 42 * size * perspective,
          angle:
            ((Math.atan2(-axis.y + axis.z * 0.12, axis.x) - Math.atan2(-0.83, 0.55)) * 180) /
            Math.PI,
          depth: z,
          squash: 0.7 + 0.3 * Math.min(1, Math.abs(normal.z + normal.y * 0.12)),
          shadowY,
        };
      })
      .sort((a, b) => a.depth - b.depth);
  }
  release();
  return {
    bodies,
    step(steps = 24) {
      if (disposed || result) return result;
      for (let tick = 0; tick < steps; tick++) {
        world.step(1 / 90);
        const quiet = bodies.every(
          ({ body }) =>
            body.velocity.lengthSquared() < 25 && body.angularVelocity.lengthSquared() < 1,
        );
        quietTicks = quiet ? quietTicks + 1 : 0;
        if (!bodies.every(({ body }) => body.sleepState === Body.SLEEPING) && quietTicks < 30)
          continue;
        for (const { body } of bodies) body.sleep();
        const candidate = project();
        // 0.44 bounds the rotated original SVG outline, not only its collision solid.
        if (candidate.some((p) => p.y + p.size * 0.44 > PILE_MAX_HEIGHT)) {
          const evaluatedIds = bodies.map((b) => b.id);
          const rejected = bodies.pop()!;
          world.removeBody(rejected.body);
          // Publish the previous supported state whole, never clip a taller pile.
          bodies.forEach(({ body }, index) => {
            body.position.copy(transforms[index]!.position);
            body.quaternion.copy(transforms[index]!.quaternion);
            body.sleep();
          });
          result = { seed, poses: accepted, evaluatedIds, capped: true };
          return result;
        }
        accepted = candidate;
        transforms = bodies.map(({ body }) => ({
          position: body.position.clone(),
          quaternion: body.quaternion.clone(),
        }));
        if (bodies.length === ids.length) {
          result = { seed, poses: accepted, evaluatedIds: bodies.map((b) => b.id), capped: false };
          return result;
        }
        release();
      }
      return undefined;
    },
    dispose() {
      disposed = true;
      for (const body of [...world.bodies]) world.removeBody(body);
    },
  };
}
