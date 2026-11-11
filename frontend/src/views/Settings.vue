<template>
  <div class="settings-page">
    <!-- 页面标题：设置页按「账单文件 / 数据管理 / 存储与审计 / 桌面」分组；/agent 复用本页只渲染 MCP 区块 -->
    <div class="page-header" :class="{ 'page-header-agent': agentOnly }">
      <h1 class="page-title">{{ agentOnly ? 'AI Agent 接入' : '设置' }}</h1>
      <p v-if="agentOnly" class="page-subtitle">
        通过 MCP 服务让 小遥Claw、Claude Code、Codex 等 Agent 读取本机账单数据；默认关闭，仅监听 127.0.0.1。
      </p>
    </div>

    <!-- 账单文件：导入与已上传列表 -->
    <h2 v-if="!agentOnly" class="section-title">账单文件</h2>
    <div v-if="!agentOnly" class="split-layout">
      <!-- 左侧：支付宝专区 -->
      <div class="provider-column alipay-column">
        <h2 class="column-header">
          <i class="fab fa-alipay"></i> 支付宝账单
        </h2>

        <!-- 支付宝指南 -->
        <div class="upload-guide">
          <h3>如何获取支付宝账单？</h3>
          <ol>
            <li>打开支付宝 App -> 我的 -> 账单</li>
            <li>右上角 ... -> 开具交易流水证明 -> 用于个人对账 -> 申请</li>
            <li>自定义时间范围（最长为一年） -> 填写邮箱 -> 下载账单</li>
            <li>申请记录中找到解压密码 -> 解压下载的文件，获取 CSV 文件</li>
            <li>按年份重命名为【alipay_record_2026.csv】格式</li>
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
            点击选择或拖拽<br />支付宝 CSV 文件
            <br /><span style="font-size: 12px; opacity: 0.7;">最大 16MB</span>
          </div>
        </div>

        <!-- 支付宝文件列表 -->
        <div class="file-list-container">
          <h4>已上传文件</h4>
          <div class="file-list">
            <div v-for="file in alipayFiles" :key="file.name" class="file-item">
              <FileSpreadsheet :size="'1em'" />
              <span class="file-name">{{ file.name }}</span>
              <div class="file-actions">
                <span class="file-status status-success">已上传</span>
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
          <i class="fab fa-weixin"></i> 微信账单
        </h2>

        <!-- 微信指南 -->
        <div class="upload-guide">
          <h3>如何获取微信账单？</h3>
          <ol>
            <li>打开微信 App -> 我 -> 服务 -> 钱包 -> 账单</li>
            <li>点击右上角 [...] -> 账单下载 -> 用于个人对账</li>
            <li>选择接收方式（微信/邮箱）、账单时间 -> 下一步</li>
            <li><strong>方式一（推荐）：</strong>刷脸验证 -> 微信消息直接下载 XLSX 文件</li>
            <li><strong>方式二：</strong>输入邮箱 -> 刷脸 -> 邮箱接收 (密码在微信消息中)</li>
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
            点击或拖拽<br />微信 XLSX/CSV 文件
            <br /><span style="font-size: 12px; opacity: 0.7;">最大 16MB</span>
          </div>
        </div>

        <!-- 微信文件列表 -->
        <div class="file-list-container">
          <h4>已上传文件</h4>
          <div class="file-list">
            <div v-for="file in wechatFiles" :key="file.name" class="file-item">
              <FileSpreadsheet :size="'1em'" />
              <span class="file-name">{{ file.name }}</span>
              <div class="file-actions">
                <span class="file-status status-success">已上传</span>
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
          <span>{{ totalFileCount > 0 ? `已就绪 ${totalFileCount} 个账单文件` : '请上传账单文件' }}</span>
        </div>
        <div class="action-buttons">
          <router-link to="/yearly" class="start-button" :class="{ disabled: totalFileCount === 0 }">
            <ChartLine :size="'1em'" />
            开始分析
          </router-link>
        </div>
      </div>
    </div>

    <!-- 删除区域 -->
    <div v-if="!agentOnly" class="delete-section">
      <h2 class="section-title">数据管理</h2>
      <div class="delete-area">
        <button class="delete-all-btn" @click="handleClearAllData">
          <Trash2 :size="'1em'" />
          删除所有账单数据
        </button>
        <p class="delete-warning">删除后将清空所有已上传的账单文件和分析数据，此操作不可恢复。</p>
      </div>
    </div>

    <!-- 存储与审计：数据目录与可恢复数据 -->
    <div v-if="!agentOnly" class="desktop-section">
      <h2 class="section-title">存储与审计</h2>
      <div class="desktop-card">
        <div class="desktop-row">
          <span class="mcp-label">数据目录</span>
          <code class="mcp-endpoint">{{ desktop.data_dir }}</code>
          <button class="mcp-btn" @click="openDataDir">打开</button>
        </div>
        <div class="desktop-row">
          <span class="mcp-label">回收站</span>
          <span class="storage-value">{{ storage.trash_count }} 个条目</span>
          <button class="mcp-btn" @click="openStorageDir('trash')">打开</button>
        </div>
        <div class="desktop-row">
          <span class="mcp-label">快照</span>
          <span class="storage-value">{{ storage.snapshot_count }} 个</span>
          <button class="mcp-btn" @click="openStorageDir('snapshots')">打开</button>
        </div>
        <div class="desktop-row">
          <span class="mcp-label">审计日志</span>
          <span class="storage-value">{{ auditSizeText }}</span>
        </div>
      </div>
    </div>

    <!-- 桌面行为 -->
    <div v-if="!agentOnly" class="desktop-section">
      <h2 class="section-title">桌面</h2>
      <div class="desktop-card">
        <div class="desktop-row">
          <span class="mcp-label">关闭行为</span>
          <button class="mcp-btn" @click="toggleCloseToTray">
            {{ desktop.close_to_tray ? '关闭窗口时最小化到托盘' : '关闭窗口时直接退出' }}
          </button>
        </div>
      </div>
    </div>

    <!-- MCP 服务 -->
    <div v-if="agentOnly" class="mcp-section">
      <div class="mcp-card">
        <div class="mcp-row">
          <span class="mcp-label">状态</span>
          <span class="mcp-status" :class="{ running: mcp.running }">
            {{ mcp.running ? '运行中' : '已停止' }}
          </span>
        </div>
        <div class="mcp-row">
          <span class="mcp-label">端点</span>
          <code class="mcp-endpoint">{{ mcp.endpoint }}</code>
        </div>
        <div v-if="mcp.running && mcp.token" class="mcp-row">
          <span class="mcp-label">Token</span>
          <code class="mcp-token">{{ mcp.token }}</code>
          <button class="mcp-btn" @click="copyToken">复制</button>
        </div>
        <p v-if="mcp.last_error" class="mcp-error">{{ mcp.last_error }}</p>
        <div class="mcp-actions">
          <button class="mcp-btn primary" @click="toggleMcp">
            {{ mcp.enabled ? '关闭 MCP 服务' : '开启 MCP 服务' }}
          </button>
          <button class="mcp-btn" @click="rotateToken">重新生成 Token</button>
        </div>
        <p class="mcp-hint">
          仅监听 127.0.0.1，默认关闭；开启后请在客户端配置 Bearer Token。
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ChartLine, FileSpreadsheet, FileText, Trash2 } from 'lucide-vue-next'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { useSessionStore } from '@/stores/session'
import { useUiStore } from '@/stores/ui'
import api from '@/api/client'

