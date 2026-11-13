export default {
  common: {
    loading: 'Loading...',
    retry: 'Retry'
  },
  nav: {
    home: 'Home',
    share: 'Share Poster',
    yearly: 'Yearly Overview',
    monthly: 'Monthly',
    category: 'Categories',
    time: 'Time Analysis',
    insights: 'Insights',
    transactions: 'Transactions',
    agent: 'AI Agent',
    claw: 'XiaoyaoClaw',
    settings: 'Settings',
    author: 'About'
  },
  layout: {
    appName: 'Xiaoyao Bill',
    appNameFull: 'Xiaoyao Bill Desktop',
    logoAlt: 'Xiaoyao Bill',
    window: {
      minimize: 'Minimize',
      maximize: 'Maximize',
      restore: 'Restore',
      close: 'Close'
    },
    sidebar: {
      expand: 'Expand sidebar',
      collapse: 'Collapse sidebar'
    },
    demo: {
      banner: 'You are viewing sample data (for feature preview only)',
      exit: 'Exit demo',
      confirmExit: 'Exit demo mode?',
      exited: 'Demo mode exited',
      exitFailed: 'Failed to exit demo mode',
      entered: 'Demo mode enabled',
      enterFailed: 'Failed to enter demo mode'
    },
    filters: {
      all: 'All transactions',
      large: 'Large only',
      small: 'Small only'
    },
    shareEntry: 'Create a yearly or monthly bill poster',
    openClawSite: 'Open the XiaoyaoClaw website',
    openClawFailed: 'Failed to open the XiaoyaoClaw website',
    closeToTray: 'Minimized to the system tray; quit from the tray menu',
    globalLoading: 'Loading...'
  },
  home: {
    title: 'Xiaoyao Bill',
    subtitle: 'Analyze Alipay and WeChat bills in one place and get detailed spending insights.',
    privacySubtitle: 'All analysis runs locally on your device to protect your privacy.',
    features: {
      analysisTitle: 'Multi-dimensional analysis',
      analysisDesc: 'Explore spending by year, month, category, and time to better understand your habits.',
      privacyTitle: 'Privacy first',
      privacyDesc: 'Your data stays on this device. No network requests are made, and you can delete files at any time.',
      trendTitle: 'Trend tracking',
      trendDesc: 'See how spending changes over time, compare periods, and plan your finances better.',
      searchTitle: 'Smart search',
      searchDesc: 'Search and filter transactions quickly to find any historical record in seconds.'
    },
    uploadTitle: 'Start analyzing your bills',
    uploadHint: 'Import Alipay and WeChat bills (CSV/XLSX). Files stay in your local data folder and can be deleted at any time.',
    privacyHint: 'All analysis runs locally. We never collect or store your personal information.',
    uploadButton: 'Upload bills',
    demoButton: 'View sample data'
  },
  posterPage: {
    title: 'Share Poster',
    scopeLabel: 'Bill scope',
    scopeYear: 'Yearly',
    scopeMonth: 'Monthly',
    yearAria: 'Select year',
    monthAria: 'Select month',
    yearOption: '{year}',
    monthOption: '{month}',
    loading: 'Preparing your bills...',
    empty: 'No bill data available for a poster yet',
    upload: 'Upload bills',
    demo: 'View sample data',
    loadFailed: 'Failed to load data',
    demoEntered: 'Demo mode enabled',
    demoFailed: 'Failed to enter demo mode'
  },
  poster: {
    sideTitleLine1: 'Turn your bills',
    sideTitleLine2: 'into a poster',
    sizeLine: '1080×1440 portrait',
    channels: 'Ready for WeChat Moments, Xiaohongshu, Weibo, Douyin, and Channels',
    templates: 'Templates',
    amounts: 'Amounts',
    hideAll: 'Hide all',
    showAll: 'Show all',
    monthlyHint: 'Monthly posters hide income and balance by default. Turn them back on anytime.',
    amountFields: {
      expense: 'Total spending',
      income: 'Income',
      balance: 'Balance',
      categories: 'Category amounts',
      peak: 'Peak amount'
    },
    generating: 'Generating poster...',
    saving: 'Saving...',
    save: 'Save image',
    tip: 'Generated locally on your device. The QR code points to the official site.',
    qrNotReady: 'The QR code is still loading. Please try again in a moment.',
    saveSuccess: 'Poster saved: {path}',
    saveFailed: 'Failed to save poster',
    filterName: 'PNG image',
    fileNameYear: 'XiaoyaoBill-{year}-yearly.png',
    fileNameMonth: 'XiaoyaoBill-{year}-{month}-monthly.png',
    themeNames: {
      sky: 'Sky',
      ink: 'Ink',
      sunset: 'Sunset',
      mint: 'Mint',
      sakura: 'Sakura',
      ocean: 'Ocean',
      latte: 'Latte',
      mono: 'Mono'
    }
  },
  settings: {
    language: {
      label: 'Language',
      description: 'Follow the system, or choose the interface language manually.',
      system: 'Follow system',
      zh: '简体中文',
      en: 'English',
      saved: 'Interface language updated',
      failed: 'Failed to update language'
    }
  }
}
