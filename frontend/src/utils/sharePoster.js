/**
 * 账单分享海报：数据模型、模板与 Canvas 绘制。
 *
 * 海报固定 1080×1440（3:4，适合朋友圈 / 小红书 / 微博 / 抖音），完全在本地绘制，
 * 不发起任何网络请求；二维码由 Rust 侧离线生成。
 */

export const POSTER_WIDTH = 1080
export const POSTER_HEIGHT = 1440
export const PRODUCT_SITE_URL = 'https://www.xiaoyaosai.com/xiaoyaoprivatebill-desktop/'
export const PRODUCT_NAME = '小遥账单桌面版'
export const PRODUCT_SLOGAN = '隐私优先 · 完全离线 · 本地分析'

const FONT_STACK = '"PingFang SC", "Microsoft YaHei", "Helvetica Neue", Arial, sans-serif'

/**
 * 海报模板：只改配色与氛围，不改变信息结构。
 */
export const POSTER_THEMES = [
  {
    id: 'sky',
    name: '晴空',
    description: '清爽品牌蓝',
    swatch: ['#E7F1FF', '#007AFF'],
    page: ['#E7F1FF', '#F1F7FF', '#FBFDFF'],
    card: '#FFFFFF',
    ink: '#0B1B33',
    soft: '#5B6B84',
    muted: '#8E9AAF',
    brand: '#007AFF',
    green: '#34C759',
    red: '#FF453A',
    panel: '#F7F9FC',
    track: '#EDF2FA',
    line: '#E6EDF7',
    bar: '#C9DDFB',
    pillBg: '#EAF3FF',
    pillInk: '#007AFF',
    glow1: 'rgba(0, 122, 255, 0.16)',
    glow2: 'rgba(88, 86, 214, 0.10)',
    shadow: 'rgba(11, 27, 51, 0.12)',
    palette: ['#007AFF', '#5856D6', '#FF9500', '#34C759', '#FF375F']
  },
  {
    id: 'ink',
    name: '墨夜',
    description: '深色高级感',
    swatch: ['#0B1528', '#4DA3FF'],
    page: ['#0A1226', '#0F1B33', '#0B1528'],
    card: '#111E38',
    ink: '#F5F8FF',
    soft: '#C3CFE4',
    muted: '#8496B3',
    brand: '#4DA3FF',
    green: '#4ADE80',
    red: '#FF7A7A',
    panel: '#16294A',
    track: '#1E355C',
    line: '#22375C',
    bar: '#2B4A7A',
    pillBg: '#1B3560',
    pillInk: '#8CC4FF',
    glow1: 'rgba(77, 163, 255, 0.24)',
    glow2: 'rgba(124, 92, 255, 0.18)',
    shadow: 'rgba(0, 0, 0, 0.42)',
    palette: ['#4DA3FF', '#8A7CFF', '#FFB020', '#4ADE80', '#FF7AA2']
  },
  {
    id: 'sunset',
    name: '暖阳',
    description: '温暖杂志风',
    swatch: ['#FFF3E6', '#E8663D'],
    page: ['#FFF3E6', '#FFF8F0', '#FFFCF7'],
    card: '#FFFDFB',
    ink: '#3A2416',
    soft: '#7A5A45',
    muted: '#A88973',
    brand: '#E8663D',
    green: '#3D9A6A',
    red: '#D14B3F',
    panel: '#FFF6EC',
    track: '#F5E7D8',
    line: '#F0E0CF',
    bar: '#F3C9A8',
    pillBg: '#FFE9D6',
    pillInk: '#C9552F',
    glow1: 'rgba(232, 102, 61, 0.16)',
    glow2: 'rgba(247, 181, 86, 0.18)',
    shadow: 'rgba(58, 36, 22, 0.14)',
    palette: ['#E8663D', '#C9552F', '#F0A02A', '#3D9A6A', '#C74B7C']
  }
]

