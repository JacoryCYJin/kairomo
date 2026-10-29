import SnowCollection from './snow-mountain/SnowCollection.vue';
import SnowFocus from './snow-mountain/SnowFocus.vue';
import SnowRest from './snow-mountain/SnowRest.vue';
import SnowRestIcon from './snow-mountain/SnowRestIcon.vue';
import SnowReward from './snow-mountain/SnowReward.vue';
import SnowSettings from './snow-mountain/SnowSettings.vue';
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
  'snow-mountain': {
    collection: SnowCollection,
    focus: SnowFocus,
    rest: SnowRest,
    restIcon: SnowRestIcon,
    reward: SnowReward,
    settings: SnowSettings,
  },
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
