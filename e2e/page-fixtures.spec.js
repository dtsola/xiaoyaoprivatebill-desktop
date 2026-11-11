import fs from 'node:fs'
import path from 'node:path'

import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

// 用 Python 生成的 golden 快照作为页面数据源：
// 与真实演示数据（7,396 条）同口径，能覆盖图表/表格的实际渲染。
const read = (name) =>
  JSON.parse(fs.readFileSync(path.join('fixtures', 'expected', name), 'utf-8'))

const fixtures = {
  '/available_years': read('available_years.json'),
  '/available_dates': read('available_dates.json'),
  '/categories': read('categories.json'),
  '/category_available_dates': read('category_available_dates.json'),
  '/category_analysis': read('category_analysis_2024.json'),
  // 分类页带 category 参数时返回单分类结构（stats/trend/distribution/pattern）
  '/category_analysis_detail': read('category_analysis_2024_food.json'),
  '/category_expenses': read('category_expenses.json'),
  '/yearly_analysis': read('yearly_analysis_2024.json'),
  '/monthly_analysis': read('monthly_analysis_2024_12.json'),
  '/time_analysis': read('time_analysis_2024.json'),
  '/daily_data': read('daily_data_2024.json'),
  '/analysis': read('analysis_2024.json'),
  '/transactions': read('transactions_expense.json'),
  '/summary': read('summary.json'),
  '/files': read('files_empty.json')
}

const pages = [
  ['年度总览', '/#/yearly', 1],
  ['月度分析', '/#/monthly', 1],
  ['分类分析', '/#/category', 1],
  ['时间分析', '/#/time', 1],
  ['消费洞察', '/#/insights', 1],
  ['交易记录', '/#/transactions', 0]
]

for (const [name, route, minCanvases] of pages) {
  test(`${name}：演示数据下正常渲染且无布局溢出`, async ({ page }) => {
    const errors = []
    page.on('pageerror', (error) => errors.push(error.message))
    page.on('console', (message) => {
      if (message.type() === 'error') errors.push(message.text())
    })

    await page.addInitScript(installTauriMock, fixtures)
    await page.setViewportSize({ width: 1280, height: 840 })
    await page.goto(route)
    await page.waitForTimeout(1200)

    await expect(page.locator('.sidebar')).toBeVisible()
    await expect(page.locator('.page-title').first()).toHaveText(name)
    // 真实数据下不应出现 NaN/Infinity 之类的无效数字
    await expect(page.locator('.content')).not.toContainText('NaN')
    await expect(page.locator('.content')).not.toContainText('Infinity')

    const info = await page.evaluate(() => {
      const content = document.querySelector('.content')
      const header = document.querySelector('.page-header, .category-page-header')
      const cr = content.getBoundingClientRect()
      return {
        canvases: document.querySelectorAll('canvas').length,
        overflow: content.scrollWidth - content.clientWidth,
        headerGap: header ? Math.round(header.getBoundingClientRect().top - cr.top) : null,
        headerSticky: header ? getComputedStyle(header).position === 'sticky' : false
      }
    })

    expect(info.canvases).toBeGreaterThanOrEqual(minCanvases)
    expect(info.overflow).toBeLessThanOrEqual(0)
    // sticky 头部必须紧贴滚动容器顶部；静态头部保持内容内边距即可
    if (info.headerSticky) expect(info.headerGap).toBe(0)

    await page.screenshot({ path: `artifacts/ui/page-fixtures/${route.replace(/[\/#]/g, '_')}.png` })
    expect(errors, `${name} 控制台错误: ${errors.join(' | ')}`).toEqual([])
  })
}
