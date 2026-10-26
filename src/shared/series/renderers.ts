import CoffeeReward from './coffee/CoffeeReward.vue';
import CoffeeCollection from './coffee/CoffeeCollection.vue';
import type { SeriesId } from './types';
export const seriesRenderers = {
  coffee: { reward: CoffeeReward, collection: CoffeeCollection },
} satisfies Record<SeriesId, { reward: typeof CoffeeReward; collection: typeof CoffeeCollection }>;
