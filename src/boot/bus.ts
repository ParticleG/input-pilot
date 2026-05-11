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
  devicesDrawer: (content: 'addDevices' | 'deviceDetails', deviceId?: string) => void;
  devicesUpdated: () => void;
  platesUpdated: () => void;
  platesDrawer: (content: 'addPlate' | 'plateDetails', plateId?: string) => void;
  filamentsDrawer: (content: 'addFilament' | 'filamentDetails', filamentId?: string) => void;
  filamentsUpdated: () => void;
}>();

export default defineBoot(({ app }) => {
  app.config.globalProperties.$bus = bus;
});
