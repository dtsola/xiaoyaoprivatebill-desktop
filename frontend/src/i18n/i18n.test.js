import { afterEach, describe, expect, it, vi } from 'vitest'

import { currentLocale, resolveLocale, setLocale } from './index'
import enUS from './locales/en-US'
import zhCN from './locales/zh-CN'

function flattenKeys(source, prefix = '', result = []) {
  for (const [key, value] of Object.entries(source)) {
    const path = prefix ? `${prefix}.${key}` : key
    if (value && typeof value === 'object') {
      flattenKeys(value, path, result)
    } else {
      result.push(path)
    }
  }
  return result.sort()
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('语言解析', () => {
  it('显式语言直接返回', () => {
    expect(resolveLocale('zh-CN')).toBe('zh-CN')
    expect(resolveLocale('en-US')).toBe('en-US')
  })

  it('system 跟随中文 / 非中文系统', () => {
    vi.stubGlobal('navigator', { language: 'zh-CN' })
    expect(resolveLocale('system')).toBe('zh-CN')

    vi.stubGlobal('navigator', { language: 'en-GB' })
    expect(resolveLocale('system')).toBe('en-US')

    vi.stubGlobal('navigator', { language: 'ja-JP' })
    expect(resolveLocale('system')).toBe('en-US')
  })

  it('未知偏好按系统语言处理', () => {
    vi.stubGlobal('navigator', { language: 'zh-TW' })
    expect(resolveLocale('fr-FR')).toBe('zh-CN')
  })

  it('切换语言同步 html lang 与标题', () => {
    vi.stubGlobal('document', {
      documentElement: { lang: '' },
      title: ''
    })

    expect(setLocale('en-US')).toBe('en-US')
    expect(currentLocale()).toBe('en-US')
    expect(document.documentElement.lang).toBe('en-US')
    expect(document.title).toBe('Xiaoyao Bill Desktop')

    setLocale('zh-CN')
    expect(document.title).toBe('小遥账单桌面版')
  })
})

describe('词条完整性', () => {
  it('中英文 key 完全一致', () => {
    expect(flattenKeys(enUS)).toEqual(flattenKeys(zhCN))
  })

  it('没有空词条', () => {
    const collect = (source, prefix = '', values = []) => {
      for (const [key, value] of Object.entries(source)) {
        const path = prefix ? `${prefix}.${key}` : key
        if (value && typeof value === 'object') {
          collect(value, path, values)
        } else {
          values.push([path, value])
        }
      }
      return values
    }

    for (const [path, value] of collect(zhCN)) {
      expect(String(value).trim(), path).not.toBe('')
    }
    for (const [path, value] of collect(enUS)) {
      expect(String(value).trim(), path).not.toBe('')
    }
  })
})
