const productName = 'Input Pilot';

export default {
  components: {
    navigations: {
      main: {
        dashboard: 'Dashboard',
        macros: 'Macros',
        targets: 'Targets',
        hotkeys: 'Hotkeys',
        windows: 'Windows',
      },
      stack: {
        about: 'About',
        settings: 'Settings',
      },
    },
    ThemeButton: {
      labels: {
        switchTheme: 'Switch Theme',
      },
    },
  },
  layouts: {
    headers: {
      MainHeader: {
        labels: {
          title: productName,
        },
      },
    },
  },
};