/** 海报上可独立开关的金额项。 */
export const AMOUNT_FIELDS = [
  { key: 'expense', label: '支出总额' },
  { key: 'income', label: '收入' },
  { key: 'balance', label: '结余' },
  { key: 'categories', label: '分类金额' },
  { key: 'peak', label: '趋势最高金额' }
]

export const DEFAULT_AMOUNT_VISIBILITY = AMOUNT_FIELDS.reduce((result, field) => {
  result[field.key] = true
  return result
}, {})

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

/** 按开关输出金额或掩码。 */
export function displayAmount(value, visible = true) {
  return visible ? formatYuan(value) : '••••••'
}

const dayNumber = (value) => String(Number(String(value ?? '').split('-')[2]) || '')

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

function buildChart(data, period) {
  const items =
    period === 'month'
      ? (data?.daily_data?.expense?.dates ?? []).map((date, index) => ({
          label: dayNumber(date),
          value: toNumber(data?.daily_data?.expense?.amounts?.[index])
        }))
      : (data?.trends?.months ?? []).map((month, index) => ({
          label: String(Number(String(month).split('-')[1]) || ''),
          value: toNumber(data?.trends?.expenses?.[index])
        }))

  const peak = items.reduce(
    (current, item) => (item.value > (current?.value ?? 0) ? item : current),
    null
  )

  return {
    items,
    peak,
    labelEvery: period === 'month' ? 5 : 1
  }
}

/**
 * 把年度 / 月度分析响应转换成海报可直接绘制的数据模型。
 *
 * @param {object} data 接口返回数据
 * @param {{ period?: 'year' | 'month', year?: number|string, month?: number|string }} options
 */
export function buildPosterModel(data, options = {}) {
  const period = options.period === 'month' ? 'month' : 'year'
  const year = String(options.year ?? '')
  const month = period === 'month' ? String(Number(options.month) || '') : ''

  const stats =
    period === 'month' ? data?.stats ?? {} : data?.yearly_stats ?? {}

  const chart = buildChart(data, period)
  const expense = toNumber(stats.total_expense)
  const income = toNumber(stats.total_income)
  const balance =
    stats.balance === undefined || stats.balance === null
      ? income - expense
      : toNumber(stats.balance)

  const activeDays =
    period === 'month'
      ? chart.items.filter((item) => item.value > 0).length
      : toNumber(stats.active_days)

  const totalCount =
    period === 'month'
      ? toNumber(stats.expense_count) + toNumber(stats.income_count)
      : toNumber(stats.total_count)

  const avgDailyExpense =
    period === 'month'
      ? (chart.items.length > 0 ? expense / chart.items.length : 0)
      : toNumber(stats.avg_daily_expense)

  return {
    period,
    year,
    month,
    titleSuffix: period === 'month' ? '月度账单' : '年度账单',
    periodLabel: period === 'month' ? `${year} 年 ${month} 月` : `${year} 年`,
    heroLabel: period === 'month' ? '本月支出' : '年度支出',
    incomeLabel: period === 'month' ? '本月收入' : '年度收入',
    balanceLabel: period === 'month' ? '本月结余' : '年度结余',
    chartTitle: period === 'month' ? '每日支出' : '月度支出趋势',
    chart,
    expense,
    income,
    balance,
    totalCount,
    activeDays,
    avgDailyExpense,
    topCategories: aggregateCategories(data).slice(0, 5)
  }
}

// ==================== Canvas 绘制 ====================

const font = (weight, size) => `${weight} ${size}px ${FONT_STACK}`

export function getPosterTheme(id) {
  return POSTER_THEMES.find((theme) => theme.id === id) ?? POSTER_THEMES[0]
}

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

