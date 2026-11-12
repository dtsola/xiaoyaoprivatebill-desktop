<template>
  <Teleport to="body" :disabled="inline">
    <div
      :class="inline ? 'share-inline-wrap' : 'share-overlay'"
      @click.self="!inline && close()"
    >
      <div
        class="share-dialog"
        :class="{ 'is-inline': inline }"
        role="dialog"
        :aria-modal="inline ? undefined : 'true'"
        aria-label="账单分享海报"
      >
        <header v-if="!inline" class="share-header">
          <div>
            <h2 class="share-title">分享海报</h2>
            <p class="share-subtitle">保存后可发朋友圈、小红书，二维码指向产品官网</p>
          </div>
          <button class="share-close" aria-label="关闭" title="关闭" @click="close">
            <X :size="18" />
          </button>
        </header>

        <div class="share-body">
          <div class="share-preview">
            <canvas ref="canvasRef" class="poster-canvas"></canvas>
            <div v-if="!ready" class="poster-loading">
              <div class="poster-spinner"></div>
              <span>正在生成海报...</span>
            </div>
          </div>

          <aside v-if="inline" class="share-side">
            <h2 class="share-side-title">把这一年的账单，<br />变成一张海报</h2>
            <p class="share-side-subtitle">1080×1440 竖版，适合发朋友圈、小红书。</p>

            <ul class="share-points">
              <li>年度收支、月度趋势与消费 TOP 5</li>
              <li>可隐藏金额，只保留分类占比</li>
              <li>自带产品官网二维码，扫码即可了解</li>
            </ul>

            <div class="share-actions">
              <button type="button" class="ghost-btn" @click="toggleHideAmounts">
                <Eye v-if="hideAmounts" :size="16" />
                <EyeOff v-else :size="16" />
                {{ hideAmounts ? '显示金额' : '隐藏金额' }}
              </button>
              <button type="button" class="primary-btn" :disabled="saving || !ready" @click="savePoster">
                <Download :size="16" />
                {{ saving ? '保存中...' : '保存图片' }}
              </button>
            </div>

            <p class="share-tip">图片完全在本机生成；担心隐私时可先隐藏金额再保存。</p>
          </aside>
        </div>

        <footer v-if="!inline" class="share-actions">
          <button type="button" class="ghost-btn" @click="toggleHideAmounts">
            <Eye v-if="hideAmounts" :size="16" />
            <EyeOff v-else :size="16" />
            {{ hideAmounts ? '显示金额' : '隐藏金额' }}
          </button>
          <button type="button" class="primary-btn" :disabled="saving || !ready" @click="savePoster">
            <Download :size="16" />
            {{ saving ? '保存中...' : '保存图片' }}
          </button>
        </footer>

        <p v-if="!inline" class="share-tip">图片完全在本机生成；担心隐私时可先隐藏金额再保存。</p>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import { Download, Eye, EyeOff, X } from 'lucide-vue-next'
import { save } from '@tauri-apps/plugin-dialog'

import api from '@/api/client'
import { useUiStore } from '@/stores/ui'
import {
  POSTER_HEIGHT,
  POSTER_WIDTH,
  buildPosterModel,
  drawSharePoster
} from '@/utils/sharePoster'

const props = defineProps({
  data: { type: Object, required: true },
  year: { type: [String, Number], required: true },
  isDemo: { type: Boolean, default: false },
  // 页面内嵌模式：不显示遮罩与关闭按钮，由页面提供标题与年份切换
  inline: { type: Boolean, default: false }
})

const emit = defineEmits(['close'])

const uiStore = useUiStore()
const canvasRef = ref(null)
const ready = ref(false)
const saving = ref(false)
const hideAmounts = ref(false)

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
    buildPosterModel(props.data, props.year),
    { logo, qr },
    { hideAmounts: hideAmounts.value, isDemo: props.isDemo }
  )
  ready.value = true
}

function toggleHideAmounts() {
  hideAmounts.value = !hideAmounts.value
  render()
}

function close() {
  emit('close')
}

function handleKeydown(event) {
  if (event.key === 'Escape' && !props.inline) close()
}

