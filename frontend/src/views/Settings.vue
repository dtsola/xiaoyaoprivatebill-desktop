<template>
  <div class="settings-page">
    <!-- 页面标题：设置页按「账单文件 / 数据管理 / 存储与审计 / 桌面」分组；/agent 复用本页只渲染 MCP 区块 -->
    <div class="page-header" :class="{ 'page-header-agent': agentOnly }">
      <h1 class="page-title">{{ agentOnly ? t('settings.agentTitle') : t('settings.title') }}</h1>
      <p v-if="agentOnly" class="page-subtitle">
        {{ t('settings.agentSubtitle') }}
      </p>
    </div>

    <!-- 账单文件：导入与已上传列表 -->
    <h2 v-if="!agentOnly" class="section-title">{{ t('settings.bills.title') }}</h2>
    <div v-if="!agentOnly" class="split-layout">
      <!-- 左侧：支付宝专区 -->
      <div class="provider-column alipay-column">
        <h2 class="column-header">
          <i class="fab fa-alipay"></i> {{ t('settings.bills.alipayTitle') }}
        </h2>

        <!-- 支付宝指南 -->
        <div class="upload-guide">
          <h3>{{ t('settings.bills.alipayGuideTitle') }}</h3>
          <ol>
            <li v-for="(step, index) in alipaySteps" :key="index" v-html="step"></li>
          </ol>
        </div>

        <!-- 支付宝上传区域 -->
        <div
          class="file-upload"
          :class="{ dragover: alipayDragging }"
          @click="pickFiles('alipay')"
        >
          <div class="upload-icon">
            <FileSpreadsheet :size="'1em'" />
          </div>
          <div class="upload-text">
            {{ t('settings.bills.pickAlipay') }}<br />{{ t('settings.bills.pickAlipaySub') }}
            <br /><span style="font-size: 12px; opacity: 0.7;">{{ t('settings.bills.maxSize', { size: '16MB' }) }}</span>
          </div>
        </div>

        <!-- 支付宝文件列表 -->
        <div class="file-list-container">
          <h4>{{ t('settings.bills.uploadedFiles') }}</h4>
          <div class="file-list">
            <div v-for="file in alipayFiles" :key="file.name" class="file-item">
              <FileSpreadsheet :size="'1em'" />
              <span class="file-name">{{ file.name }}</span>
              <div class="file-actions">
                <span class="file-status status-success">{{ t('settings.bills.uploadedStatus') }}</span>
                <button class="delete-btn" @click="deleteFile(file.name)">
                  <Trash2 :size="'1em'" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧：微信专区 -->
      <div class="provider-column wechat-column">
        <h2 class="column-header">
          <i class="fab fa-weixin"></i> {{ t('settings.bills.wechatTitle') }}
        </h2>

        <!-- 微信指南 -->
        <div class="upload-guide">
          <h3>{{ t('settings.bills.wechatGuideTitle') }}</h3>
          <ol>
            <li v-for="(step, index) in wechatSteps" :key="index" v-html="step"></li>
          </ol>
        </div>

        <!-- 微信上传区域 -->
        <div
          class="file-upload"
          :class="{ dragover: wechatDragging }"
          @click="pickFiles('wechat')"
        >
          <div class="upload-icon">
            <FileText :size="'1em'" />
          </div>
          <div class="upload-text">
            {{ t('settings.bills.pickWechat') }}<br />{{ t('settings.bills.pickWechatSub') }}
            <br /><span style="font-size: 12px; opacity: 0.7;">{{ t('settings.bills.maxSize', { size: '16MB' }) }}</span>
          </div>
        </div>

        <!-- 微信文件列表 -->
        <div class="file-list-container">
          <h4>{{ t('settings.bills.uploadedFiles') }}</h4>
          <div class="file-list">
            <div v-for="file in wechatFiles" :key="file.name" class="file-item">
              <FileSpreadsheet :size="'1em'" />
              <span class="file-name">{{ file.name }}</span>
              <div class="file-actions">
                <span class="file-status status-success">{{ t('settings.bills.uploadedStatus') }}</span>
                <button class="delete-btn" @click="deleteFile(file.name)">
                  <Trash2 :size="'1em'" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部操作栏 -->
    <div class="bottom-action-bar" :class="{ visible: totalFileCount > 0 }">
      <div class="action-bar-content">
        <div class="file-summary">
          <FileText :size="'1em'" />
          <span>{{ totalFileCount > 0 ? t('settings.bills.ready', { count: totalFileCount }) : t('settings.bills.needUpload') }}</span>
        </div>
        <div class="action-buttons">
          <router-link to="/yearly" class="start-button" :class="{ disabled: totalFileCount === 0 }">
            <ChartLine :size="'1em'" />
            {{ t('settings.bills.start') }}
          </router-link>
        </div>
      </div>
    </div>

    <!-- 删除区域 -->
    <div v-if="!agentOnly" class="delete-section">
      <h2 class="section-title">{{ t('settings.data.title') }}</h2>
      <div class="delete-area">
        <button class="delete-all-btn" @click="handleClearAllData">
          <Trash2 :size="'1em'" />
          {{ t('settings.data.deleteAll') }}
        </button>
        <p class="delete-warning">{{ t('settings.data.warning') }}</p>
      </div>
    </div>

    <!-- 存储与审计：数据目录与可恢复数据 -->
    <div v-if="!agentOnly" class="desktop-section">
      <h2 class="section-title">{{ t('settings.storage.title') }}</h2>
      <div class="desktop-card">
        <div class="desktop-row">
          <span class="mcp-label">{{ t('settings.storage.dataDir') }}</span>
          <code class="mcp-endpoint">{{ desktop.data_dir }}</code>
          <button class="mcp-btn" @click="openDataDir">{{ t('settings.storage.open') }}</button>
        </div>
        <div class="desktop-row">
          <span class="mcp-label">{{ t('settings.storage.trash') }}</span>
          <span class="storage-value">{{ t('settings.storage.trashCount', { count: storage.trash_count }) }}</span>
          <button class="mcp-btn" @click="openStorageDir('trash')">{{ t('settings.storage.open') }}</button>
        </div>
        <div class="desktop-row">
          <span class="mcp-label">{{ t('settings.storage.snapshots') }}</span>
          <span class="storage-value">{{ t('settings.storage.snapshotCount', { count: storage.snapshot_count }) }}</span>
          <button class="mcp-btn" @click="openStorageDir('snapshots')">{{ t('settings.storage.open') }}</button>
        </div>
        <div class="desktop-row">
          <span class="mcp-label">{{ t('settings.storage.audit') }}</span>
          <span class="storage-value">{{ auditSizeText }}</span>
        </div>
      </div>
    </div>

    <!-- 桌面行为 -->
    <div v-if="!agentOnly" class="desktop-section">
      <h2 class="section-title">{{ t('settings.desktop.title') }}</h2>
      <div class="desktop-card">
        <div class="desktop-row">
          <span class="mcp-label">{{ t('settings.language.label') }}</span>
          <select
            class="language-select"
            :value="desktop.language"
            :aria-label="t('settings.language.label')"
            @change="changeLanguage($event.target.value)"
          >
            <option value="system">{{ t('settings.language.system') }}</option>
            <option value="zh-CN">{{ t('settings.language.zh') }}</option>
            <option value="en-US">{{ t('settings.language.en') }}</option>
          </select>
        </div>
        <div class="desktop-row">
          <span class="mcp-label">{{ t('settings.desktop.closeBehavior') }}</span>
          <button class="mcp-btn" @click="toggleCloseToTray">
            {{ desktop.close_to_tray ? t('settings.desktop.minimize') : t('settings.desktop.quit') }}
          </button>
        </div>
      </div>
    </div>

    <!-- MCP 服务 -->
    <div v-if="agentOnly" class="mcp-section">
      <div class="mcp-card">
        <div class="mcp-row">
          <span class="mcp-label">{{ t('settings.mcp.status') }}</span>
          <span class="mcp-status" :class="{ running: mcp.running }">
            {{ mcp.running ? t('settings.mcp.running') : t('settings.mcp.stopped') }}
          </span>
        </div>
        <div class="mcp-row">
          <span class="mcp-label">{{ t('settings.mcp.endpoint') }}</span>
          <code class="mcp-endpoint">{{ mcp.endpoint }}</code>
        </div>
        <div v-if="mcp.running && mcp.token" class="mcp-row">
          <span class="mcp-label">{{ t('settings.mcp.token') }}</span>
          <code class="mcp-token">{{ mcp.token }}</code>
          <button class="mcp-btn" @click="copyToken">{{ t('settings.mcp.copy') }}</button>
        </div>
        <div v-if="mcp.running && mcp.token" class="mcp-config">
          <div class="mcp-config-head">
            <span class="mcp-label">{{ t('settings.mcp.config') }}</span>
            <button class="mcp-btn" @click="copyMcpConfig">{{ t('settings.mcp.copyConfig') }}</button>
          </div>
          <pre class="mcp-config-code">{{ mcpConfigJson }}</pre>
          <p class="mcp-hint">
            {{ t('settings.mcp.configHint') }}
          </p>
        </div>
        <p v-if="mcp.last_error" class="mcp-error">{{ mcp.last_error }}</p>
        <div class="mcp-actions">
          <button class="mcp-btn primary" @click="toggleMcp">
            {{ mcp.enabled ? t('settings.mcp.disable') : t('settings.mcp.enable') }}
          </button>
          <button class="mcp-btn" @click="rotateToken">{{ t('settings.mcp.rotate') }}</button>
        </div>
        <p class="mcp-hint">
          {{ t('settings.mcp.securityHint') }}
        </p>
      </div>

      <div class="claw-guide">
        <div class="claw-guide-head">
          <div>
            <h3>{{ t('settings.claw.title') }}</h3>
            <p>{{ t('settings.claw.desc') }}</p>
          </div>
          <span class="claw-badge">{{ t('settings.claw.recommended') }}</span>
        </div>
        <ol class="claw-steps">
          <li class="claw-step">
            <span class="claw-step-num">1</span>
            <div class="claw-step-body">
              <strong>{{ t('settings.claw.step1Title') }}</strong>
              <p>{{ t('settings.claw.step1Desc') }}</p>
            </div>
          </li>
          <li class="claw-step">
            <span class="claw-step-num">2</span>
            <div class="claw-step-body">
              <strong>{{ t('settings.claw.step2Title') }}</strong>
              <p v-html="t('settings.claw.step2Desc')"></p>
            </div>
          </li>
          <li class="claw-step">
            <span class="claw-step-num">3</span>
            <div class="claw-step-body">
              <strong>{{ t('settings.claw.step3Title') }}</strong>
              <p>{{ t('settings.claw.step3Desc') }}</p>
            </div>
          </li>
          <li class="claw-step">
            <span class="claw-step-num">4</span>
            <div class="claw-step-body">
              <strong>{{ t('settings.claw.step4Title') }}</strong>
              <p>{{ t('settings.claw.step4Desc') }}</p>
            </div>
          </li>
        </ol>
        <p class="claw-shots-title">{{ t('settings.claw.shotsTitle') }}</p>
        <div class="claw-shots">
          <button
            v-for="shot in clawShots"
            :key="shot.src"
            type="button"
            class="claw-shot"
            @click="previewShot = shot"
          >
            <img :src="shot.src" :alt="shot.alt" loading="lazy">
            <span>{{ shot.caption }}</span>
          </button>
        </div>
        <div class="claw-guide-foot">
          <button class="mcp-btn" @click="openClawSite">{{ t('settings.claw.site') }}</button>
          <span>{{ t('settings.claw.runningHint') }}</span>
        </div>
      </div>

      <div
        v-if="previewShot"
        class="claw-preview"
        role="dialog"
        aria-modal="true"
        :aria-label="previewShot.caption"
        @click.self="previewShot = null"
      >
        <button class="claw-preview-close" :aria-label="t('settings.claw.closePreview')" @click="previewShot = null">×</button>
        <img :src="previewShot.src" :alt="previewShot.alt">
      </div>
    </div>
  </div>
