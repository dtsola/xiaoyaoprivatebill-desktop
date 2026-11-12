/**
 * 账单分享海报：数据模型与 Canvas 绘制。
 *
 * 海报固定 1080×1440（3:4，适合朋友圈 / 小红书），完全在本地绘制，
 * 不发起任何网络请求；二维码由 Rust 侧离线生成。
 */

export const POSTER_WIDTH = 1080
export const POSTER_HEIGHT = 1440
export const PRODUCT_SITE_URL = 'https://www.xiaoyaosai.com/xiaoyaoprivatebill-desktop/'
export const PRODUCT_NAME = '小遥账单桌面版'
export const PRODUCT_SLOGAN = '隐私优先 · 完全离线 · 本地分析'

const FONT_STACK = '"PingFang SC", "Microsoft YaHei", "Helvetica Neue", Arial, sans-serif'
const INK = '#0B1B33'
const INK_SOFT = '#5B6B84'
const MUTED = '#8E9AAF'
const BRAND = '#007AFF'
const GREEN = '#34C759'
const RED = '#FF453A'
const TRACK = '#EDF2FA'
const PANEL = '#F7F9FC'
const LINE = '#E6EDF7'

const toNumber = (value) => {
  const number = Number(value)
  return Number.isFinite(number) ? number : 0
}

/** 千分位金额，保留两位小数。 */
export function formatYuan(value) {
  return toNumber(value).toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })
}

/** 分类金额缩写：≥1 亿用「亿」，≥1 万用「万」，其余保持千分位。 */
export function formatCompactYuan(value) {
  const amount = toNumber(value)
  const abs = Math.abs(amount)
  if (abs >= 100_000_000) {
    return `${(amount / 100_000_000).toFixed(2)}亿`
  }
  if (abs >= 10_000) {
    return `${(amount / 10_000).toFixed(2)}万`
  }
  return formatYuan(amount)
}

/** 分享时可选隐藏金额，只保留掩码。 */
export function displayAmount(value, hideAmounts = false) {
  return hideAmounts ? '••••••' : formatYuan(value)
}

const monthLabel = (value) => {
  const parts = String(value ?? '').split('-')
  const month = Number(parts[1])
  return Number.isFinite(month) && month >= 1 && month <= 12 ? `${month}月` : String(value ?? '')
}

function aggregateCategories(data) {
  const totals = new Map()
  const rows = data?.categories_source?.expense
  if (Array.isArray(rows)) {
    for (const row of rows) {
      const name = row?.['交易分类']
      if (!name) continue
      totals.set(name, (totals.get(name) ?? 0) + toNumber(row?.['金额']))
    }
  } else {
    const names = data?.categories?.expense?.names ?? []
    const amounts = data?.categories?.expense?.amounts ?? []
    names.forEach((name, index) => {
      totals.set(name, (totals.get(name) ?? 0) + toNumber(amounts[index]))
    })
  }

  const total = [...totals.values()].reduce((sum, value) => sum + value, 0)
  return [...totals.entries()]
    .map(([name, amount]) => ({
      name,
      amount,
      ratio: total > 0 ? Math.round((amount / total) * 1000) / 10 : 0
    }))
    .sort((a, b) => b.amount - a.amount)
}

/** 把年度分析响应转换成海报可直接绘制的数据模型。 */
export function buildPosterModel(data, year) {
  const stats = data?.yearly_stats ?? {}
  const trends = data?.trends ?? {}

  const months = Array.isArray(trends.months)
    ? trends.months.map((month, index) => ({
        label: monthLabel(month),
        value: toNumber(trends.expenses?.[index])
      }))
    : []

  const peakMonth = months.reduce(
    (peak, month) => (month.value > (peak?.value ?? 0) ? month : peak),
    null
  )

  const expense = toNumber(stats.total_expense)
  const income = toNumber(stats.total_income)

  return {
    year: String(year ?? ''),
    expense,
    income,
    balance: stats.balance === undefined || stats.balance === null
      ? income - expense
      : toNumber(stats.balance),
    totalCount: toNumber(stats.total_count),
    expenseCount: toNumber(stats.expense_count),
    activeDays: toNumber(stats.active_days),
    avgDailyExpense: toNumber(stats.avg_daily_expense),
    months,
    peakMonth,
    topCategories: aggregateCategories(data).slice(0, 5)
  }
}

// ==================== Canvas 绘制 ====================