function drawLogo(ctx, logo, theme) {
  const x = 112
  const y = 112
  const size = 96
  ctx.save()
  ctx.beginPath()
  ctx.arc(x + size / 2, y + size / 2, size / 2, 0, Math.PI * 2)
  ctx.closePath()
  ctx.fillStyle = theme.pillBg
  ctx.fill()
  ctx.clip()
  if (logo && logo.complete && logo.naturalWidth > 0) {
    ctx.drawImage(logo, x, y, size, size)
  } else {
    ctx.fillStyle = theme.brand
    ctx.font = font(600, 40)
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillText('遥', x + size / 2, y + size / 2 + 2)
  }
  ctx.restore()
  ctx.beginPath()
  ctx.arc(x + size / 2, y + size / 2, size / 2, 0, Math.PI * 2)
  ctx.lineWidth = 3
  ctx.strokeStyle = theme.line
  ctx.stroke()
}

function drawHeader(ctx, model, theme, options) {
  drawLogo(ctx, options.logo, theme)

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.ink
  ctx.font = font(600, 34)
  ctx.fillText(`小遥账单 · ${model.titleSuffix}`, 236, 152)

  ctx.fillStyle = theme.muted
  ctx.font = font(400, 24)
  ctx.fillText('每一笔支出，都是这一年的注脚', 236, 192)

  const pill = `${model.periodLabel}${options.isDemo ? ' · 示例' : ''}`
  ctx.font = font(500, 24)
  const pillWidth = Math.max(112, ctx.measureText(pill).width + 48)
  const pillX = 968 - pillWidth
  roundRect(ctx, pillX, 116, pillWidth, 52, 26)
  ctx.fillStyle = theme.pillBg
  ctx.fill()
  ctx.fillStyle = theme.pillInk
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.fillText(pill, pillX + pillWidth / 2, 143)
}

function drawHero(ctx, model, theme, options) {
  const visibility = options.visibility
  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'

  ctx.fillStyle = theme.muted
  ctx.font = font(400, 26)
  ctx.fillText(model.heroLabel, 112, 306)

  const amountText = displayAmount(model.expense, visibility.expense)
  const amountFontSize = fitFont(ctx, `¥${amountText}`, 700, 96, 780)
  ctx.fillStyle = theme.ink
  ctx.font = font(700, amountFontSize)
  ctx.fillText('¥', 112, 408)
  const symbolWidth = ctx.measureText('¥').width
  ctx.fillText(amountText, 112 + symbolWidth + 6, 408)

  ctx.fillStyle = theme.muted
  ctx.font = font(400, 24)
  ctx.fillText(
    visibility.expense
      ? `日均消费 ¥${formatYuan(model.avgDailyExpense)}`
      : '日均消费 ••••',
    112,
    448
  )

  const cards = [
    {
      label: model.incomeLabel,
      value: visibility.income ? `¥${formatYuan(model.income)}` : '••••',
      color: theme.ink
    },
    {
      label: model.balanceLabel,
      value: visibility.balance
        ? `${model.balance < 0 ? '-' : ''}¥${formatYuan(Math.abs(model.balance))}`
        : '••••',
      color: model.balance >= 0 ? theme.green : theme.red
    },
    {
      label: '交易笔数',
      value: `${model.totalCount} 笔`,
      color: theme.ink
    }
  ]

  cards.forEach((card, index) => {
    const x = 112 + index * 292
    roundRect(ctx, x, 476, 272, 104, 24)
    ctx.fillStyle = theme.panel
    ctx.fill()

    ctx.textAlign = 'left'
    ctx.fillStyle = theme.muted
    ctx.font = font(400, 22)
    ctx.fillText(card.label, x + 24, 514)

    ctx.fillStyle = card.color
    ctx.font = font(600, 30)
    ctx.fillText(card.value, x + 24, 554)
  })
}

