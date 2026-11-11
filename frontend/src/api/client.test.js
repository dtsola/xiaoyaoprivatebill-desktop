import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

import { invoke } from '@tauri-apps/api/core'
import api from './client'

beforeEach(() => {
  invoke.mockReset()
})

describe('API 客户端（Tauri IPC）', () => {
  it('GET /available_years 通过 invoke 调用 api_request', async () => {
    invoke.mockResolvedValue({ success: true, years: [2024] })

    await expect(api.getAvailableYears()).resolves.toEqual({ years: [2024] })
    expect(invoke).toHaveBeenCalledWith('api_request', {
      req: {
        method: 'GET',
        path: '/available_years',
        query: {},
        body: null
      }
    })
  })

  it('GET 参数转换为字符串 query，0 不被丢弃', async () => {
    invoke.mockResolvedValue({ success: true, data: { ok: true } })

    await expect(
      api.getYearlyAnalysis({ year: 2024, min_amount: 0, max_amount: null })
    ).resolves.toEqual({ ok: true })

    expect(invoke).toHaveBeenCalledWith('api_request', {
      req: {
        method: 'GET',
        path: '/yearly_analysis',
        query: { year: '2024', min_amount: '0' },
        body: null
      }
    })
  })

  it('业务失败抛出 error 文案', async () => {
    invoke.mockResolvedValue({ success: false, error: '接口不存在: /x' })

    await expect(api.getCategories()).rejects.toThrow('接口不存在: /x')
  })

  it('无 success 字段时原样返回', async () => {
    invoke.mockResolvedValue({ categories: ['餐饮'] })

    await expect(api.getCategoryAnalysis()).resolves.toEqual({ categories: ['餐饮'] })
  })
})