const font = (weight, size) => `${weight} ${size}px ${FONT_STACK}`

function roundRect(ctx, x, y, width, height, radius) {
  const r = Math.min(radius, width / 2, height / 2)
  ctx.beginPath()
  ctx.moveTo(x + r, y)
  ctx.arcTo(x + width, y, x + width, y + height, r)
  ctx.arcTo(x + width, y + height, x, y + height, r)
  ctx.arcTo(x, y + height, x, y, r)
  ctx.arcTo(x, y, x + width, y, r)
  ctx.closePath()
}

function fitFont(ctx, text, weight, size, maxWidth) {
  let current = size
  ctx.font = font(weight, current)
  while (ctx.measureText(text).width > maxWidth && current > 24) {
    current -= 2
    ctx.font = font(weight, current)
  }
  return current
}

function drawLogo(ctx, logo) {
  const x = 112
  const y = 112
  const size = 96
  ctx.save()
  ctx.beginPath()
  ctx.arc(x + size / 2, y + size / 2, size / 2, 0, Math.PI * 2)
  ctx.closePath()
  ctx.fillStyle = '#E7F1FF'
  ctx.fill()
  ctx.clip()
  if (logo && logo.complete && logo.naturalWidth > 0) {
    ctx.drawImage(logo, x, y, size, size)
  } else {
    ctx.fillStyle = BRAND
    ctx.font = font(600, 40)
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillText('遥', x + size / 2, y + size / 2 + 2)
  }
  ctx.restore()
  ctx.beginPath()
  ctx.arc(x + size / 2, y + size / 2, size / 2, 0, Math.PI * 2)
  ctx.lineWidth = 3
  ctx.strokeStyle = '#D6E7FF'
  ctx.stroke()
}

function drawHeader(ctx, model, options) {
  drawLogo(ctx, options.logo)

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = INK
  ctx.font = font(600, 34)
  ctx.fillText('小遥账单 · 年度账单', 236, 152)

  ctx.fillStyle = MUTED
  ctx.font = font(400, 24)
  ctx.fillText('每一笔支出，都是这一年的注脚', 236, 192)

  const pill = `${model.year} 年${options.isDemo ? ' · 示例数据' : ''}`
  ctx.font = font(500, 24)
  const pillWidth = Math.max(112, ctx.measureText(pill).width + 48)
  const pillX = 968 - pillWidth
  roundRect(ctx, pillX, 116, pillWidth, 52, 26)
  ctx.fillStyle = '#EAF3FF'
  ctx.fill()
  ctx.fillStyle = BRAND
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.fillText(pill, pillX + pillWidth / 2, 143)
}

function drawHero(ctx, model, options) {
  const hide = options.hideAmounts
  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'

  ctx.fillStyle = MUTED
  ctx.font = font(400, 26)
  ctx.fillText('年度支出', 112, 306)

  const amountText = hide ? '••••••' : formatYuan(model.expense)
  const amountFontSize = fitFont(ctx, `¥${amountText}`, 700, 96, 780)
  ctx.fillStyle = INK
  ctx.font = font(700, amountFontSize)
  ctx.fillText('¥', 112, 408)
  const symbolWidth = ctx.measureText('¥').width
  ctx.fillText(amountText, 112 + symbolWidth + 6, 408)

  ctx.fillStyle = MUTED
  ctx.font = font(400, 24)
  ctx.fillText(
    hide ? '日均消费 •• 元' : `日均消费 ¥${formatYuan(model.avgDailyExpense)}`,
    112,
    448
  )

  const cards = [
    { label: '年度收入', value: hide ? '••••' : `¥${formatYuan(model.income)}`, color: INK },
    {
      label: '年度结余',
      value: hide ? '••••' : `${model.balance < 0 ? '-' : ''}¥${formatYuan(Math.abs(model.balance))}`,
      color: model.balance >= 0 ? GREEN : RED
    },
    { label: '交易笔数', value: `${model.totalCount} 笔`, color: INK }
  ]

  cards.forEach((card, index) => {
    const x = 112 + index * 292
    roundRect(ctx, x, 476, 272, 104, 24)
    ctx.fillStyle = PANEL
    ctx.fill()

    ctx.textAlign = 'left'
    ctx.fillStyle = MUTED
    ctx.font = font(400, 22)
    ctx.fillText(card.label, x + 24, 514)

    ctx.fillStyle = card.color
    ctx.font = font(600, 30)
    ctx.fillText(card.value, x + 24, 554)
  })
}