function drawChart(ctx, model, theme, options) {
  const visibility = options.visibility
  const x = 112
  const y = 608
  const width = 856
  const height = 296

  roundRect(ctx, x, y, width, height, 28)
  ctx.fillStyle = theme.panel
  ctx.fill()

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.ink
  ctx.font = font(600, 28)
  ctx.fillText(model.chartTitle, x + 32, y + 52)

  ctx.textAlign = 'right'
  ctx.fillStyle = theme.muted
  ctx.font = font(400, 22)
  const peak = model.chart.peak
  const peakText = peak
    ? `最高 ${peak.label}${model.period === 'month' ? '日' : '月'}${
        visibility.peak ? ` · ${formatCompactYuan(peak.value)}` : ''
      }`
    : ''
  if (peakText) ctx.fillText(peakText, x + width - 32, y + 52)

  const items = model.chart.items
  if (items.length === 0) {
    ctx.textAlign = 'center'
    ctx.fillStyle = theme.muted
    ctx.font = font(400, 24)
    ctx.fillText('这段时间还没有支出数据', x + width / 2, y + height / 2)
    return
  }

  const max = Math.max(...items.map((item) => item.value), 0)
  const chartLeft = x + 40
  const chartRight = x + width - 40
  const baseline = y + 232
  const maxBarHeight = 122
  const slot = (chartRight - chartLeft) / items.length
  const barWidth = Math.min(40, slot * 0.56)

  items.forEach((item, index) => {
    const center = chartLeft + slot * index + slot / 2
    const ratio = max > 0 ? item.value / max : 0
    const barHeight = Math.max(6, maxBarHeight * ratio)
    const isPeak = max > 0 && item.value === max

    roundRect(ctx, center - barWidth / 2, baseline - barHeight, barWidth, barHeight, barWidth / 2)
    ctx.fillStyle = isPeak ? theme.brand : theme.bar
    ctx.fill()

    const showLabel =
      (index + 1) % model.chart.labelEvery === 0 || index === 0
    if (showLabel) {
      ctx.textAlign = 'center'
      ctx.fillStyle = isPeak ? theme.brand : theme.muted
      ctx.font = font(isPeak ? 600 : 400, 18)
      ctx.fillText(item.label, center, baseline + 30)
    }
  })
}

function drawCategories(ctx, model, theme, options) {
  const visibility = options.visibility
  const x = 112
  const titleY = 972

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.ink
  ctx.font = font(600, 28)
  ctx.fillText('消费 TOP 5', x, titleY)

  ctx.textAlign = 'right'
  ctx.fillStyle = theme.muted
  ctx.font = font(400, 22)
  ctx.fillText('按支出金额排序', 968, titleY)

  if (model.topCategories.length === 0) {
    ctx.textAlign = 'left'
    ctx.fillStyle = theme.muted
    ctx.font = font(400, 24)
    ctx.fillText('还没有分类数据', x, titleY + 64)
    return
  }

  const maxAmount = Math.max(...model.topCategories.map((item) => item.amount), 0)

  model.topCategories.forEach((item, index) => {
    const rowY = titleY + 44 + index * 48
    const centerY = rowY + 10
    const color = theme.palette[index % theme.palette.length]

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
    ctx.fillStyle = theme.ink
    ctx.fillText(item.name, x + 40, rowY + 4)

    ctx.fillStyle = theme.muted
    ctx.font = font(400, 21)
    ctx.fillText(`${item.ratio.toFixed(1)}%`, x + 40 + nameWidth + 14, rowY + 4)

    const trackX = x + 40
    const trackWidth = 620
    roundRect(ctx, trackX, rowY + 14, trackWidth, 7, 4)
    ctx.fillStyle = theme.track
    ctx.fill()

    const fillWidth = maxAmount > 0 ? Math.max(10, (item.amount / maxAmount) * trackWidth) : 10
    roundRect(ctx, trackX, rowY + 14, fillWidth, 7, 4)
    ctx.fillStyle = color
    ctx.fill()

    ctx.textAlign = 'right'
    ctx.fillStyle = theme.ink
    ctx.font = font(600, 26)
    ctx.fillText(
      visibility.categories ? formatCompactYuan(item.amount) : '••••',
      968,
      rowY + 4
    )
  })
}

