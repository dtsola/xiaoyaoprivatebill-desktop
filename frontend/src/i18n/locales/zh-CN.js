export default {
  common: {
    loading: '加载中...',
    retry: '重试'
  },
  nav: {
    home: '首页',
    share: '分享海报',
    yearly: '年度总览',
    monthly: '月度分析',
    category: '分类分析',
    time: '时间分析',
    insights: '消费洞察',
    transactions: '交易记录',
    agent: 'AI Agent 接入',
    claw: '小遥Claw',
    settings: '设置',
    author: '关于作者'
  },
  layout: {
    appName: '小遥账单',
    appNameFull: '小遥账单桌面版',
    logoAlt: '小遥账单助手',
    window: {
      minimize: '最小化',
      maximize: '最大化',
      restore: '还原',
      close: '关闭'
    },
    sidebar: {
      expand: '展开侧边栏',
      collapse: '收起侧边栏'
    },
    demo: {
      banner: '您正在浏览示例数据（仅供演示功能预览）',
      exit: '退出演示',
      confirmExit: '确定要退出演示模式吗？',
      exited: '已退出演示模式',
      exitFailed: '退出失败',
      entered: '已进入演示模式',
      enterFailed: '进入演示模式失败'
    },
    filters: {
      all: '全部交易',
      large: '仅大额交易',
      small: '仅小额交易'
    },
    shareEntry: '生成年度 / 月度账单海报',
    openClawSite: '打开小遥Claw 官网',
    openClawFailed: '打开小遥Claw 官网失败',
    closeToTray: '已最小化到托盘，可从托盘菜单退出',
    globalLoading: '加载中...'
  },
  home: {
    title: '小遥账单助手',
    subtitle: '支持支付宝和微信账单的一站式分析，获取详细的消费洞察。',
    privacySubtitle: '所有数据分析均在本地进行，保护您的隐私安全。',
    features: {
      analysisTitle: '多维度分析',
      analysisDesc: '支持年度、月度、分类、时间等多个维度的消费分析，帮助您更好地了解消费习惯。',
      privacyTitle: '隐私保护',
      privacyDesc: '数据仅保存在本机，不发起任何网络请求。文件由你手动管理，可随时删除。',
      trendTitle: '趋势追踪',
      trendDesc: '展示消费趋势变化，支持环比分析，帮助您更好地规划财务。',
      searchTitle: '智能搜索',
      searchDesc: '支持交易记录快速搜索和筛选，轻松找到任意历史交易信息。'
    },
    uploadTitle: '开始分析您的账单',
    uploadHint: '支持导入支付宝和微信账单（CSV/XLSX），文件保存在本机数据目录，可随时删除。',
    privacyHint: '所有数据分析均在本地进行，我们不会收集或存储您的个人信息。',
    uploadButton: '上传账单文件',
    demoButton: '查看示例数据'
  },
  posterPage: {
    title: '分享海报',
    scopeLabel: '账单范围',
    scopeYear: '年度账单',
    scopeMonth: '月度账单',
    yearAria: '选择年份',
    monthAria: '选择月份',
    yearOption: '{year} 年',
    monthOption: '{month} 月',
    loading: '正在整理账单...',
    empty: '还没有可生成海报的账单数据',
    upload: '上传账单',
    demo: '查看示例数据',
    loadFailed: '加载数据失败',
    demoEntered: '已进入演示模式',
    demoFailed: '进入演示模式失败'
  },
  poster: {
    sideTitleLine1: '把账单变成',
    sideTitleLine2: '一张海报',
    sizeLine: '1080×1440 竖版',
    channels: '适合朋友圈、小红书、微博、抖音、视频号',
    templates: '海报模板',
    amounts: '金额显示',
    hideAll: '全部隐藏',
    showAll: '全部显示',
    monthlyHint: '月度海报默认隐藏收入与结余，可随时打开。',
    amountFields: {
      expense: '支出总额',
      income: '收入',
      balance: '结余',
      categories: '分类金额',
      peak: '趋势最高金额'
    },
    generating: '正在生成海报...',
    saving: '保存中...',
    save: '保存图片',
    tip: '图片完全在本机生成；二维码指向产品官网。',
    qrNotReady: '二维码尚未加载完成，请稍后再试',
    saveSuccess: '海报已保存：{path}',
    saveFailed: '保存海报失败',
    filterName: 'PNG 图片',
    fileNameYear: '小遥账单-{year}年度账单.png',
    fileNameMonth: '小遥账单-{year}年{month}月账单.png',
    themeNames: {
      sky: '晴空',
      ink: '墨夜',
      sunset: '暖阳',
      mint: '薄荷',
      sakura: '樱花',
      ocean: '深海',
      latte: '拿铁',
      mono: '极简'
    }
  },
  settings: {
    language: {
      label: '语言',
      description: '跟随系统，或手动指定界面语言。',
      system: '跟随系统',
      zh: '简体中文',
      en: 'English',
      saved: '界面语言已更新',
      failed: '语言设置失败'
    }
  }
}