function drawMonths(ctx, model, options) {
  const x = 112
  const y = 608
  const width = 856
  const height = 296

  roundRect(ctx, x, y, width, height, 28)
  ctx.fillStyle = PANEL
  ctx.fill()

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = INK
  ctx.font = font(600, 28)
  ctx.fillText('月度支出趋势', x + 32, y + 52)

  ctx.textAlign = 'right'
  ctx.fillStyle = MUTED
  ctx.font = font(400, 22)
  const peakText = model.peakMonth && !options.hideAmounts
    ? `最高 ${model.peakMonth.label} · ${formatCompactYuan(model.peakMonth.value)}`
    : model.peakMonth
      ? `最高 ${model.peakMonth.label}`
      : ''
  if (peakText) ctx.fillText(peakText, x + width - 32, y + 52)

  const months = model.months
  if (months.length === 0) {
    ctx.textAlign = 'center'
    ctx.fillStyle = MUTED
    ctx.font = font(400, 24)
    ctx.fillText('这一年还没有支出数据', x + width / 2, y + height / 2)
    return
  }

  const peak = Math.max(...months.map((month) => month.value), 0)
  const chartLeft = x + 40
  const chartRight = x + width - 40
  const baseline = y + 232
  const maxBarHeight = 122
  const slot = (chartRight - chartLeft) / months.length
  const barWidth = Math.min(40, slot * 0.52)

  months.forEach((month, index) => {
    const center = chartLeft + slot * index + slot / 2
    const ratio = peak > 0 ? month.value / peak : 0
    const barHeight = Math.max(6, maxBarHeight * ratio)
    const isPeak = peak > 0 && month.value === peak

    roundRect(ctx, center - barWidth / 2, baseline - barHeight, barWidth, barHeight, 10)
    ctx.fillStyle = isPeak ? BRAND : '#C9DDFB'
    ctx.fill()

    ctx.textAlign = 'center'
    ctx.fillStyle = isPeak ? BRAND : MUTED
    ctx.font = font(isPeak ? 600 : 400, 18)
    ctx.fillText(month.label.replace('月', ''), center, baseline + 30)
  })
}

function drawCategories(ctx, model, options) {
  const hide = options.hideAmounts
  const x = 112
  const titleY = 972

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = INK
  ctx.font = font(600, 28)
  ctx.fillText('消费 TOP 5', x, titleY)

  ctx.textAlign = 'right'
  ctx.fillStyle = MUTED
  ctx.font = font(400, 22)
  ctx.fillText('按支出金额排序', 968, titleY)

  if (model.topCategories.length === 0) {
    ctx.textAlign = 'left'
    ctx.fillStyle = MUTED
    ctx.font = font(400, 24)
    ctx.fillText('还没有分类数据', x, titleY + 64)
    return
  }

  const maxAmount = Math.max(...model.topCategories.map((item) => item.amount), 0)
  const palette = ['#007AFF', '#5856D6', '#FF9500', '#34C759', '#FF375F']

  model.topCategories.forEach((item, index) => {
    const rowY = titleY + 44 + index * 48
    const centerY = rowY + 10
    const color = palette[index % palette.length]

    ctx.beginPath()
    ctx.arc(x + 14, centerY - 6, 14, 0, Math.PI * 2)
    ctx.fillStyle = color
    ctx.fill()

    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillStyle = '#FFFFFF'
    ctx.font = font(600, 18)
    ctx.fillText(String(index + 1), x + 14, centerY - 5)

    ctx.textAlign = 'left'
    ctx.textBaseline = 'alphabetic'
    ctx.font = font(600, 26)
    const nameWidth = ctx.measureText(item.name).width
    ctx.fillStyle = INK
    ctx.fillText(item.name, x + 40, rowY + 4)

    ctx.fillStyle = MUTED
    ctx.font = font(400, 21)
    ctx.fillText(`${item.ratio.toFixed(1)}%`, x + 40 + nameWidth + 14, rowY + 4)

    const trackX = x + 40
    const trackWidth = 620
    roundRect(ctx, trackX, rowY + 14, trackWidth, 7, 4)
    ctx.fillStyle = TRACK
    ctx.fill()

    const fillWidth = maxAmount > 0 ? Math.max(10, (item.amount / maxAmount) * trackWidth) : 10
    roundRect(ctx, trackX, rowY + 14, fillWidth, 7, 4)
    ctx.fillStyle = color
    ctx.fill()

    ctx.textAlign = 'right'
    ctx.fillStyle = INK
    ctx.font = font(600, 26)
    ctx.fillText(hide ? '••••' : formatCompactYuan(item.amount), 968, rowY + 4)
  })
}