function drawFooter(ctx, theme, assets) {
  ctx.beginPath()
  ctx.moveTo(112, 1256)
  ctx.lineTo(968, 1256)
  ctx.strokeStyle = theme.line
  ctx.lineWidth = 2
  ctx.stroke()

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.ink
  ctx.font = font(600, 30)
  ctx.fillText(PRODUCT_NAME, 112, 1306)

  ctx.fillStyle = theme.muted
  ctx.font = font(400, 22)
  ctx.fillText(PRODUCT_SLOGAN, 112, 1338)

  ctx.fillStyle = theme.brand
  ctx.font = font(400, 19)
  ctx.fillText(PRODUCT_SITE_URL.replace('https://', ''), 112, 1370)

  const qrSize = 96
  const qrX = 856
  const qrY = 1264

  if (assets.qr && assets.qr.complete && assets.qr.naturalWidth > 0) {
    ctx.drawImage(assets.qr, qrX, qrY, qrSize, qrSize)
  } else {
    roundRect(ctx, qrX, qrY, qrSize, qrSize, 12)
    ctx.fillStyle = theme.panel
    ctx.fill()
    ctx.textAlign = 'center'
    ctx.fillStyle = theme.muted
    ctx.font = font(400, 16)
    ctx.textBaseline = 'middle'
    ctx.fillText('二维码', qrX + qrSize / 2, qrY + qrSize / 2)
  }

  ctx.textAlign = 'center'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.muted
  ctx.font = font(400, 18)
  ctx.fillText('扫码了解详情', qrX + qrSize / 2, 1380)
}

/**
 * 把海报绘制到 1080×1440 的 canvas。
 *
 * @param {CanvasRenderingContext2D} ctx
 * @param {ReturnType<typeof buildPosterModel>} model
 * @param {{ logo?: HTMLImageElement, qr?: HTMLImageElement }} assets
 * @param {{
 *   theme?: string,
 *   visibility?: Record<string, boolean>,
 *   isDemo?: boolean
 * }} options
 */
export function drawSharePoster(ctx, model, assets = {}, options = {}) {
  const theme = getPosterTheme(options.theme)
  const visibility = { ...DEFAULT_AMOUNT_VISIBILITY, ...(options.visibility ?? {}) }
  const resolved = { visibility, isDemo: Boolean(options.isDemo), logo: assets.logo }

  ctx.save()
  ctx.clearRect(0, 0, POSTER_WIDTH, POSTER_HEIGHT)

  const background = ctx.createLinearGradient(0, 0, POSTER_WIDTH, POSTER_HEIGHT)
  background.addColorStop(0, theme.page[0])
  background.addColorStop(0.55, theme.page[1])
  background.addColorStop(1, theme.page[2])
  ctx.fillStyle = background
  ctx.fillRect(0, 0, POSTER_WIDTH, POSTER_HEIGHT)

  const glow = ctx.createRadialGradient(960, 60, 20, 960, 60, 420)
  glow.addColorStop(0, theme.glow1)
  glow.addColorStop(1, 'rgba(0, 0, 0, 0)')
  ctx.fillStyle = glow
  ctx.fillRect(520, 0, 560, 520)

  const glow2 = ctx.createRadialGradient(80, 1380, 20, 80, 1380, 380)
  glow2.addColorStop(0, theme.glow2)
  glow2.addColorStop(1, 'rgba(0, 0, 0, 0)')
  ctx.fillStyle = glow2
  ctx.fillRect(0, 980, 460, 460)

  ctx.shadowColor = theme.shadow
  ctx.shadowBlur = 48
  ctx.shadowOffsetY = 18
  roundRect(ctx, 56, 56, 968, 1328, 48)
  ctx.fillStyle = theme.card
  ctx.fill()
  ctx.shadowColor = 'transparent'
  ctx.shadowBlur = 0
  ctx.shadowOffsetY = 0

  drawHeader(ctx, model, theme, resolved)
  drawHero(ctx, model, theme, resolved)
  drawChart(ctx, model, theme, resolved)
  drawCategories(ctx, model, theme, resolved)
  drawFooter(ctx, theme, assets)
  ctx.restore()
}
