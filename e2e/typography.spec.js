import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('全局字号对齐 xiaoyaotime 标尺', async ({ page }) => {
  await page.goto('/#/settings')

  const sizes = await page.evaluate(() => {
    const pick = (selector) => {
      const el = document.querySelector(selector)
      return el ? getComputedStyle(el).fontSize : null
    }
    return {
      body: getComputedStyle(document.body).fontSize,
      weight: getComputedStyle(document.body).fontWeight,
      title: pick('.page-title'),
      nav: pick('.nav-item'),
      section: pick('.section-title')
    }
  })

  expect(sizes.body).toBe('14px')
  expect(Number(sizes.weight)).toBeGreaterThanOrEqual(450)
  expect(sizes.title).toBe('28px')
  expect(sizes.nav).toBe('14px')
  expect(sizes.section).toBe('20px')
})