</template>

<script setup>
import { ChartLine, FileSpreadsheet, FileText, Trash2 } from 'lucide-vue-next'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { useSessionStore } from '@/stores/session'
import { useUiStore } from '@/stores/ui'
import { useI18n } from 'vue-i18n'
import { resolveLocale, setLocale } from '@/i18n'
import api from '@/api/client'

const props = defineProps({
  // /agent 路由复用本组件，只渲染 MCP 区块
  agentOnly: { type: Boolean, default: false }
})

const sessionStore = useSessionStore()
const uiStore = useUiStore()
const { t, tm } = useI18n()

// 文件列表
const alipayFiles = ref([])
const wechatFiles = ref([])

// 拖拽状态
const alipayDragging = ref(false)
const wechatDragging = ref(false)

// 桌面设置
const desktop = ref({
  close_to_tray: true,
  language: 'system',
  data_dir: ''
})

const storage = ref({
  trash_count: 0,
  snapshot_count: 0,
  audit_bytes: 0
})

// MCP 服务状态
const mcp = ref({
  enabled: false,
  running: false,
  endpoint: 'http://127.0.0.1:17650/mcp',
  token: '',
  last_error: null
})

// 小遥Claw 接入实操截图（点击放大）
const alipaySteps = computed(() => tm('settings.bills.alipaySteps'))
const wechatSteps = computed(() => tm('settings.bills.wechatSteps'))