const props = defineProps({
  // /agent 路由复用本组件，只渲染 MCP 区块
  agentOnly: { type: Boolean, default: false }
})

const sessionStore = useSessionStore()
const uiStore = useUiStore()

// 文件列表
const alipayFiles = ref([])
const wechatFiles = ref([])

// 拖拽状态
const alipayDragging = ref(false)
const wechatDragging = ref(false)

// 桌面设置
const desktop = ref({
  close_to_tray: true,
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

async function toggleCloseToTray() {
  try {
    const result = await api.setCloseToTray(!desktop.value.close_to_tray)
    desktop.value = { ...desktop.value, ...result }
    uiStore.showSuccess(result.close_to_tray ? '关闭窗口将最小化到托盘' : '关闭窗口将直接退出')
  } catch (error) {
    uiStore.showError('设置失败: ' + error.message)
  }
}

async function openStorageDir(kind) {
  try {
    await api.openStorageDir(kind)
  } catch (error) {
    uiStore.showError('打开目录失败: ' + error.message)
  }
}

async function openDataDir() {
  try {
    await api.openDataDir()
  } catch (error) {
    uiStore.showError('打开数据目录失败: ' + error.message)
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
      uiStore.showSuccess('MCP 服务已开启')
    } else if (status.last_error) {
      uiStore.showError(status.last_error)
    } else {
      uiStore.showSuccess('MCP 服务已关闭')
    }
  } catch (error) {
    uiStore.showError('MCP 操作失败: ' + error.message)
  }
}

async function rotateToken() {
  try {
    const config = await api.mcpRotateToken()
    mcp.value = { ...mcp.value, ...config }
    uiStore.showSuccess('Token 已重新生成')
  } catch (error) {
    uiStore.showError('Token 轮换失败: ' + error.message)
  }
}

async function copyToken() {
  try {
    await navigator.clipboard.writeText(mcp.value.token)
    uiStore.showSuccess('Token 已复制')
  } catch (error) {
    uiStore.showError('复制失败: ' + error.message)
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
    ? [{ name: '微信账单', extensions: ['xlsx', 'csv'] }]
    : [{ name: '支付宝账单', extensions: ['csv'] }]

  try {
    const selected = await open({ multiple: true, filters })
    if (!selected) return
    await handlePaths(Array.isArray(selected) ? selected : [selected], provider)
  } catch (error) {
    uiStore.showError('打开文件选择器失败: ' + error.message)
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
    uiStore.showError('演示模式下无法上传文件，请先退出演示模式')
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
    uiStore.showError('请上传 .csv 或 .xlsx 格式的账单文件')
    return
  }

  try {
    uiStore.setGlobalLoading(true)
    await api.uploadPaths(allowed, provider)
    uiStore.showSuccess('文件上传成功')
    await loadFiles()
  } catch (error) {
    uiStore.showError('上传失败: ' + error.message)
  } finally {
    uiStore.setGlobalLoading(false)
  }
}


// 删除文件
async function deleteFile(filename) {
  if (confirm(`确定要删除文件 "${filename}" 吗？`)) {
    try {
      await api.deleteFile(filename)

      // 从列表中移除
      alipayFiles.value = alipayFiles.value.filter(f => f.name !== filename)
      wechatFiles.value = wechatFiles.value.filter(f => f.name !== filename)

      uiStore.showSuccess('文件已删除')
    } catch (error) {
      uiStore.showError('删除失败: ' + error.message)
    }
  }
}

// 清除所有数据
async function handleClearAllData() {
  if (confirm('确定要删除所有账单数据吗？此操作不可恢复！')) {
    try {
      await api.clearData()

      // 清空文件列表
      alipayFiles.value = []
      wechatFiles.value = []

      uiStore.showSuccess('所有数据已清除')
    } catch (error) {
      uiStore.showError('清除失败: ' + error.message)
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

/* 桌面设置 */
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
