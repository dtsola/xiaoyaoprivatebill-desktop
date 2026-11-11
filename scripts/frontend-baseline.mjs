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
  'frontend/src/views/Home.vue'
])

export const ALLOWED_PREFIXES = ['frontend/public/vendor/']
export const ALLOWED_EXACT = new Set([
  'frontend/package.json',
  'frontend/package-lock.json',
  'frontend/vitest.config.js'
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
