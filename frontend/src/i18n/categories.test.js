import { describe, expect, it } from 'vitest'

import { CATEGORY_COLORS } from '../utils/format'
import { CATEGORY_LABELS_EN, localizeAmountRange, localizeCategory } from './categories'

describe('分类名本地化', () => {
  it('英文返回映射名，中文保留原文', () => {
    expect(localizeCategory('住房物业', 'en-US')).toBe('Housing')
    expect(localizeCategory('餐饮美食', 'en-US')).toBe('Dining')
    expect(localizeCategory('住房物业', 'zh-CN')).toBe('住房物业')
  })

  it('未知分类与空值保留原文', () => {
    expect(localizeCategory('自定义分类', 'en-US')).toBe('自定义分类')
    expect(localizeCategory('', 'en-US')).toBe('')
  })

  it('来源与星期同样可映射', () => {
    expect(localizeCategory('支付宝', 'en-US')).toBe('Alipay')
    expect(localizeCategory('微信', 'en-US')).toBe('WeChat')
    expect(localizeCategory('周一', 'en-US')).toBe('Mon')
    expect(localizeCategory('周末', 'en-US')).toBe('Weekend')
  })

  it('金额区间转换为英文单位', () => {
    expect(localizeAmountRange('0-100元', 'en-US')).toBe('0–100 CNY')
    expect(localizeAmountRange('1000-5000元', 'en-US')).toBe('1,000–5,000 CNY')
    expect(localizeAmountRange('>5000元', 'en-US')).toBe('> 5,000 CNY')
    expect(localizeAmountRange('0-100元', 'zh-CN')).toBe('0-100元')
  })

  it('所有内置分类颜色都有英文名', () => {
    const missing = Object.keys(CATEGORY_COLORS).filter(
      (name) => !CATEGORY_LABELS_EN[name]
    )
    expect(missing).toEqual([])
  })
})