const clawShots = computed(() => [
  {
    src: '/images/claw/01-add-mcp.png',
    alt: t('settings.claw.shotAddAlt'),
    caption: t('settings.claw.shotAddCaption')
  },
  {
    src: '/images/claw/02-connected.png',
    alt: t('settings.claw.shotConnectedAlt'),
    caption: t('settings.claw.shotConnectedCaption')
  },
  {
    src: '/images/claw/03-ask.png',
    alt: t('settings.claw.shotAskAlt'),
    caption: t('settings.claw.shotAskCaption')
  },
  {
    src: '/images/claw/04-analysis.png',
    alt: t('settings.claw.shotAnalysisAlt'),
    caption: t('settings.claw.shotAnalysisCaption')
  }
])
const previewShot = ref(null)

let unlistenDragDrop = null
let unlistenMcp = null

// 总文件数
const totalFileCount = computed(() => alipayFiles.value.length + wechatFiles.value.length)

onMounted(async () => {
  if (!props.agentOnly) {
    await loadFiles()
    await listenDragDrop()
    await loadDesktop()
    await loadStorage()
  }
  await loadMcp()
  unlistenMcp = await listen('desktop://mcp-status', async (event) => {
    mcp.value = { ...mcp.value, ...event.payload, token: '' }
    if (event.payload?.running) {
      const config = await api.mcpConfig()
      mcp.value = { ...mcp.value, ...config }
    }
  })
})

