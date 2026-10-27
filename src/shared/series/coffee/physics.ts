import {
  Body,
  ConvexPolyhedron,
  GSSolver,
  Plane,
  Quaternion,
  SAPBroadphase,
  Sphere,
  Vec3,
  World,
} from 'cannon-es';
import { coffeeSeries } from './definition.ts';

export interface SpatialBean {
  readonly id: string;
  readonly body: Body;
  readonly size: number;
}
function randomFor(id: string) {
  let seed = 2166136261;
  for (const c of id) seed = Math.imul(seed ^ c.charCodeAt(0), 16777619);
  return () => {
    seed = Math.imul(seed, 1664525) + 1013904223;
    return (seed >>> 0) / 4294967296;
  };
}
// The oval vessel has front/back occupancy; the illustration remains SVG.
function glass() {
  const rings = [
    { y: 0, x: 54, z: 28 },
    { y: 8, x: 65, z: 33 },
    { y: 22, x: 70, z: 36 },
    { y: 142, x: 70, z: 36 },
    { y: 158, x: 67, z: 34 },
    { y: 172, x: 64, z: 32 },
    { y: 190, x: 64, z: 32 },
  ];
  const panels: Body[] = [];
  const segments = 24;
  for (let row = 0; row < rings.length - 1; row++)
    for (let i = 0; i < segments; i++) {
      const a = (i * Math.PI * 2) / segments,
        b = ((i + 1) * Math.PI * 2) / segments;
      const low = rings[row]!,
        high = rings[row + 1]!;
      const vertices = [
        new Vec3(Math.cos(a) * low.x, low.y, Math.sin(a) * low.z),
        new Vec3(Math.cos(b) * low.x, low.y, Math.sin(b) * low.z),
        new Vec3(Math.cos(a) * high.x, high.y, Math.sin(a) * high.z),
        new Vec3(Math.cos(b) * high.x, high.y, Math.sin(b) * high.z),
      ];
      vertices.push(
        ...vertices.map(
          (v) => new Vec3(v.x + Math.cos((a + b) / 2) * 4, v.y, v.z + Math.sin((a + b) / 2) * 4),
        ),
      );
      const center = vertices.reduce((sum, v) => sum.vadd(v), new Vec3()).scale(1 / 8);
      for (const vertex of vertices) vertex.vsub(center, vertex);
      const faces = [
        [0, 2, 3, 1],
        [4, 5, 7, 6],
        [0, 1, 5, 4],
        [2, 6, 7, 3],
        [0, 4, 6, 2],
        [1, 3, 7, 5],
      ];
      for (const face of faces) {
        const p = vertices[face[0]]!,
          q = vertices[face[1]]!,
          r = vertices[face[2]]!;
        if (q.vsub(p).cross(r.vsub(p)).dot(p) < 0) face.reverse();
      }
      panels.push(
        new Body({ mass: 0, shape: new ConvexPolyhedron({ vertices, faces }), position: center }),
      );
    }
  return panels;
}
export function createBeanWorld(ids: readonly string[]) {
  const world = new World({ gravity: new Vec3(0, -800, 0), allowSleep: true });
  world.broadphase = new SAPBroadphase(world);
  const solver = new GSSolver();
  solver.iterations = 20;
  solver.tolerance = 0.001;
  world.solver = solver;
  world.defaultContactMaterial.friction = 0.55;
  world.defaultContactMaterial.restitution = 0.02;
  world.defaultContactMaterial.contactEquationStiffness = 1e7;
  const floor = new Body({ mass: 0, shape: new Plane() });
  floor.quaternion.setFromEuler(-Math.PI / 2, 0, 0);
  world.addBody(floor);
  for (const panel of glass()) world.addBody(panel);
  const bodies: SpatialBean[] = ids.slice(0, coffeeSeries.collection.displayLimit).map((id) => {
    const random = randomFor(id);
    const size = 0.94 + random() * 0.12;
    const body = new Body({
      mass: 1,
      linearDamping: 0.45,
      angularDamping: 0.6,
      allowSleep: true,
      sleepSpeedLimit: 3,
      sleepTimeLimit: 0.7,
    });
    // Rounded elongated solids all share one collision world, inside and out.
    const shape = new Sphere(10.8 * size);
    body.addShape(shape, new Vec3(-2.8 * size, -4.2 * size, 0));
    body.addShape(shape, new Vec3(2.8 * size, 4.2 * size, 0));
    body.quaternion.copy(
      new Quaternion().setFromEuler(random() * 2, random() * 2, random() * Math.PI * 2),
    );
    return { id, body, size };
  });
  let released = 0,
    ticks = 0,
    finished = false,
    disposed = false;
  return {
    bodies,
    get released() {
      return released;
    },
    step() {
      if (disposed || finished) return finished;
      if (released < bodies.length && ticks % 30 === 0) {
        const bean = bodies[released++]!;
        const random = randomFor(bean.id + '-drop');
        // Every bean enters above the opening: no side/front spawning or
        // collision bypass, and no index-based overflow threshold.
        bean.body.position.set((random() - 0.5) * 72, 250, (random() - 0.5) * 30);
        world.addBody(bean.body);
      }
      world.step(1 / 90);
      ticks++;
      finished =
        released === bodies.length && bodies.every(({ body }) => body.sleepState === Body.SLEEPING);
      return finished;
    },
    dispose() {
      disposed = true;
      for (const body of [...world.bodies]) world.removeBody(body);
    },
  };
}
