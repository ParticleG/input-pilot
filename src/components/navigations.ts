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
    icon: 'folder_open',
    available: true,
    route: 'dashboard',
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
