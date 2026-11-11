import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
  // 人为拖慢接口，便于观察加载态
  await page.addInitScript(() => {
    const original = window.__TAURI_INTERNALS__.invoke
    window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
      if (cmd === 'api_request') {
        await new Promise((resolve) => setTimeout(resolve, 800))
      }
      return original(cmd, args)
    }
  })
})

test('加载态只覆盖内容区，侧边栏保持可用', async ({ page }) => {
  await page.goto('/#/category')

  const loader = page.locator('.global-loader')
  await expect(loader).toBeVisible()

  const box = await loader.boundingBox()
  expect(box.x).toBe(240)
  expect(box.y).toBe(36)
  expect(box.width).toBe(1280 - 240)
  expect(box.height).toBe(3)

  await page.screenshot({ path: 'artifacts/ui/home/loading.png' })

  // 只是顶部进度条：透明底、不拦截点击
  const style = await loader.evaluate((el) => {
    const computed = getComputedStyle(el)
    return { background: computed.backgroundColor, pointerEvents: computed.pointerEvents }
  })
  expect(style.background).toBe('rgba(0, 0, 0, 0)')
  expect(style.pointerEvents).toBe('none')

  // 侧边栏在蒙版之外，可以正常切换页面
  await expect(page.locator('.sidebar').getByText('月度分析')).toBeVisible()
  await expect(loader).toBeHidden({ timeout: 10_000 })
})
