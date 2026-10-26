import { onMounted, onUnmounted, shallowRef, watch } from 'vue';
import { createBeanLayoutJob } from './layouts';
import type { BeanLayout } from './layouts';
export function useCoffeeLayout(ids: () => string[]) {
  const layout = shallowRef<BeanLayout | null>(null);
  let job: ReturnType<typeof createBeanLayoutJob> | undefined;
  let frame = 0,
    generation = 0;
  function stop() {
    cancelAnimationFrame(frame);
    frame = 0;
  }
  function run() {
    stop();
    if (!job || document.hidden) return;
    const version = generation;
    function solve() {
      if (!job || version !== generation) return;
      const result = job.result ?? job.step();
      if (result) {
        layout.value = result;
        frame = 0;
      } else frame = requestAnimationFrame(solve);
    }
    solve();
  }
  function rebuild() {
    generation++;
    stop();
    job?.dispose();
    layout.value = null;
    job = createBeanLayoutJob(ids());
    run();
  }
  function visibility() {
    if (document.hidden) stop();
    else if (!layout.value) run();
  }
  watch(() => JSON.stringify(ids()), rebuild);
  onMounted(() => {
    document.addEventListener('visibilitychange', visibility);
    rebuild();
  });
  onUnmounted(() => {
    generation++;
    stop();
    job?.dispose();
    document.removeEventListener('visibilitychange', visibility);
  });
  return { layout };
}