onUnmounted(() => {
  if (unlistenDragDrop) {
    unlistenDragDrop()
    unlistenDragDrop = null
  }
  if (unlistenMcp) {
    unlistenMcp()
    unlistenMcp = null
  }
})

// ==================== 桌面设置 ====================

const auditSizeText = computed(() => {
  const bytes = storage.value.audit_bytes
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
})

async function loadStorage() {
  try {
    storage.value = { ...storage.value, ...(await api.storageStatus()) }
  } catch (error) {
    console.error('[Settings] 读取存储概览失败:', error)
  }
}

async function loadDesktop() {
  try {
    desktop.value = { ...desktop.value, ...(await api.desktopSettings()) }
  } catch (error) {
    console.error('[Settings] 读取桌面设置失败:', error)
  }
}

async function changeLanguage(preference) {
  const previous = desktop.value.language
  const locale = resolveLocale(preference)
  try {
    await api.setLanguage(preference, locale)
    desktop.value = { ...desktop.value, language: preference }
    setLocale(locale)
    uiStore.showSuccess(t('settings.language.saved'))
  } catch (error) {
    desktop.value = { ...desktop.value, language: previous }
    uiStore.showError(`${t('settings.language.failed')}: ${error.message}`)
  }
}

async function toggleCloseToTray() {
  try {
    const result = await api.setCloseToTray(!desktop.value.close_to_tray)
    desktop.value = { ...desktop.value, ...result }
    uiStore.showSuccess(result.close_to_tray ? t('settings.desktop.minimizedToast') : t('settings.desktop.quitToast'))
  } catch (error) {
    uiStore.showError(t('settings.desktop.saveFailed') + ': ' + error.message)
  }
}

async function openStorageDir(kind) {
  try {
    await api.openStorageDir(kind)
  } catch (error) {
    uiStore.showError(t('settings.errors.openDirFailed') + ': ' + error.message)
  }
}

async function openDataDir() {
  try {
    await api.openDataDir()
  } catch (error) {
    uiStore.showError(t('settings.errors.openDataDirFailed') + ': ' + error.message)
  }
}