function drawFooter(ctx, model, assets) {
  ctx.beginPath()
  ctx.moveTo(112, 1256)
  ctx.lineTo(968, 1256)
  ctx.strokeStyle = LINE
  ctx.lineWidth = 2
  ctx.stroke()

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = INK
  ctx.font = font(600, 30)
  ctx.fillText(PRODUCT_NAME, 112, 1306)

  ctx.fillStyle = MUTED
  ctx.font = font(400, 22)
  ctx.fillText(PRODUCT_SLOGAN, 112, 1338)

  ctx.fillStyle = BRAND
  ctx.font = font(400, 19)
  ctx.fillText(PRODUCT_SITE_URL.replace('https://', ''), 112, 1370)

  const qrSize = 96
  const qrX = 856
  const qrY = 1264

  if (assets.qr && assets.qr.complete && assets.qr.naturalWidth > 0) {
    ctx.drawImage(assets.qr, qrX, qrY, qrSize, qrSize)
  } else {
    roundRect(ctx, qrX, qrY, qrSize, qrSize, 12)
    ctx.fillStyle = '#F1F5FB'
    ctx.fill()
    ctx.textAlign = 'center'
    ctx.fillStyle = MUTED
    ctx.font = font(400, 16)
    ctx.textBaseline = 'middle'
    ctx.fillText('二维码', qrX + qrSize / 2, qrY + qrSize / 2)
  }

  ctx.textAlign = 'center'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = MUTED
  ctx.font = font(400, 18)
  ctx.fillText('扫码了解详情', qrX + qrSize / 2, 1380)
}

/**
 * 把海报绘制到 1080×1440 的 canvas。
 *
 * @param {CanvasRenderingContext2D} ctx
 * @param {ReturnType<typeof buildPosterModel>} model
 * @param {{ logo?: HTMLImageElement, qr?: HTMLImageElement }} assets
 * @param {{ hideAmounts?: boolean, isDemo?: boolean }} options
 */
export function drawSharePoster(ctx, model, assets = {}, options = {}) {
  const resolved = { hideAmounts: false, isDemo: false, ...options }

  ctx.save()
  ctx.clearRect(0, 0, POSTER_WIDTH, POSTER_HEIGHT)

  const background = ctx.createLinearGradient(0, 0, POSTER_WIDTH, POSTER_HEIGHT)
  background.addColorStop(0, '#E7F1FF')
  background.addColorStop(0.55, '#F1F7FF')
  background.addColorStop(1, '#FBFDFF')
  ctx.fillStyle = background
  ctx.fillRect(0, 0, POSTER_WIDTH, POSTER_HEIGHT)

  const glow = ctx.createRadialGradient(960, 60, 20, 960, 60, 420)
  glow.addColorStop(0, 'rgba(0, 122, 255, 0.16)')
  glow.addColorStop(1, 'rgba(0, 122, 255, 0)')
  ctx.fillStyle = glow
  ctx.fillRect(520, 0, 560, 520)

  const glow2 = ctx.createRadialGradient(80, 1380, 20, 80, 1380, 380)
  glow2.addColorStop(0, 'rgba(88, 86, 214, 0.10)')
  glow2.addColorStop(1, 'rgba(88, 86, 214, 0)')
  ctx.fillStyle = glow2
  ctx.fillRect(0, 980, 460, 460)

  ctx.shadowColor = 'rgba(11, 27, 51, 0.12)'
  ctx.shadowBlur = 48
  ctx.shadowOffsetY = 18
  roundRect(ctx, 56, 56, 968, 1328, 48)
  ctx.fillStyle = '#FFFFFF'
  ctx.fill()
  ctx.shadowColor = 'transparent'
  ctx.shadowBlur = 0
  ctx.shadowOffsetY = 0

  drawHeader(ctx, model, { ...resolved, logo: assets.logo })
  drawHero(ctx, model, resolved)
  drawMonths(ctx, model, resolved)
  drawCategories(ctx, model, resolved)
  drawFooter(ctx, model, assets)
  ctx.restore()
}
