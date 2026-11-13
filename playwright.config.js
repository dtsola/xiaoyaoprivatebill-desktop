import { defineConfig } from '@playwright/test'

// AppLayout 用 navigator.userAgent 区分 macOS 与 Windows，两者标题栏布局不同。
// 默认固定为 Windows UA，让断言与跑测试的机器无关；
// macOS 专项用例在文件内用 test.use({ userAgent }) 覆盖。
const WINDOWS_UA =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 ' +
  '(KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'

// UI 自动验收：跑 Vite Web 版，Tauri IPC 由 e2e/fixtures/tauri-mock.js 拦截。
// 浏览器通道可用 PW_CHANNEL 覆盖（Windows 装了 Edge 用 msedge，macOS 用内置 chromium）。
export default defineConfig({
  testDir: './e2e',
  // 每个可交付功能的验收产物独立存放，默认不入库。
  outputDir: process.env.PW_OUTPUT_DIR || 'test-results',
  timeout: 30_000,
  fullyParallel: false,
  retries: 0,
  reporter: [['list']],
  workers: process.env.CI ? 2 : 4,
  use: {
    baseURL: 'http://localhost:41420',
    browserName: 'chromium',
    channel: process.env.PW_CHANNEL || undefined,
    userAgent: WINDOWS_UA,
    viewport: { width: 1280, height: 840 },
    locale: 'zh-CN',
    trace: 'retain-on-failure',
    screenshot: 'on'
  },
  webServer: {
    command: 'npm --prefix frontend run dev',
    url: 'http://localhost:41420',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000
  }
})