// ==================== MCP 服务 ====================

async function loadMcp() {
  try {
    const status = await api.mcpStatus()
    mcp.value = { ...mcp.value, ...status, token: '' }
    if (status.running) {
      const config = await api.mcpConfig()
      mcp.value = { ...mcp.value, ...config }
    }
  } catch (error) {
    console.error('[Settings] 读取 MCP 状态失败:', error)
  }
}

async function toggleMcp() {
  try {
    const next = !mcp.value.enabled
    const status = await api.mcpSetEnabled(next)
    mcp.value = { ...mcp.value, ...status, token: '' }
    if (status.running) {
      const config = await api.mcpConfig()
      mcp.value = { ...mcp.value, ...config }
      uiStore.showSuccess(t('settings.mcp.enabledToast'))
    } else if (status.last_error) {
      uiStore.showError(status.last_error)
    } else {
      uiStore.showSuccess(t('settings.mcp.disabledToast'))
    }
  } catch (error) {
    uiStore.showError(t('settings.mcp.opFailed') + ': ' + error.message)
  }
}

async function rotateToken() {
  try {
    const config = await api.mcpRotateToken()
    mcp.value = { ...mcp.value, ...config }
    uiStore.showSuccess(t('settings.mcp.tokenRotated'))
  } catch (error) {
    uiStore.showError(t('settings.mcp.rotateFailed') + ': ' + error.message)
  }
}

async function copyToken() {
  try {
    await navigator.clipboard.writeText(mcp.value.token)
    uiStore.showSuccess(t('settings.mcp.tokenCopied'))
  } catch (error) {
    uiStore.showError(t('settings.mcp.copyFailed') + ': ' + error.message)
  }
}

// 小遥Claw 标准 MCP 服务格式：http 传输 + Bearer Token
const mcpConfigJson = computed(() =>
  JSON.stringify(
    {
      type: 'http',
      url: mcp.value.endpoint,
      headers: {
        Authorization: `Bearer ${mcp.value.token}`
      }
    },
    null,
    2
  )
)

async function copyMcpConfig() {
  try {
    await navigator.clipboard.writeText(mcpConfigJson.value)
    uiStore.showSuccess(t('settings.mcp.configCopied'))
  } catch (error) {
    uiStore.showError(t('settings.mcp.copyFailed') + ': ' + error.message)
  }
}

async function openClawSite() {
  try {
    await openUrl('https://www.xiaoyaosai.com/xiaoyaoclaw/')
  } catch (error) {
    uiStore.showError(t('settings.claw.openSiteFailed') + ': ' + error.message)
  }
}

// 监听系统拖拽事件：路径由 Tauri 提供，统一交给后端按内容识别来源
async function listenDragDrop() {
  try {
    const webview = getCurrentWebview()
    unlistenDragDrop = await webview.onDragDropEvent((event) => {
      const type = event.payload.type
      if (type === 'over') {
        alipayDragging.value = true
        wechatDragging.value = true
      } else if (type === 'drop') {
        alipayDragging.value = false
        wechatDragging.value = false
        handlePaths(event.payload.paths || [])
      } else {
        alipayDragging.value = false
        wechatDragging.value = false
      }
    })
  } catch (error) {
    console.warn('[Settings] 拖拽监听不可用:', error)
  }
}

// 打开系统文件对话框
async function pickFiles(provider) {
  if (!checkDemoMode()) return

  const filters = provider === 'wechat'
    ? [{ name: t('settings.bills.wechatTitle'), extensions: ['xlsx', 'csv'] }]
    : [{ name: t('settings.bills.alipayTitle'), extensions: ['csv'] }]

  try {
    const selected = await open({ multiple: true, filters })
    if (!selected) return
    await handlePaths(Array.isArray(selected) ? selected : [selected], provider)
  } catch (error) {
    uiStore.showError(t('settings.errors.pickerFailed') + ': ' + error.message)
  }
}

