// 前端冻结基线规则（供 CLI 与测试共用）。
//
// 设计文档允许改 8 个文件；此外还有三类**非页面源码**的合法变更：
// 离线图标资源、测试配置与测试文件。其余任何前端源码改动都应被拦截。

export const ALLOWED_FILES = new Set([
  'frontend/src/api/client.js',
  'frontend/src/views/Settings.vue',
  'frontend/index.html',
  'frontend/src/router/index.js',
  'frontend/src/components/AppLayout.vue',
  'frontend/src/views/AboutAuthor.vue',
  'frontend/vite.config.js',
  'frontend/src/views/Home.vue',
  // 国际化：启动时先读取语言偏好再挂载
  'frontend/src/main.js',
  // 图标迁移例外：Font Awesome → Lucide（仅保留支付宝/微信/Bilibili 品牌图标）
  'frontend/src/views/Yearly.vue',
  'frontend/src/views/Monthly.vue',
  'frontend/src/views/Category.vue',
  'frontend/src/views/Time.vue',
  'frontend/src/views/Insights.vue',
  'frontend/src/views/Transactions.vue',
  'frontend/src/components/common/Toast.vue',
  'frontend/src/components/common/Modal.vue',
  'frontend/src/components/common/StatCard.vue',
  // 分享海报：新增页面、组件与纯逻辑模块
  'frontend/src/views/SharePosterPage.vue',
  'frontend/src/components/SharePoster.vue',
  'frontend/src/utils/sharePoster.js',
  // 全局字号标尺对齐 xiaoyaotime（仅 :root token、body 与 h1–h4）
  'frontend/src/assets/styles/main.css'
])

export const ALLOWED_PREFIXES = [
  'frontend/public/vendor/',
  // 国际化：词典与语言切换模块（纯新增文件）
  'frontend/src/i18n/',
  // AI Agent 接入页的小遥Claw 实操截图（只读静态资源）
  'frontend/public/images/claw/'
]
export const ALLOWED_EXACT = new Set([
  'frontend/package.json',
  'frontend/package-lock.json',
  'frontend/vitest.config.js',
  // 品牌资源：桌面版统一使用新 LOGO（图标由 docs/产品文档/logos 生成）
  'frontend/public/images/logo_128.png',
  'frontend/public/favicon.png'
])

export function isAllowed(path) {
  if (!path.startsWith('frontend/')) return true
  if (ALLOWED_FILES.has(path)) return true
  if (ALLOWED_EXACT.has(path)) return true
  if (path.endsWith('.test.js')) return true
  return ALLOWED_PREFIXES.some((prefix) => path.startsWith(prefix))
}

export function violations(paths) {
  return paths.filter((path) => !isAllowed(path))
}
