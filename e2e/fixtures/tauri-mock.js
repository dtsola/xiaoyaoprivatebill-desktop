/**
 * Tauri v2 的最小 mock 桥。
 *
 * 在页面脚本执行前注入 window.__TAURI_INTERNALS__，让 Web 版启动时的
 * invoke / 事件监听安全返回，不依赖真实 Rust 后端。
 *
 * 注意：本函数由 page.addInitScript 序列化到浏览器执行，所有数据必须定义在函数体内。
 */
export function installTauriMock() {
  const hourly = Array.from({ length: 24 }, (_, hour) => ({
    hour,
    amount: 0,
    count: 0
  }))

  const yearly = {
    year: 2024,
    yearly_stats: {
      total_income: 0,
      total_expense: 0,
      total_count: 0,
      avg_daily_expense: 0,
      avg_monthly_income: 0,
      avg_transaction: 0,
      active_days: 0,
      income_count: 0,
      expense_count: 0
    },
    monthly_data: [],
    category_data: [],
    comparison: {}
  }

  const monthly = {
    year: 2024,
    month: 1,
    monthly_stats: {
      total_income: 0,
      total_expense: 0,
      total_count: 0,
      avg_daily_expense: 0,
      income_count: 0,
      expense_count: 0
    },
    daily_data: [],
    category_data: [],
    comparison: {}
  }

  // `/analysis` 的 23 个键，取值与 bill-services 的契约测试使用同一份数据。
  const analysis = {
    merchant_analysis: {
      merchant_stats: {},
      frequent_merchants: [
        { name: '家电店', amount: 500, count: 1, last_visit: '2024-04-15' },
        { name: '健身房', amount: 400, count: 2, last_visit: '2024-02-10' },
        { name: '服装店', amount: 300, count: 1, last_visit: '2024-05-20' },
        { name: '咖啡店', amount: 120, count: 6, last_visit: '2024-01-10' }
      ]
    },
    scenario_analysis: [
      { name: '线上', value: 400, category: '渠道' },
      { name: '线下', value: 1305, category: '渠道' },
      { name: '上午(9-12点)', value: 355, category: '时段' },
      { name: '深夜(23-6点)', value: 150, category: '时段' },
      { name: '零花(0-100)', value: 1045, category: '层级' }
    ],
    habit_analysis: {
      daily_avg: 81.19,
      active_days: 21,
      weekend_ratio: 25.2,
      fixed_expenses: 0,
      month_start_ratio: 5.6
    },
    latte_factor: { total_amount: 120, item_count: 6, top_merchant: '咖啡店', avg_price: 20 },
    nighttime_analysis: { total_amount: 180, ratio: 10.56, top_merchant: '深夜食堂', transaction_count: 3 },
    subscription_analysis: [
      { name: '视频会员', monthly_amount: 15, annual_amount: 180 },
      { name: '云存储', monthly_amount: 10, annual_amount: 120 }
    ],
    inflation_analysis: { trend: 'up', rate: 872.38, first_avg: 41.14, last_avg: 400 },
    brand_loyalty: {
      top_amount: { name: '家电店', value: 500 },
      top_count: { name: '咖啡店', value: 6 }
    },
    sankey_data: {
      nodes: [
        { name: '支付宝', category: 'source' },
        { name: '微信', category: 'source' },
        { name: '餐饮美食', category: 'category' },
        { name: '购物', category: 'category' },
        { name: '娱乐', category: 'category' }
      ],
      links: [
        { source: 0, target: 2, value: 260 },
        { source: 1, target: 2, value: 50 },
        { source: 0, target: 3, value: 510 },
        { source: 1, target: 3, value: 410 },
        { source: 0, target: 4, value: 445 },
        { source: 1, target: 4, value: 30 }
      ]
    },
    engel_coefficient: {
      coefficient: 18.18,
      food_expense: 310,
      total_expense: 1705,
      level: '富裕'
    },
    weekend_monday: [
      { day: '周一', amount: 875, count: 6, avg: 145.83 },
      { day: '周末', amount: 430, count: 7, avg: 61.43 },
      { day: '工作日', amount: 400, count: 11, avg: 36.36 }
    ],
    story_data: {
      max_day: { date: '2024年04月15日', amount: 500 },
      max_month: { month: '2024-04', amount: 500 },
      latest_tx: { time: '02:00', merchant: '凌晨超市', amount: 50 },
      top_category: { name: '购物', amount: 920 },
      summary: { total_days: 141, tx_count: 24, total_amount: 1705 },
      features: {
        coffee: { count: 6, amount: 120 },
        night: { avg: 60, total: 180, count: 3 },
        weekend: { weekday_avg: 75, weekend_avg: 61.43 },
        inflation: { merchant: '咖啡店', start_price: 20, end_price: 20, trend: 'stable' },
        first_tx: { date: '2024-01-01 08:00', merchant: '视频会员', amount: 15, product: '' },
        peak_hour: 8,
        takeout: { count: 1, amount: 100 },
        top_season: '冬'
      }
    },
    tags: {
      tags: ['娱乐控', '购物控', '餐饮美食控', '随性消费派'],
      time_pattern: '您的消费时间比较规律，集中在日间',
      spending_preference: '最常消费的品类是娱乐(27.9%), 购物(54.0%), 餐饮美食(18.2%)',
      spending_pattern: '您的消费比较随性，波动较大',
      spending_power: '日均消费81元，属于中等消费人群'
    },
    payment_analysis: [
      { method: '微信', amount: 490, count: 9 },
      { method: '支付宝', amount: 1215, count: 15 }
    ],
    chord_data: {
      nodes: [
        { name: '周一', category: 'weekday' },
        { name: '周二', category: 'weekday' },
        { name: '周三', category: 'weekday' },
        { name: '周四', category: 'weekday' },
        { name: '周五', category: 'weekday' },
        { name: '周六', category: 'weekday' },
        { name: '周日', category: 'weekday' },
        { name: '购物', category: 'category' },
        { name: '娱乐', category: 'category' },
        { name: '餐饮美食', category: 'category' }
      ],
      links: [
        { source: '周一', target: '娱乐', value: 25 },
        { source: '周一', target: '购物', value: 830 },
        { source: '周三', target: '娱乐', value: 200 },
        { source: '周六', target: '餐饮美食', value: 120 }
      ]
    },
    funnel_data: [
      { name: '100-500元', value: 800 },
      { name: '500-1000元', value: 500 },
      { name: '0-50元', value: 355 },
      { name: '50-100元', value: 50 },
      { name: '1000-5000元', value: 0 },
      { name: '>5000元', value: 0 }
    ],
    quadrant_data: [
      { name: '健身房', category: '娱乐', frequency: 2, avg_amount: 200, total_amount: 400 },
      { name: '咖啡店', category: '餐饮美食', frequency: 6, avg_amount: 20, total_amount: 120 },
      { name: '奶茶店', category: '餐饮美食', frequency: 2, avg_amount: 25, total_amount: 50 }
    ],
    radar_data: {
      indicator: [
        { name: '购物', max: 880 },
        { name: '娱乐', max: 880 },
        { name: '餐饮美食', max: 880 }
      ],
      series: [
        { name: 'Q1', value: [120, 475, 310] },
        { name: 'Q2', value: [800, 0, 0] }
      ]
    },
    wordcloud_data: [
      { name: '家电店', value: 500 },
      { name: '健身房', value: 400 },
      { name: '服装店', value: 300 },
      { name: '咖啡店', value: 120 },
      { name: '深夜食堂', value: 100 }
    ],
    themeriver_data: {
      categories: ['购物', '娱乐', '餐饮美食'],
      data: [
        ['2024-01', '购物', 40],
        ['2024-01', '娱乐', 225],
        ['2024-01', '餐饮美食', 170],
        ['2024-02', '购物', 80],
        ['2024-02', '娱乐', 225],
        ['2024-02', '餐饮美食', 140],
        ['2024-03', '购物', 0],
        ['2024-03', '娱乐', 25],
        ['2024-03', '餐饮美食', 0]
      ]
    },
    boxplot_data: {
      categories: ['购物', '娱乐', '餐饮美食'],
      data: [
        { c: 0, v: 10, m: '便利店', d: '2024-01-13' },
        { c: 0, v: 500, m: '家电店', d: '2024-04-15' },
        { c: 1, v: 15, m: '视频会员', d: '2024-01-01' },
        { c: 1, v: 200, m: '健身房', d: '2024-01-10' },
        { c: 2, v: 20, m: '咖啡店', d: '2024-01-05' },
        { c: 2, v: 100, m: '深夜食堂', d: '2024-02-10' }
      ],
      box_data: [
        [10, 30, 40, 237.5, 500],
        [10, 10, 15, 61.25, 200],
        [20, 20, 20, 25, 100]
      ]
    },
    heatmap_data: [
      [8, 0, 2],
      [9, 0, 1],
      [5, 1, 1],
      [23, 5, 1],
      [2, 6, 1]
    ],
    pareto_data: {
      categories: ['购物', '娱乐', '餐饮美食'],
      values: [920, 475, 310],
      percentages: [53.96, 81.82, 100]
    }
  }

  const routes = {
    '/available_years': { success: true, years: [2024] },
    '/available_dates': { success: true, months: [], years: [2024] },
    '/categories': { success: true, categories: [] },
    '/category_available_dates': { years: [], months: {} },
    '/category_analysis': { categories: [], category_groups: { alipay: [], wechat: [] } },
    '/category_expenses': { categories: [], amounts: [] },
    '/session/status': { active: false, is_demo: false, user_id: null, session_start: null },
    '/files': { files: [] },
    '/yearly_analysis': { success: true, data: yearly },
    '/monthly_analysis': { success: true, data: monthly },
    '/time_analysis': { hourly, weekday_weekend: {} },
    '/daily_data': {
      expense: [],
      income: [],
      transaction: [],
      expense_quantiles: [],
      income_quantiles: []
    },
    '/analysis': { success: true, data: analysis },
    '/transactions': {
      success: true,
      transactions: [],
      summary: { total_income: 0, total_expense: 0, balance: 0 },
      pagination: { current_page: 1, total_pages: 1, total_records: 0 }
    },
    '/summary': {},
    '/yearly_data': {},
    '/monthly_data': { success: true, data: {} },
    '/overview_data': {},
    '/filtered_monthly_analysis': {},
    '/top_transactions': { success: true, transactions: [] },
    '/upload': { success: true },
    '/clear_data': { success: true },
    '/demo/enter': { success: true },
    '/demo/exit': { success: true }
  }

  const callbacks = new Map()
  let nextCallbackId = 1

  let closeToTray = true
  let mcpEnabled = false
  const mcpStatus = () => ({
    enabled: mcpEnabled,
    running: mcpEnabled,
    port: 17650,
    endpoint: 'http://127.0.0.1:17650/mcp',
    last_error: null
  })

  const invoke = async (cmd, args) => {
    if (cmd === 'desktop_settings') {
      return {
        close_to_tray: closeToTray,
        data_dir: 'C:/Users/demo/AppData/Roaming/xiaoyaobill-desktop'
      }
    }
    if (cmd === 'set_close_to_tray') {
      closeToTray = Boolean(args?.enabled)
      return { close_to_tray: closeToTray }
    }
    if (cmd === 'open_data_dir') return null
    if (cmd === 'open_storage_dir') return null
    if (cmd === 'storage_status') {
      return {
        trash_count: 2,
        snapshot_count: 1,
        audit_bytes: 2048,
        audit_path: 'C:/Users/demo/AppData/Roaming/xiaoyaobill-desktop/audit.jsonl'
      }
    }
    if (cmd === 'mcp_status') return mcpStatus()
    if (cmd === 'mcp_set_enabled') {
      mcpEnabled = Boolean(args?.enabled)
      return mcpStatus()
    }
    if (cmd === 'mcp_config') {
      return { endpoint: mcpStatus().endpoint, token: 'a'.repeat(64) }
    }
    if (cmd === 'mcp_rotate_token') {
      return { endpoint: mcpStatus().endpoint, token: 'b'.repeat(64) }
    }
    if (cmd === 'api_request') {
      const req = args?.req ?? {}
      const path = req.path ?? ''
      if (path.startsWith('/category_detail/')) return []
      if (path.startsWith('/category_trend/')) return {}
      if (path === '/upload') {
        const paths = req.body?.paths ?? []
        const provider = req.body?.provider ?? 'alipay'
        const added = paths.map((item) => {
          const name = String(item).split(/[\/]/).pop()
          return { name, size: 1024, size_formatted: '1.0 KB', source: provider }
        })
        routes['/files'].files.push(...added)
        return { success: true, filename: added[0]?.name ?? '', message: '文件上传成功' }
      }
      if (path === '/clear_data') {
        routes['/files'].files = []
        return { success: true, message: '数据已清除' }
      }
      if (req.method === 'DELETE' && path.startsWith('/files/')) {
        const name = decodeURIComponent(path.slice('/files/'.length))
        routes['/files'].files = routes['/files'].files.filter((file) => file.name !== name)
        return { success: true }
      }
      return routes[path] ?? { success: false, error: `接口不存在: ${path}` }
    }
    if (cmd === 'plugin:event|listen') return 1
    if (cmd === 'plugin:event|unlisten') return null
    if (cmd === 'plugin:dialog|open') return window.__TAURI_TEST_DIALOG ?? null
    if (cmd === 'plugin:opener|open_url') {
      window.__TAURI_TEST_OPENED = args?.url ?? null
      return null
    }
    return null
  }

  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'main' } },
    invoke,
    transformCallback: (cb) => {
      const id = nextCallbackId++
      callbacks.set(id, cb)
      return id
    },
    unregisterCallback: (id) => {
      callbacks.delete(id)
    },
    convertFileSrc: (path) => path
  }

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: () => {}
  }
}
