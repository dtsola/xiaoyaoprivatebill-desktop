<template>
  <div class="share-poster-inline" :aria-label="t('poster.save')">
    <div class="share-body">
      <div class="share-preview">
        <canvas ref="canvasRef" class="poster-canvas"></canvas>
        <div v-if="!ready" class="poster-loading">
          <div class="poster-spinner"></div>
          <span>{{ t('poster.generating') }}</span>
        </div>
      </div>

      <aside class="share-side">
        <h2 class="share-side-title">{{ t('poster.sideTitleLine1') }}<br />{{ t('poster.sideTitleLine2') }}</h2>
        <p class="share-side-subtitle">
          {{ t('poster.sizeLine') }}<br />
          {{ t('poster.channels') }}
        </p>

        <section class="share-section">
          <h3 class="share-section-title">{{ t('poster.templates') }}</h3>
          <div class="theme-grid">
            <button
              v-for="theme in POSTER_THEMES"
              :key="theme.id"
              type="button"
              class="theme-card"
              :class="{ active: selectedTheme === theme.id }"
              :title="theme.description"
              @click="selectTheme(theme.id)"
            >
              <span
                class="theme-swatch"
                :style="{ background: `linear-gradient(135deg, ${theme.swatch[0]}, ${theme.swatch[1]})` }"
              ></span>
              <span class="theme-name">{{ t(`poster.themeNames.${theme.id}`) }}</span>
            </button>
          </div>
        </section>

        <section class="share-section">
          <div class="share-section-head">
            <h3 class="share-section-title">{{ t('poster.amounts') }}</h3>
            <button type="button" class="link-btn" @click="toggleHideAmounts">
              {{ allAmountsVisible ? t('poster.hideAll') : t('poster.showAll') }}
            </button>
          </div>
          <p v-if="period === 'month'" class="switch-hint">
            {{ t('poster.monthlyHint') }}
          </p>
          <div class="amount-switches">
            <button
              v-for="field in AMOUNT_FIELDS"
              :key="field.key"
              type="button"
              class="switch-row"
              :class="{ on: visibility[field.key] }"
              :aria-pressed="visibility[field.key]"
              @click="toggleAmount(field.key)"
            >
              <span>{{ t(`poster.amountFields.${field.key}`) }}</span>
              <span class="switch-track"><span class="switch-thumb"></span></span>
            </button>
          </div>
        </section>

        <div class="share-actions">
          <button
            type="button"
            class="primary-btn block-btn"
            :disabled="saving || !ready"
            @click="savePoster"
          >
            <Download :size="16" />
            {{ saving ? t('poster.saving') : t('poster.save') }}
          </button>
        </div>

        <p class="share-tip">{{ t('poster.tip') }}</p>
      </aside>
    </div>
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, reactive, ref, watch } from 'vue'
import { Download } from 'lucide-vue-next'
import { save } from '@tauri-apps/plugin-dialog'
import { useI18n } from 'vue-i18n'

import api from '@/api/client'
import { useUiStore } from '@/stores/ui'
import {
  AMOUNT_FIELDS,
  POSTER_HEIGHT,
  POSTER_THEMES,
  POSTER_WIDTH,
  buildPosterModel,
  defaultAmountVisibility,
  drawSharePoster
} from '@/utils/sharePoster'

const props = defineProps({
  data: { type: Object, required: true },
  year: { type: [String, Number], required: true },
  period: { type: String, default: 'year' },
  month: { type: [String, Number], default: '' },
  isDemo: { type: Boolean, default: false }
})

const uiStore = useUiStore()
const { t, locale } = useI18n()
const canvasRef = ref(null)
const ready = ref(false)
const saving = ref(false)
const selectedTheme = ref(POSTER_THEMES[0].id)
const visibility = reactive(defaultAmountVisibility(props.period))
const allAmountsVisible = computed(() =>
  AMOUNT_FIELDS.every((field) => visibility[field.key])
)

let logo = null
let qr = null

/** 加载图片；失败时返回 null，海报会绘制占位内容。 */
function loadImage(src) {
  return new Promise((resolve) => {
    const image = new Image()
    image.onload = () => resolve(image)
    image.onerror = () => resolve(null)
    image.src = src
  })
}

async function loadQr() {
  try {
    const svg = await api.shareQrSvg()
    return await loadImage(`data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`)
  } catch (error) {
    console.error('[SharePoster] 二维码加载失败:', error)
    return null
  }
}

function render() {
  const canvas = canvasRef.value
  if (!canvas) return
  canvas.width = POSTER_WIDTH
  canvas.height = POSTER_HEIGHT
  const context = canvas.getContext('2d')
  drawSharePoster(
    context,
    buildPosterModel(props.data, {
      period: props.period,
      year: props.year,
      month: props.month,
      locale: locale.value
    }),
    { logo, qr },
    {
      theme: selectedTheme.value,
      visibility,
      isDemo: props.isDemo
    }
  )
  ready.value = true
}

function toggleHideAmounts() {
  const next = !allAmountsVisible.value
  AMOUNT_FIELDS.forEach((field) => {
    visibility[field.key] = next
  })
  render()
}

function selectTheme(id) {
  selectedTheme.value = id
  render()
}

function toggleAmount(key) {
  visibility[key] = !visibility[key]
  render()
}

