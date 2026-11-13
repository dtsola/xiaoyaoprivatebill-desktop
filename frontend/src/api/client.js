import { invoke } from '@tauri-apps/api/core'

/**
 * 统一 API 客户端。
 *
 * 桌面端不再走 HTTP，而是通过 Tauri IPC 调用 Rust 的 `api_request`。
 * 20 个业务方法的名称与参数保持与 Web 版一致。
 */
export const api = {
  // ==================== 数据分析 API ====================
  getYearlyAnalysis: (params) => get('/yearly_analysis', params),
  getMonthlyAnalysis: (params) => get('/monthly_analysis', params),
  getCategoryExpenses: (params) => get('/category_expenses', params),
  getTransactions: (params) => get('/transactions', params),
  getAnalysis: (params) => get('/analysis', params),
  getTimeAnalysis: (params) => get('/time_analysis', params),
  getDailyData: (params) => get('/daily_data', params),
  getSummary: () => get('/summary'),

  // ==================== 元数据 API ====================
  getAvailableDates: () => get('/available_dates'),
  getAvailableYears: () => get('/available_years'),
  getCategories: () => get('/categories'),
  getCategoryAnalysis: (params) => get('/category_analysis', params),
  getCategoryAvailableDates: () => get('/category_available_dates'),

  // ==================== 文件管理 API ====================
  uploadFile: (formData) => post('/upload', formData),
  uploadPaths: (paths, provider) => post('/upload', { paths, provider }),
  getFiles: () => get('/files'),
  deleteFile: (filename) => delete_(`/files/${encodeURIComponent(filename)}`),
  clearData: () => post('/clear_data'),

  // ==================== 桌面设置 ====================
  desktopSettings: () => invoke('desktop_settings'),
  setCloseToTray: (enabled) => invoke('set_close_to_tray', { enabled }),
  openDataDir: () => invoke('open_data_dir'),
  storageStatus: () => invoke('storage_status'),
  openStorageDir: (kind) => invoke('open_storage_dir', { kind }),
  setLanguage: (preference, locale) => invoke('set_language', { preference, locale }),
  applyShellLanguage: (locale) => invoke('apply_shell_language', { locale }),

  // ==================== MCP 服务 ====================
  mcpStatus: () => invoke('mcp_status'),
  mcpSetEnabled: (enabled) => invoke('mcp_set_enabled', { enabled }),
  mcpConfig: () => invoke('mcp_config'),
  mcpRotateToken: () => invoke('mcp_rotate_token'),

  // ==================== 分享海报 ====================
  shareQrSvg: () => invoke('share_qr_svg'),
  saveSharePoster: (path, pngBase64) => invoke('save_share_poster', { path, pngBase64 }),

  // ==================== 会话管理 API ====================
  enterDemo: () => post('/demo/enter'),
  exitDemo: () => post('/demo/exit'),
  getSessionStatus: () => getSessionStatus()
}

// ==================== 请求封装函数 ====================

async function request(method, path, query, body) {
  try {
    return await invoke('api_request', {
      req: {
        method,
        path,
        query: query ?? {},
        body: body ?? null
      }
    })
  } catch (error) {
    throw new Error(typeof error === 'string' ? error : '请求失败')
  }
}

/**
 * 处理三种响应格式：
 * - { success: false, error } → 抛错
 * - { success: true, data } → 解包 data
 * - 其他对象 → 去掉 success 后返回
 */
function normalize(data) {
  if (data && typeof data === 'object' && 'success' in data && !data.success) {
    throw new Error(data.error || '请求失败')
  }
  if (data && typeof data === 'object' && 'data' in data) {
    return data.data
  }
  if (data && typeof data === 'object') {
    const { success, ...rest } = data
    return rest
  }
  return data
}

async function get(endpoint, params) {
  const query = {}
  for (const [key, value] of Object.entries(params ?? {})) {
    if (value !== undefined && value !== null && value !== '') {
      query[key] = String(value)
    }
  }
  return normalize(await request('GET', endpoint, query, null))
}

async function post(endpoint, data) {
  if (typeof FormData !== 'undefined' && data instanceof FormData) {
    throw new Error('桌面端不支持 FormData 上传，请使用 uploadPaths')
  }
  return normalize(await request('POST', endpoint, {}, data ?? null))
}

async function delete_(endpoint) {
  return normalize(await request('DELETE', endpoint, {}, null))
}

/**
 * 会话状态接口的字段形状与普通接口不同。
 * 桌面端返回 { active, is_demo, user_id, session_start }。
 */
async function getSessionStatus() {
  try {
    const data = await request('GET', '/session/status', {}, null)
    if (data && data.success === false) {
      throw new Error(data.error || '请求失败')
    }
    return {
      active: data?.active || false,
      message: data?.message || '',
      is_demo: data?.is_demo || false,
      user_id: data?.user_id ?? null,
      session_start: data?.session_start ?? null
    }
  } catch (error) {
    return {
      active: false,
      message: '获取会话状态失败',
      is_demo: false,
      user_id: null,
      session_start: null
    }
  }
}

export default api
