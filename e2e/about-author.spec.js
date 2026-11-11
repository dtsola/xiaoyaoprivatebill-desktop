import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('关于页外链交给系统浏览器打开', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/#/about-author')

  // 标题区与其它页面一致
  await expect(page.locator('.page-title')).toHaveText('关于作者')

  // 联系方式包含小遥官网
  const official = page.locator('.contact-item', { hasText: '小遥官网' })
  await expect(official).toContainText('www.xiaoyaosai.com')
  await official.click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_OPENED))
    .toBe('https://www.xiaoyaosai.com/')

  await page.locator('.contact-item', { hasText: 'www.dtsola.com' }).click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_OPENED))
    .toBe('https://www.dtsola.com')

  await page.locator('.project-link').click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_OPENED))
    .toBe('https://github.com/dtsola/xiaoyaoprivatebill-desktop')

  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})

test('扫码联系三张图尺寸统一且不溢出容器', async ({ page }) => {
  await page.goto('/#/about-author')
  await page.locator('.content').evaluate((el) => {
    el.scrollTop = 700
  })

  const layout = await page.evaluate(() => {
    const container = document.querySelector('.qrcode-container').getBoundingClientRect()
    const images = [...document.querySelectorAll('.qrcode-image')].map((el) => {
      const box = el.getBoundingClientRect()
      return { x: Math.round(box.x), width: Math.round(box.width), height: Math.round(box.height) }
    })
    return {
      containerLeft: Math.round(container.x),
      containerRight: Math.round(container.x + container.width),
      images
    }
  })

  // 三张图同尺寸
  expect(new Set(layout.images.map((img) => `${img.width}x${img.height}`)).size).toBe(1)
  // 都在灰色容器内，不再左右溢出
  for (const img of layout.images) {
    expect(img.x).toBeGreaterThanOrEqual(layout.containerLeft)
    expect(img.x + img.width).toBeLessThanOrEqual(layout.containerRight)
  }
})

test('GitHub 仓库指向桌面版仓库', async ({ page }) => {
  await page.goto('/#/about-author')

  await page.locator('.project-link', { hasText: 'GitHub 仓库' }).click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_OPENED))
    .toBe('https://github.com/dtsola/xiaoyaoprivatebill-desktop')
})
