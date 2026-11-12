<template>
  <Teleport to="body">
    <div class="share-overlay" @click.self="close">
      <div class="share-dialog" role="dialog" aria-modal="true" aria-label="账单分享海报">
        <header class="share-header">
          <div>
            <h2 class="share-title">分享海报</h2>
            <p class="share-subtitle">保存后可发朋友圈、小红书，二维码指向产品官网</p>
          </div>
          <button class="share-close" aria-label="关闭" title="关闭" @click="close">
            <X :size="18" />
          </button>
        </header>

        <div class="share-preview">
          <canvas ref="canvasRef" class="poster-canvas"></canvas>
          <div v-if="!ready" class="poster-loading">
            <div class="poster-spinner"></div>
            <span>正在生成海报...</span>
          </div>
        </div>

        <footer class="share-actions">
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

        <p class="share-tip">图片完全在本机生成；担心隐私时可先隐藏金额再保存。</p>
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
  isDemo: { type: Boolean, default: false }
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
  if (event.key === 'Escape') close()
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
</style>
