import { defineStore } from '#q-app/wrappers';
import { createPinia } from 'pinia';
import { createPersistedState } from 'pinia-plugin-persistedstate';

import packageJson from 'app/package.json';

/*
 * When adding new properties to stores, you should also
 * extend the `PiniaCustomProperties` interface.
 * @see https://pinia.vuejs.org/core-concepts/plugins.html#typing-new-store-properties
 */
/* eslint-disable @typescript-eslint/no-empty-object-type */
declare module 'pinia' {
  // noinspection JSUnusedGlobalSymbols
  export interface PiniaCustomProperties {
    // add your custom properties here, if any
  }
}
/* eslint-enable @typescript-eslint/no-empty-object-type */

/*
 * If not building with SSR mode, you can
 * directly export the Store instantiation;
 *
 * The function below can be async too; either use
 * async/await or return a Promise which resolves
 * with the Store instance.
 */

export default defineStore((/* { ssrContext } */) => {
  const pinia = createPinia();

  pinia.use(
    createPersistedState({
      auto: true,
      key: (id) => `ParticleG.${packageJson.name}.${id}`,
    }),
  );

  return pinia;
});
