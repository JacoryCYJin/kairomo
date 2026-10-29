import manifest from '../../../theme-kits/catalog.json' with { type: 'json' };
import type { Settings, ThemeOptionValue } from '../ipc/types';
import type { SeriesId } from './types';
interface OptionSpec {
  default: ThemeOptionValue;
  legacyField?: string;
}
/** Legacy fields are compatibility bindings, never a second copy of the option. */
export function themeOptions(
  settings: Settings,
  id: SeriesId = settings.visual_series,
): Record<string, ThemeOptionValue> {
  const specs: Record<string, OptionSpec> = manifest.themes[id].options;
  return Object.fromEntries(
    Object.entries(specs).map(([key, spec]) => [
      key,
      spec.legacyField
        ? ((settings as unknown as Record<string, ThemeOptionValue>)[spec.legacyField] ??
          spec.default)
        : (settings.theme_options?.[id]?.[key] ?? spec.default),
    ]),
  );
}
