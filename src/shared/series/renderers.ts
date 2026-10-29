import CoffeeReward from './coffee/CoffeeReward.vue';
import CoffeeCollection from './coffee/CoffeeCollection.vue';
import CoffeeFocus from './coffee/CoffeeFocus.vue';
import CoffeeRest from './coffee/CoffeeRest.vue';
import CoffeeRestIcon from './coffee/CoffeeRestIcon.vue';
import CoffeeSettings from './coffee/CoffeeSettings.vue';
import { defaultSeriesId } from './catalog';
import { themeResolver } from './registry';
import type { SeriesId } from './types';
import type { ThemeComponents } from './contracts';
export const seriesRenderers = {
  coffee: {
    reward: CoffeeReward,
    collection: CoffeeCollection,
    focus: CoffeeFocus,
    rest: CoffeeRest,
    restIcon: CoffeeRestIcon,
    settings: CoffeeSettings,
  },
} satisfies Record<SeriesId, ThemeComponents>;
export const componentsFor = themeResolver(seriesRenderers, defaultSeriesId);