// 加载已上传文件
async function loadFiles() {
  try {
    const data = await api.getFiles()
    console.log('[Settings] Loaded files:', data)

    // 清空现有列表
    alipayFiles.value = []
    wechatFiles.value = []

    // 根据文件来源分类
    data.files.forEach(file => {
      // 使用后端返回的 source 字段判断，如果不存在则回退到扩展名判断
      let isWechat = false
      if (file.source) {
        isWechat = file.source === 'wechat'
      } else {
        isWechat = file.name.endsWith('.xlsx')
      }

      if (isWechat) {
        wechatFiles.value.push(file)
      } else {
        alipayFiles.value.push(file)
      }
    })

    // 排序文件列表
    sortFileList(alipayFiles.value)
    sortFileList(wechatFiles.value)
  } catch (error) {
    console.error('[Settings] 加载文件列表失败:', error)
  }
}

// 排序文件列表
function sortFileList(fileList) {
  fileList.sort((a, b) => {
    return a.name.localeCompare(b.name, 'zh-CN', { numeric: true })
  })
}

// 检查是否在演示模式
function checkDemoMode() {
  if (sessionStore.isDemo) {
    uiStore.showError(t('settings.errors.demoUpload'))
    return false
  }
  return true
}

// 按路径上传（桌面端）
async function handlePaths(paths, provider = 'alipay') {
  if (!checkDemoMode()) return

  const allowed = paths.filter(path => {
    const lower = path.toLowerCase()
    return lower.endsWith('.csv') || lower.endsWith('.xlsx')
  })
  if (allowed.length === 0) {
    uiStore.showError(t('settings.errors.invalidFile'))
    return
  }

  try {
    uiStore.setGlobalLoading(true)
    await api.uploadPaths(allowed, provider)
    uiStore.showSuccess(t('settings.errors.uploadSuccess'))
    await loadFiles()
  } catch (error) {
    uiStore.showError(t('settings.errors.uploadFailed') + ': ' + error.message)
  } finally {
    uiStore.setGlobalLoading(false)
  }
}


// 删除文件
async function deleteFile(filename) {
  if (confirm(t('settings.errors.deleteConfirm', { name: filename }))) {
    try {
      await api.deleteFile(filename)

      // 从列表中移除
      alipayFiles.value = alipayFiles.value.filter(f => f.name !== filename)
      wechatFiles.value = wechatFiles.value.filter(f => f.name !== filename)

      uiStore.showSuccess(t('settings.errors.deleted'))
    } catch (error) {
      uiStore.showError(t('settings.errors.deleteFailed') + ': ' + error.message)
    }
  }
}

// 清除所有数据
async function handleClearAllData() {
  if (confirm(t('settings.errors.clearConfirm'))) {
    try {
      await api.clearData()

      // 清空文件列表
      alipayFiles.value = []
      wechatFiles.value = []

      uiStore.showSuccess(t('settings.errors.cleared'))
    } catch (error) {
      uiStore.showError(t('settings.errors.clearFailed') + ': ' + error.message)
    }
  }
}
</script>

<style scoped>
/* 基础样式 */
.settings-page {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 20px 140px 20px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-header-agent {
  /* 标题与说明上下排列，长文案不再挤压标题 */
  flex-direction: column;
  align-items: flex-start;
}

.page-subtitle {
  margin: 8px 0 0;
  font-size: 14px;
  color: var(--secondary-text);
}

.page-title {
  font-size: 28px;
  font-weight: 600;
  color: var(--text-color);
  margin: 0;
}

/* 双栏布局 */
.split-layout {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  margin-bottom: 24px;
}

.provider-column {
  background: #fff;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.03);
  border: 1px solid rgba(0, 0, 0, 0.04);
  transition: all 0.3s ease;
}

.provider-column:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06);
}

.column-header {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color);
  margin: 0 0 20px 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.column-header i {
  font-size: 20px;
}

.alipay-column .column-header i {
  color: #00A0E9;
}

.wechat-column .column-header i {
  color: #07C160;
}

/* 上传指南 */
.upload-guide {
  background: #fbfbfd;
  border-radius: 10px;
  padding: 16px;
  margin-bottom: 24px;
  border: 1px solid rgba(0, 0, 0, 0.03);
}

.upload-guide h3 {
  font-size: 14px;
  font-weight: 600;
  color: #1d1d1f;
  margin-bottom: 12px;
  margin-top: 0;
}

.upload-guide ol {
  margin: 0;
  padding-left: 20px;
}

.upload-guide li {
  font-size: 13px;
  line-height: 1.6;
  color: #424245;
  margin-bottom: 8px;
}

