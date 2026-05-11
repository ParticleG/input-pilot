import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '',
    redirect: '/main/dashboard',
  },
  {
    path: '/stack',
    component: () => import('layouts/MainLayout.vue'),
    children: [],
  },
  {
    path: '/main',
    component: () => import('layouts/MainLayout.vue'),
    children: [
      {
        name: 'dashboard',
        path: 'dashboard',
        components: {
          default: () => import('pages/main/DashboardPage.vue'),
          leftDrawer: () => import('layouts/drawers/MainLeftDrawer.vue'),
          header: () => import('layouts/headers/MainHeader.vue'),
        },
      },
      {
        name: 'macros',
        path: 'macros',
        components: {
          default: () => import('pages/main/MacrosPage.vue'),
          leftDrawer: () => import('layouts/drawers/MainLeftDrawer.vue'),
          header: () => import('layouts/headers/MainHeader.vue'),
        },
      },
      {
        name: 'targets',
        path: 'targets',
        components: {
          default: () => import('pages/main/TargetsPage.vue'),
          leftDrawer: () => import('layouts/drawers/MainLeftDrawer.vue'),
          header: () => import('layouts/headers/MainHeader.vue'),
        },
      },
      {
        name: 'windows',
        path: 'windows',
        components: {
          default: () => import('pages/main/WindowsPage.vue'),
          leftDrawer: () => import('layouts/drawers/MainLeftDrawer.vue'),
          header: () => import('layouts/headers/MainHeader.vue'),
        },
      },
    ],
  },
  // Always leave this as last one,
  // but you can also remove it
  {
    path: '/:catchAll(.*)*',
    component: () => import('pages/ErrorNotFound.vue'),
  },
];

export default routes;