async function savePoster() {
  if (!ready.value || saving.value) return
  if (!qr) {
    uiStore.showError(t('poster.qrNotReady'))
    return
  }

  saving.value = true
  try {
    const suffix =
      props.period === 'month'
        ? t('poster.fileNameMonth', { year: props.year, month: props.month })
        : t('poster.fileNameYear', { year: props.year })
    const target = await save({
      defaultPath: suffix,
      filters: [{ name: t('poster.filterName'), extensions: ['png'] }]
    })
    if (!target) return

    const dataUrl = canvasRef.value.toDataURL('image/png')
    const result = await api.saveSharePoster(target, dataUrl.split(',')[1])
    uiStore.showSuccess(t('poster.saveSuccess', { path: result?.path ?? target }))
  } catch (error) {
    console.error('[SharePoster] 保存失败:', error)
    uiStore.showError(`${t('poster.saveFailed')}: ${error?.message ?? error}`)
  } finally {
    saving.value = false
  }
}

// 切换年度 / 月度时套用该范围的默认金额开关（月度默认隐藏收入与结余）。
watch(
  () => props.period,
  (next) => {
    Object.assign(visibility, defaultAmountVisibility(next))
    if (ready.value) render()
  }
)

// 页面切换年份 / 月份 / 范围会替换 data，必须重绘画布。
watch(
  () => [props.data, props.period, props.year, props.month, props.isDemo, locale.value],
  () => {
    if (ready.value) render()
  }
)

onMounted(async () => {
  const [logoImage, qrImage] = await Promise.all([
    loadImage('/images/logo_128.png'),
    loadQr()
  ])
  logo = logoImage
  qr = qrImage
  await nextTick()
  render()
})
</script>

<style scoped>
.share-poster-inline {
  width: 100%;
}

.share-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 360px;
  gap: 36px;
  align-items: start;
}

.share-preview {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 320px;
}

.poster-canvas {
  display: block;
  width: auto;
  height: min(calc(100vh - 220px), 720px);
  aspect-ratio: 3 / 4;
  border-radius: 16px;
  box-shadow: 0 20px 48px rgba(11, 27, 51, 0.16);
}

.poster-loading {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: var(--secondary-text, #6e6e73);
  background: #eef3fb;
  border-radius: 16px;
}

.poster-spinner {
  width: 26px;
  height: 26px;
  border: 3px solid rgba(0, 122, 255, 0.2);
  border-top-color: #007aff;
  border-radius: 50%;
  animation: poster-spin 0.8s linear infinite;
}

@keyframes poster-spin {
  to {
    transform: rotate(360deg);
  }
}

.share-side {
  position: sticky;
  top: 96px;
  padding: 28px;
  border-radius: 20px;
  background: var(--card-bg, #fff);
  box-shadow: 0 4px 20px rgba(11, 27, 51, 0.06);
}

.share-side-title {
  margin: 0 0 10px;
  font-size: 24px;
  line-height: 1.35;
  font-weight: 600;
  color: var(--text-color, #1c1c1e);
}

.share-side-subtitle {
  margin: 0 0 20px;
  font-size: 14px;
  line-height: 1.6;
  color: var(--secondary-text, #6e6e73);
}

.share-section {
  margin-bottom: 20px;
}

.share-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.share-section-head .share-section-title {
  margin-bottom: 0;
}

.share-section-title {
  margin: 0 0 10px;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--secondary-text, #6e6e73);
}

.link-btn {
  padding: 0;
  border: none;
  background: none;
  color: #007aff;
  font-size: 12px;
  cursor: pointer;
}

.theme-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 6px;
}

.theme-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
  padding: 6px 4px 5px;
  border: 1.5px solid var(--border-color, #e5e5ea);
  border-radius: 10px;
  background: var(--card-bg, #fff);
  cursor: pointer;
  transition: all 0.2s ease;
}

.theme-card:hover {
  border-color: rgba(0, 122, 255, 0.4);
}

.theme-card.active {
  border-color: #007aff;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.12);
}

.theme-swatch {
  width: 100%;
  height: 28px;
  border-radius: 7px;
}

.theme-name {
  font-size: 11px;
  color: var(--text-color, #1c1c1e);
}

.theme-card.active .theme-name {
  color: #007aff;
  font-weight: 600;
}

.switch-hint {
  margin: 0 0 8px;
  font-size: 12px;
  line-height: 1.5;
  color: var(--secondary-text, #6e6e73);
}

.amount-switches {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.switch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 7px 2px;
  border: none;
  background: none;
  font-size: 13px;
  color: var(--text-color, #1c1c1e);
  cursor: pointer;
}

.switch-track {
  position: relative;
  flex: none;
  width: 38px;
  height: 22px;
  border-radius: 999px;
  background: #e2e2e7;
  transition: background 0.2s ease;
}

.switch-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.2s ease;
}

.switch-row.on .switch-track {
  background: #007aff;
}

.switch-row.on .switch-thumb {
  transform: translateX(16px);
}

.share-actions {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  flex-wrap: wrap;
  gap: 12px;
}

.primary-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 38px;
  padding: 0 18px;
  border: none;
  border-radius: 10px;
  background: #007aff;
  color: #fff;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.primary-btn:hover:not(:disabled) {
  background: #0a6fe0;
}

.primary-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.block-btn {
  width: 100%;
  justify-content: center;
}

.share-tip {
  margin: 10px 0 0;
  font-size: 12px;
  color: var(--secondary-text, #6e6e73);
  text-align: left;
}

@media (max-width: 1024px) {
  .share-body {
    grid-template-columns: minmax(0, 1fr);
  }

  .share-side {
    position: static;
  }

  .poster-canvas {
    height: auto;
    width: min(100%, 460px);
  }
}
</style>