async function savePoster() {
  if (!ready.value || saving.value) return
  if (!qr) {
    uiStore.showError('二维码尚未加载完成，请稍后再试')
    return
  }

  saving.value = true
  try {
    const target = await save({
      defaultPath: `小遥账单-${props.year}年度账单.png`,
      filters: [{ name: 'PNG 图片', extensions: ['png'] }]
    })
    if (!target) return

    const dataUrl = canvasRef.value.toDataURL('image/png')
    const result = await api.saveSharePoster(target, dataUrl.split(',')[1])
    uiStore.showSuccess(`海报已保存：${result?.path ?? target}`)
  } catch (error) {
    console.error('[SharePoster] 保存失败:', error)
    uiStore.showError(`保存海报失败: ${error?.message ?? error}`)
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)
  const [logoImage, qrImage] = await Promise.all([
    loadImage('/images/logo_128.png'),
    loadQr()
  ])
  logo = logoImage
  qr = qrImage
  await nextTick()
  render()
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.share-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(11, 27, 51, 0.45);
  backdrop-filter: blur(6px);
}

.share-dialog {
  width: min(760px, 92vw);
  max-height: 92vh;
  display: flex;
  flex-direction: column;
  padding: 20px 24px 16px;
  border-radius: 20px;
  background: var(--card-bg, #fff);
  box-shadow: 0 24px 64px rgba(11, 27, 51, 0.28);
}

.share-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 12px;
}

.share-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color, #1c1c1e);
}

.share-subtitle {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--secondary-text, #6e6e73);
}

.share-close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--secondary-text, #6e6e73);
  cursor: pointer;
  transition: background 0.2s ease;
}

.share-close:hover {
  background: rgba(0, 0, 0, 0.06);
}

.share-body {
  flex: 1;
  min-height: 0;
  display: flex;
}

.share-preview {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border-radius: 14px;
  background: #eef3fb;
}

.poster-canvas {
  display: block;
  width: auto;
  height: 100%;
  max-height: min(62vh, 760px);
  aspect-ratio: 3 / 4;
  border-radius: 12px;
  box-shadow: 0 12px 32px rgba(11, 27, 51, 0.16);
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

.share-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-top: 14px;
}

.ghost-btn,
.primary-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 38px;
  padding: 0 18px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.ghost-btn {
  border: 1px solid var(--border-color, #e5e5ea);
  background: transparent;
  color: var(--text-color, #1c1c1e);
}

.ghost-btn:hover {
  background: rgba(0, 0, 0, 0.04);
}

.primary-btn {
  border: none;
  background: #007aff;
  color: #fff;
}

.primary-btn:hover:not(:disabled) {
  background: #0a6fe0;
}

.primary-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.share-tip {
  margin: 10px 0 0;
  font-size: 12px;
  color: var(--secondary-text, #6e6e73);
  text-align: center;
}

/* 页面内嵌模式 */
.share-inline-wrap {
  width: 100%;
}

.share-dialog.is-inline {
  width: 100%;
  max-width: none;
  max-height: none;
  padding: 0;
  background: transparent;
  box-shadow: none;
}

.is-inline .share-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 360px;
  gap: 36px;
  align-items: start;
}

.is-inline .share-preview {
  background: transparent;
  overflow: visible;
  border-radius: 0;
}

.is-inline .poster-canvas {
  width: auto;
  height: min(calc(100vh - 220px), 720px);
  max-height: none;
  border-radius: 16px;
  box-shadow: 0 20px 48px rgba(11, 27, 51, 0.16);
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

.share-points {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin: 0 0 24px;
  padding: 0;
  list-style: none;
}

.share-points li {
  position: relative;
  padding-left: 24px;
  font-size: 14px;
  line-height: 1.5;
  color: var(--text-color, #1c1c1e);
}

.share-points li::before {
  content: '✓';
  position: absolute;
  left: 0;
  top: 0;
  font-weight: 700;
  color: #007aff;
}

.is-inline .share-actions {
  justify-content: flex-start;
  flex-wrap: wrap;
  padding-top: 0;
}

.is-inline .share-tip {
  margin-top: 14px;
  text-align: left;
}

@media (max-width: 1024px) {
  .is-inline .share-body {
    grid-template-columns: minmax(0, 1fr);
  }

  .share-side {
    position: static;
  }

  .is-inline .poster-canvas {
    height: auto;
    width: min(100%, 460px);
  }
}
</style>
