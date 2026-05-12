import { i18nSubPath } from 'src/utils/common';

interface Navigation {
  label: string;
  icon: string;
  available: boolean;
  route: string;
}

const i18n = i18nSubPath('components.navigations');

export const MAIN_NAVIGATIONS: Navigation[] = [
  {
    label: i18n('main.dashboard'),
    icon: 'dashboard',
    available: true,
    route: 'dashboard',
  },
  {
    label: i18n('main.macros'),
    icon: 'play_circle',
    available: true,
    route: 'macros',
  },
  {
    label: i18n('main.targets'),
    icon: 'ads_click',
    available: true,
    route: 'targets',
  },
  {
    label: i18n('main.hotkeys'),
    icon: 'keyboard',
    available: true,
    route: 'hotkeys',
  },
  {
    label: i18n('main.windows'),
    icon: 'window',
    available: true,
    route: 'windows',
  },
];

export const STACK_NAVIGATIONS: Navigation[] = [
  {
    label: i18n('stack.about'),
    icon: 'information',
    available: true,
    route: 'about',
  },
  {
    label: i18n('stack.settings'),
    icon: 'cog',
    available: true,
    route: 'settings',
  },
];