.upload-guide li strong {
  color: #1d1d1f;
  font-weight: 600;
}

/* 文件上传区域 */
.file-upload {
  display: block;
  border: 2px dashed #d2d2d7;
  background: #fafafc;
  border-radius: 12px;
  padding: 32px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s ease;
  height: 180px;
}

.file-upload:hover,
.file-upload.dragover {
  border-color: #007aff;
  background: #f0f8ff;
  transform: scale(1.02);
}

.file-upload input[type="file"] {
  display: none;
}

.upload-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.upload-icon i {
  color: #86868b;
  transition: color 0.2s ease;
}

.file-upload:hover .upload-icon i {
  color: #007aff;
  transform: scale(1.1);
}

.upload-text {
  color: #86868b;
  font-size: 14px;
  font-weight: 500;
}

/* 文件列表 */
.file-list-container {
  margin-top: 24px;
}

.file-list-container h4 {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-color);
  margin: 0 0 12px 0;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-color);
  border-radius: var(--radius-sm);
  transition: background-color 0.2s;
}

.file-item:hover {
  background: var(--hover-bg);
}

.file-item > i {
  color: var(--primary-color);
  font-size: 18px;
}

.file-name {
  flex: 1;
  font-size: 13px;
  color: var(--text-color);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-status {
  font-size: 12px;
  color: var(--secondary-text);
}

.file-status.status-success {
  color: #34C759;
}

.delete-btn {
  background: none;
  border: none;
  padding: 6px;
  cursor: pointer;
  color: var(--secondary-text);
  border-radius: var(--radius-sm);
  transition: all 0.2s;
}

.delete-btn:hover {
  color: var(--danger-color);
  background: rgba(255, 59, 48, 0.1);
}

/* 底部操作栏 */
.bottom-action-bar {
  position: fixed;
  bottom: 0;
  left: var(--sidebar-width);
  right: 0;
  background: rgba(255, 255, 255, 0.85);
  backdrop-filter: saturate(180%) blur(20px);
  border-top: 1px solid var(--border-color);
  padding: 16px 20px;
  transform: translateY(100%);
  transition: transform 0.3s ease;
  z-index: 1000;
}

.bottom-action-bar.visible {
  transform: translateY(0);
}

.action-bar-content {
  max-width: 1400px;
  margin: 0 auto;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.file-summary {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 14px;
  color: var(--text-color);
}

.file-summary i {
  color: var(--primary-color);
}

.start-button {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  text-decoration: none;
  transition: all 0.2s ease;
}

.start-button:hover:not(.disabled) {
  background: #0066E6;
  transform: translateY(-1px);
}

.start-button.disabled {
  opacity: 0.3;
  cursor: not-allowed;
  pointer-events: none;
}

/* 删除区域 */
.delete-section {
  background: var(--card-bg);
  border-radius: var(--radius-md);
  padding: 24px;
  margin-top: 24px;
}

.section-title {
  font-size: var(--font-size-xl);
  font-weight: var(--font-weight-medium);
  color: var(--text-color);
  margin: 0 0 16px 0;
}

.delete-area {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: flex-start;
}

.delete-all-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: var(--danger-color);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.delete-all-btn:hover {
  background: #E6352A;
}

.delete-warning {
  font-size: 13px;
  color: var(--secondary-text);
  margin: 0;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .split-layout {
    grid-template-columns: 1fr;
  }

  .bottom-action-bar {
    left: 0;
    padding: 12px 16px;
  }

  .action-bar-content {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .start-button {
    justify-content: center;
  }

  .settings-page {
    padding: 0 16px 140px 16px;
  }

  .page-header {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }
}

/* MCP 服务 */
.mcp-section {
  margin-top: 32px;
}

.mcp-card {
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  padding: 20px;
}

.mcp-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.mcp-label {
  width: 64px;
  color: #6b7280;
  font-size: 13px;
}

.mcp-status {
  color: #8e8e93;
  font-weight: 600;
}

.mcp-status.running {
  color: #34c759;
}

.mcp-endpoint,
.mcp-token {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  background: #f3f4f6;
  padding: 4px 8px;
  border-radius: 6px;
  word-break: break-all;
}

.mcp-error {
  color: #ff3b30;
  font-size: 13px;
  margin: 8px 0;
}

.mcp-config {
  margin: -4px 0 16px;
}

.mcp-config-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.mcp-config-head .mcp-label {
  width: auto;
  white-space: nowrap;
}

.mcp-config-code {
  margin: 0 0 8px;
  padding: 12px 14px;
  background: #F5F5F7;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-family: var(--font-family-mono);
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-color);
  overflow-x: auto;
  white-space: pre;
}

.mcp-actions {
  display: flex;
  gap: 12px;
  margin-top: 16px;
}

.mcp-btn {
  padding: 8px 16px;
  border-radius: 8px;
  border: 1px solid #d1d5db;
  background: #fff;
  cursor: pointer;
  font-size: 13px;
}

.mcp-btn.primary {
  background: #007aff;
  border-color: #007aff;
  color: #fff;
}

.mcp-hint {
  margin-top: 12px;
  color: #8e8e93;
  font-size: 12px;
}

/* 小遥Claw 接入引导 */
.claw-guide {
  margin-top: 16px;
  padding: 20px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
}

.claw-guide-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}

.claw-guide-head h3 {
  margin: 0 0 6px;
  color: #111827;
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
}

.claw-guide-head p {
  margin: 0;
  color: #6b7280;
  font-size: 13px;
  line-height: 1.6;
}

.claw-badge {
  flex: 0 0 auto;
  padding: 4px 10px;
  color: #007aff;
  font-size: 12px;
  font-weight: 600;
  background: #eef5ff;
  border-radius: 999px;
}

.claw-steps {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.claw-step {
  display: flex;
  gap: 12px;
  padding: 14px;
  background: #f7f8fa;
  border-radius: 10px;
}

.claw-step-num {
  display: inline-flex;
  flex: 0 0 24px;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  color: #fff;
  font-size: 12px;
  font-weight: 700;
  background: #007aff;
  border-radius: 50%;
}

.claw-step-body strong {
  display: block;
  margin-bottom: 4px;
  color: #111827;
  font-size: 13px;
  font-weight: 600;
}

.claw-step-body p {
  margin: 0;
  color: #6b7280;
  font-size: 12px;
  line-height: 1.6;
}

.claw-step-body code {
  padding: 1px 4px;
  font-family: var(--font-family-mono);
  font-size: 11px;
  background: #e5e7eb;
  border-radius: 4px;
}

.claw-shots-title {
  margin: 20px 0 8px;
  color: #6b7280;
  font-size: 12px;
  font-weight: 600;
}

.claw-shots {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.claw-shot {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 0;
  background: none;
  border: 0;
  cursor: zoom-in;
}

.claw-shot img {
  width: 100%;
  aspect-ratio: 16 / 10;
  object-fit: cover;
  background: #f5f5f7;
  border: 1px solid #e5e7eb;
  border-radius: 10px;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.claw-shot:hover img {
  transform: translateY(-1px);
  box-shadow: 0 6px 18px rgba(15, 23, 42, 0.12);
}

.claw-shot span {
  color: #6b7280;
  font-size: 12px;
  text-align: center;
}

.claw-preview {
  position: fixed;
  inset: 0;
  z-index: 5000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
  background: rgba(15, 23, 42, 0.78);
}

.claw-preview img {
  max-width: 92vw;
  max-height: 84vh;
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.35);
}

.claw-preview-close {
  position: absolute;
  top: 20px;
  right: 24px;
  width: 36px;
  height: 36px;
  color: #111827;
  font-size: 22px;
  line-height: 1;
  background: rgba(255, 255, 255, 0.92);
  border: 0;
  border-radius: 50%;
  cursor: pointer;
}

.claw-guide-foot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #f0f1f3;
}

.claw-guide-foot span {
  color: #8e8e93;
  font-size: 12px;
}

@media (max-width: 900px) {
  .claw-steps,
  .claw-shots {
    grid-template-columns: 1fr;
  }
}

/* 桌面设置 */
.language-select {
  height: 34px;
  padding: 0 28px 0 10px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: #fff;
  color: #111827;
  font-size: 13px;
  cursor: pointer;
}

.desktop-section {
  margin-top: 32px;
}

.desktop-card {
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  padding: 20px;
}

.desktop-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.storage-value {
  color: #111827;
  font-size: 13px;
}
</style>
