import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

/**
 * macOS 布局回归。
 *
 * 背景：窗口用 titleBarStyle: Overlay + hiddenTitle，原生红绿灯浮在窗口左上角。
 * 旧实现让侧边栏贴到窗口最顶部，只用 padding 给红绿灯让位，横向管不住——
 * 侧边栏折叠成 64px 后，红绿灯必然越过侧栏右边界压到侧栏内容上。
 * 现在改为统一顶部标题栏：整条 38px 安全区留给红绿灯，侧边栏与内容区都从其下方开始。
 */

const MAC_UA =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 ' +
  '(KHTML, like Gecko) Version/17.0 Safari/605.1.15'

test.describe('macOS 标题栏', () => {
  test.use({ userAgent: MAC_UA })

  test.beforeEach(async ({ page }) => {
    await page.addInitScript(installTauriMock)
  })

  test('统一标题栏独占顶部 38px，侧边栏与内容区都在其下方', async ({ page }) => {
    await page.goto('/')

    const titlebar = page.locator('.mac-titlebar')
    await expect(titlebar).toBeVisible()
    await expect(titlebar).toHaveAttribute('data-tauri-drag-region', '')

    const bar = await titlebar.boundingBox()
    expect(bar.y).toBe(0)
    expect(bar.height).toBe(38)
    expect(bar.x).toBe(0)
    // 横跨整个窗口宽度，红绿灯所在行整条可拖拽
    expect(bar.width).toBe(1280)

    // 侧边栏从标题栏下方开始，不再贴窗口最顶
    const sidebar = await page.locator('.sidebar').boundingBox()
    expect(sidebar.y).toBe(38)

    // 侧边栏内的 LOGO 同样落在标题栏下方
    const logo = await page.locator('.logo').boundingBox()
    expect(logo.y).toBeGreaterThanOrEqual(38)

    // 页面标题区也在标题栏下方
    await page.goto('/#/category')
    const header = await page.locator('.category-page-header').boundingBox()
    expect(header.y).toBe(38)

    // macOS 不渲染前端自绘的窗口按钮
    await expect(page.locator('.window-controls')).toHaveCount(0)
  })

  test('标题栏左侧分色块与侧边栏同宽，折叠后同步收窄', async ({ page }) => {
    await page.goto('/')

    const side = page.locator('.mac-titlebar-side')
    const expanded = await side.boundingBox()
    const sidebar = await page.locator('.sidebar').boundingBox()
    expect(expanded.width).toBe(sidebar.width)
    expect(expanded.x).toBe(0)

    await page.getByRole('button', { name: '收起侧边栏' }).click()
    await expect(page.locator('.sidebar')).toHaveCSS('width', '64px')

    const collapsed = await side.boundingBox()
    expect(collapsed.width).toBe(64)
  })

  test('侧边栏折叠到 64px 后仍完全落在标题栏下方，红绿灯不压侧栏', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: '收起侧边栏' }).click()
    await expect(page.locator('.sidebar')).toHaveCSS('width', '64px')

    const sidebar = await page.locator('.sidebar').boundingBox()
    expect(sidebar.y).toBe(38)
    expect(sidebar.width).toBe(64)

    // 侧栏内所有可见元素的顶部都不在标题栏（0~38px）区域内
    const tops = await page.evaluate(() => {
      const nodes = document.querySelectorAll('.sidebar .nav-item, .sidebar .logo-icon')
      return Array.from(nodes).map((el) => Math.round(el.getBoundingClientRect().top))
    })
    expect(tops.length).toBeGreaterThan(0)
    for (const top of tops) {
      expect(top).toBeGreaterThanOrEqual(38)
    }
  })

  test('底部浮动筛选菜单不遮挡末尾的「关于作者」菜单项', async ({ page }) => {
    // 用较矮的窗口，确保导航列表需要滚动
    await page.setViewportSize({ width: 1280, height: 700 })
    await page.goto('/')

    const nav = page.locator('.nav-menu')
    await nav.evaluate((el) => {
      el.scrollTop = el.scrollHeight
    })

    const author = await page.locator('.nav-item:has(.icon-author)').boundingBox()
    const floating = await page.locator('.floating-menu').boundingBox()

    expect(author).not.toBeNull()
    expect(floating).not.toBeNull()
    // 「关于作者」整条都在浮动菜单上方
    expect(author.y + author.height).toBeLessThanOrEqual(floating.y)
  })

  test('滚动条不占布局宽度，折叠后图标仍水平居中', async ({ page }) => {
    // 矮窗口 + 折叠态，确保导航列表溢出并出现滚动条
    await page.setViewportSize({ width: 1280, height: 600 })
    await page.goto('/')
    await page.getByRole('button', { name: '收起侧边栏' }).click()
    await expect(page.locator('.sidebar')).toHaveCSS('width', '64px')

    const nav = page.locator('.nav-menu')
    await nav.evaluate((el) => {
      el.scrollTop = el.scrollHeight
    })
    await page.waitForTimeout(200)

    // 侧边栏内容区宽度 = 侧边栏宽度（扣掉 1px 右边框），滚动条不额外占位
    const metrics = await page.evaluate(() => {
      const sidebar = document.querySelector('.sidebar')
      const menu = document.querySelector('.nav-menu')
      const icon = document.querySelector('.nav-item .nav-icon')
      const box = (el) => el.getBoundingClientRect()
      return {
        sidebarWidth: box(sidebar).width,
        menuClientWidth: menu.clientWidth,
        sidebarClientWidth: sidebar.clientWidth,
        sidebarCenter: Math.round(box(sidebar).x + box(sidebar).width / 2),
        iconCenter: Math.round(box(icon).x + box(icon).width / 2),
        overflows: menu.scrollHeight > menu.clientHeight
      }
    })

    expect(metrics.overflows).toBe(true)
    // 滚动条不占布局：导航列表宽度等于侧边栏内容区宽度
    expect(metrics.menuClientWidth).toBe(metrics.sidebarClientWidth)
    expect(metrics.iconCenter).toBe(metrics.sidebarCenter)
  })
})

test.describe('Windows 标题栏', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(installTauriMock)
  })

  test('沿用自绘窗口按钮，侧边栏贴窗口顶部', async ({ page }) => {
    await page.goto('/')

    await expect(page.locator('.mac-titlebar')).toHaveCount(0)

    const sidebar = await page.locator('.sidebar').boundingBox()
    expect(sidebar.y).toBe(0)

    await expect(page.locator('.window-controls')).toBeVisible()
    await expect(page.locator('.window-btn')).toHaveCount(3)
  })

  test('底部浮动筛选菜单不遮挡末尾的「关于作者」菜单项', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 700 })
    await page.goto('/')

    const nav = page.locator('.nav-menu')
    await nav.evaluate((el) => {
      el.scrollTop = el.scrollHeight
    })

    const author = await page.locator('.nav-item:has(.icon-author)').boundingBox()
    const floating = await page.locator('.floating-menu').boundingBox()

    expect(author.y + author.height).toBeLessThanOrEqual(floating.y)
  })
})
