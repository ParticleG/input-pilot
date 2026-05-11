import { defineBoot } from '#q-app/wrappers';
import { EventBus } from 'quasar';

declare module 'vue' {
  // noinspection JSUnusedGlobalSymbols
  interface ComponentCustomProperties {
    $bus: typeof bus;
  }
}

export const bus = new EventBus<{
  drawer: (
    action: 'close' | 'open' | 'toggle' | 'minimize' | 'maximize' | 'switch',
    position: 'left' | 'right',
  ) => void;
}>();

export default defineBoot(({ app }) => {
  app.config.globalProperties.$bus = bus;
});
