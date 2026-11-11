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
    '/analysis': { success: true, data: {} },
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

  const invoke = async (cmd, args) => {
    if (cmd === 'api_request') {
      const req = args?.req ?? {}
      const path = req.path ?? ''
      if (path.startsWith('/category_detail/')) return []
      if (path.startsWith('/category_trend/')) return {}
      return routes[path] ?? { success: false, error: `接口不存在: ${path}` }
    }
    if (cmd === 'plugin:event|listen') return 1
    if (cmd === 'plugin:event|unlisten') return null
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
