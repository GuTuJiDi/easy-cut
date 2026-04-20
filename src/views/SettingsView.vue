<template>
  <div class="view-container">
    <header class="view-header">
      <h1 class="view-title">全局偏好设置</h1>
      <p class="view-subtitle">定制您的易剪工作区、性能引擎调度与授权状态。</p>
    </header>

    <div class="settings-layout">
      <!-- ========================================= -->
      <!-- 🌟 1. 商业化核心：PRO 旗舰版授权面板 -->
      <!-- ========================================= -->
      <div class="card auth-card" :class="{'is-pro': authStore.isPro}">
        <div class="card-header">
          <span class="icon">👑</span>
          <h2>产品授权状态</h2>
        </div>
        <div class="card-body padded auth-body">

          <!-- 已激活 PRO 状态 (带有专属尊贵紫色光晕) -->
          <transition name="fade" mode="out-in">
            <div v-if="authStore.isPro" class="pro-status">
              <div class="status-info">
                <div class="pro-title-group">
                  <h3>尊贵的 PRO 旗舰版用户</h3>
                  <span class="pro-badge">Active</span>
                </div>
                <p>您已解锁“无限数量分割”、“极限并发调度”等全部高阶生产力功能。</p>
                <div class="license-display">
                  <span class="label">设备凭证:</span>
                  <code>{{ maskLicense(authStore.licenseKey) }}</code>
                </div>
              </div>
              <div class="status-action">
                <button class="secondary-btn danger-text" @click="handleDeactivate" title="解绑当前设备的授权">注销设备</button>
              </div>
            </div>

            <!-- 未激活 免费状态 -->
            <div v-else class="free-status">
              <div class="status-info">
                <h3>免费基础版</h3>
                <p>当前享有基础音视频无损处理权益。输入由发卡平台获取的数字签名凭证，立即解锁生产力上限。</p>
                <div class="activation-input-group">
                  <div class="input-wrapper">
                    <span class="key-icon">🔑</span>
                    <input
                        v-model="tempLicense"
                        type="text"
                        placeholder="请输入您的易剪 PRO 激活凭证 (格式如: eyJt...)"
                        class="license-input"
                        @keyup.enter="handleActivate"
                    />
                  </div>
                  <button class="primary-btn activate-btn" @click="handleActivate" :disabled="isVerifying || !tempLicense.trim()">
                    <svg v-if="isVerifying" class="spinner-svg" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <!-- 🌟 修改文案，使其更加紧凑 -->
                    {{ isVerifying ? '验证中...' : '立即激活' }}
                  </button>
                </div>
              </div>
            </div>
          </transition>
        </div>
      </div>

      <!-- ========================================= -->
      <!-- 🌟 2. 性能怪兽：底层引擎并发调度拉杆 -->
      <!-- ========================================= -->
      <div class="card">
        <div class="card-header">
          <span class="icon">🚀</span>
          <h2>底层引擎性能调度</h2>
        </div>
        <div class="card-body padded">
          <div class="setting-item">
            <div class="item-content">
              <label class="item-title">Tokio 异步并发线程数</label>
              <p class="item-desc">调高数值可压榨多核 CPU 和 SSD 极限吞吐量，极速缩短批量切割时间。配置较低的电脑请慎重拉满。</p>
            </div>
            <div class="item-action slider-action">
              <div class="slider-wrapper">
                <span class="slider-min">1</span>
                <input
                    type="range"
                    min="1"
                    max="16"
                    step="1"
                    v-model.number="settingsStore.maxConcurrentTasks"
                    @change="syncSettings"
                    class="range-slider"
                >
                <span class="slider-max">16</span>
              </div>
              <div class="range-val-badge">
                <strong>{{ settingsStore.maxConcurrentTasks }}</strong> 线程
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- ========================================= -->
      <!-- 3. 数据与存储 (原有功能强化布局) -->
      <!-- ========================================= -->
      <div class="card">
        <div class="card-header">
          <span class="icon">🗄️</span>
          <h2>数据与存储</h2>
        </div>
        <div class="card-body padded">
          <div class="setting-item">
            <div class="item-content">
              <label class="item-title">易剪工作区 (Workspace)</label>
              <p class="item-desc">所有配置文件、标记数据、自动日志均统一安全存放于此。</p>
            </div>
            <div class="item-action workspace-action">
              <input type="text" :value="workspaceStore.currentWorkspace" readonly class="readonly-input path-input" />
              <button class="secondary-btn" @click="openWorkspace" title="在资源管理器中打开">
                📂 打开目录
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- ========================================= -->
      <!-- 4. 工作流与安全首选项 -->
      <!-- ========================================= -->
      <div class="card">
        <div class="card-header">
          <span class="icon">🛡️</span>
          <h2>安全与首选项</h2>
        </div>
        <div class="card-body padded">
          <div class="setting-item">
            <div class="item-content">
              <label class="item-title">智能打轴自动保存</label>
              <p class="item-desc">开启后，在打轴面板的任何操作将静默同步到硬盘，防止突发断电导致心血丢失。</p>
            </div>
            <div class="item-action toggle-action">
              <label class="toggle-switch">
                <input type="checkbox" v-model="settingsStore.autoSave" @change="syncSettings" />
                <span class="slider round"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="item-content">
              <label class="item-title">废弃标记保留周期</label>
              <p class="item-desc">被清空的 JSON 标记文件将被移入回收站。软件将在每次启动时自动静默清理废件。</p>
            </div>
            <div class="item-action select-action">
              <select class="custom-select" v-model.number="settingsStore.trashDays" @change="syncSettings">
                <option :value="7">保留 7 天</option>
                <option :value="15">保留 15 天</option>
                <option :value="30">保留 30 天 (推荐)</option>
                <option :value="90">保留 90 天</option>
                <option :value="0">永不自动清理</option>
              </select>
            </div>
          </div>
        </div>
      </div>

    </div>

    <!-- 优雅的全局 Toast 提示 -->
    <transition name="toast">
      <div v-if="toastMsg" class="toast-message" :class="toastType">{{ toastMsg }}</div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../stores/settings';
import { useWorkspaceStore } from '../stores/workspace';
import { useAuthStore } from '../stores/auth';
import { confirm as tauriConfirm } from '@tauri-apps/plugin-dialog';

const settingsStore = useSettingsStore();
const workspaceStore = useWorkspaceStore();
const authStore = useAuthStore();

// 提示状态
const toastMsg = ref('');
const toastType = ref<'success' | 'error' | 'info'>('success');
let toastTimer: number | null = null;

// 激活相关状态
const tempLicense = ref('');
const isVerifying = ref(false);

onMounted(() => {
  // 确保工作区路径和设置被加载
  if (!workspaceStore.currentWorkspace) {
    workspaceStore.initWorkspace();
  }
  settingsStore.fetchSettings();
});

// =========================================
// 🌟 商业化逻辑：激活与注销
// =========================================
async function handleActivate() {
  const key = tempLicense.value.trim();
  if (!key) return;

  isVerifying.value = true;
  try {
    // 呼叫 Store 里封装好的鉴权逻辑（内部已打通 Rust 的 verify_license_cmd）
    await authStore.verifyAndActivate(key);
    showToast('🎉 签名验证通过！欢迎使用 PRO 旗舰版！', 'success');
    tempLicense.value = '';
  } catch (error) {
    showToast(`🚨 激活失败: ${error}`, 'error');
  } finally {
    isVerifying.value = false;
  }
}

async function handleDeactivate() {
  // 🌟 修复：使用 await 等待 Tauri 弹窗的真实布尔值结果
  const confirmed = await tauriConfirm("注销后将失去 PRO 专属功能权限，确定要注销此设备的授权吗？", {
    title: '易剪 EasyCut',
    kind: 'warning'
  });

  if (confirmed) {
    authStore.deactivate();
    showToast('设备已成功解绑', 'info');
  }
}
// 🌟 掩码显示密钥 (前端脱敏，保护用户隐私)
function maskLicense(key: string) {
  if (!key || key.length < 20) return key;
  return key.substring(0, 10) + '••••••••••••••••' + key.substring(key.length - 10);
}

// =========================================
// 原有设置逻辑优化
// =========================================
async function syncSettings() {
  try {
    await settingsStore.saveSettings(settingsStore.autoSave, settingsStore.trashDays, settingsStore.maxConcurrentTasks);
    showToast('✅ 偏好设置已全局更新', 'success');
  } catch (error) {
    showToast(`❌ 更新失败: ${error}`, 'error');
  }
}

async function openWorkspace() {
  try {
    if (!workspaceStore.currentWorkspace) { showToast("❌ 工作区路径未就绪", "error"); return; }
    await invoke('open_folder', { path: workspaceStore.currentWorkspace });
  } catch (error) { showToast(`❌ 无法打开目录`, "error"); }
}

function showToast(msg: string, type: 'success' | 'error' | 'info' = 'success') {
  toastMsg.value = msg;
  toastType.value = type;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => { toastMsg.value = ''; }, 3000);
}
</script>

<style scoped>
/* ========================================= */
/* 🧱 基础布局 */
/* ========================================= */
.settings-layout { display: flex; flex-direction: column; gap: 1.5rem; flex: 1; }
.card-header h2 { margin: 0; font-size: 1.1rem; color: #1f2937; font-weight: 700; }
.card-body.padded { padding: 1.5rem; }
.setting-item { display: flex; justify-content: space-between; align-items: flex-start; padding: 1rem 0; border-bottom: 1px dashed #e5e7eb; gap: 1.5rem; flex-wrap: wrap; }
.setting-item:last-child { border-bottom: none; padding-bottom: 0; }
.item-content { flex: 1; min-width: 0; }
.item-title { display: block; font-size: 1.05rem; font-weight: 600; color: #374151; margin-bottom: 0.35rem; }
.item-desc { margin: 0; font-size: 0.85rem; color: #6b7280; line-height: 1.6; word-wrap: break-word; white-space: normal; }
.item-action { flex-shrink: 0; width: auto; display: flex; align-items: center; }

/* 表单组件 */
.workspace-action { gap: 10px; width: 100%; max-width: 450px; flex-wrap: wrap; }
.readonly-input { flex: 1; min-width: 100px; padding: 0.6rem 1rem; border: 1px solid #d1d5db; border-radius: 8px; background: #f8fafc; color: #4b5563; font-family: 'Consolas', monospace; font-size: 0.85rem; outline: none; text-overflow: ellipsis; }
.secondary-btn { padding: 0.6rem 1.2rem; background: white; border: 1px solid #d1d5db; border-radius: 8px; color: #374151; font-weight: 600; cursor: pointer; white-space: nowrap; transition: all 0.2s; }
.secondary-btn:hover { background: #f1f5f9; border-color: #cbd5e1;}
.primary-btn { padding: 0.6rem 1.5rem; background: linear-gradient(135deg, #2563eb, #1d4ed8); color: white; border: none; border-radius: 8px; font-weight: 600; cursor: pointer; white-space: nowrap; transition: all 0.2s; box-shadow: 0 4px 6px -1px rgba(37, 99, 235, 0.2);}
.primary-btn:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 6px 8px -1px rgba(37, 99, 235, 0.3); }
.primary-btn:disabled { background: #94a3b8; cursor: not-allowed; box-shadow: none; opacity: 0.8;}
.custom-select { padding: 0.6rem 1rem; border: 1px solid #cbd5e1; border-radius: 8px; background: white; color: #1f2937; font-size: 0.95rem; font-weight: 500; cursor: pointer; outline: none; min-width: 160px; transition: border-color 0.2s;}
.custom-select:focus { border-color: #3b82f6; box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1); }

/* 开关组件 (iOS风格) */
.toggle-switch { position: relative; display: inline-block; width: 50px; height: 28px; }
.toggle-switch input { opacity: 0; width: 0; height: 0; }
.slider { position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: #cbd5e1; transition: .3s; border-radius: 34px;}
.slider:before { position: absolute; content: ""; height: 20px; width: 20px; left: 4px; bottom: 4px; background-color: white; transition: .3s; box-shadow: 0 2px 4px rgba(0,0,0,0.2); border-radius: 50%;}
input:checked + .slider { background-color: #10b981; }
input:checked + .slider:before { transform: translateX(22px); }

/* ========================================= */
/* 🌟 PRO 商业化授权卡片 (极客尊贵美学) */
/* ========================================= */
.auth-card { transition: all 0.4s ease; border: 2px solid transparent; }
.auth-card.is-pro { border-color: #c084fc; box-shadow: 0 10px 25px -5px rgba(168, 85, 247, 0.15); }
.auth-card.is-pro .card-header { background: linear-gradient(to right, #faf5ff, white); }
.auth-body { padding-top: 0.5rem; }

/* PRO 激活态 */
.pro-status { display: flex; justify-content: space-between; align-items: center; background: linear-gradient(135deg, #fdf4ff 0%, #f3e8ff 100%); padding: 1.5rem; border-radius: 12px; border: 1px solid #e9d5ff; gap: 2rem; flex-wrap: wrap;}
.pro-title-group { display: flex; align-items: center; gap: 10px; margin-bottom: 0.5rem; }
.pro-status h3 { color: #6b21a8; margin: 0; font-size: 1.2rem; font-weight: 800;}
.pro-badge { background: #9333ea; color: white; font-size: 0.7rem; padding: 2px 8px; border-radius: 12px; font-weight: bold; text-transform: uppercase; letter-spacing: 0.5px;}
.pro-status p { color: #7e22ce; margin: 0 0 1rem 0; font-size: 0.95rem; font-weight: 500;}
.license-display { display: inline-flex; align-items: center; background: rgba(255,255,255,0.6); padding: 6px 12px; border-radius: 8px; border: 1px dashed #d8b4fe; margin-top: 5px;}
.license-display .label { font-size: 0.8rem; color: #9333ea; margin-right: 8px; font-weight: 600;}
.license-display code { color: #a855f7; font-size: 0.85rem; letter-spacing: 2px; font-family: 'Consolas', monospace; font-weight: bold;}
.danger-text { color: #ef4444; border-color: #fca5a5; }
.danger-text:hover { background: #fef2f2; color: #dc2626; border-color: #ef4444; }

/* Free 未激活态 */
.free-status { display: flex; flex-direction: column; gap: 1rem; background: #f8fafc; padding: 1.5rem; border-radius: 12px; border: 1px dashed #cbd5e1;}
.free-status h3 { margin: 0 0 0.5rem 0; color: #334155; font-size: 1.15rem;}
.free-status p { margin: 0; color: #64748b; font-size: 0.95rem; }

/* 🌟 核心修复：输入框与按钮的精美布局 */
.activation-input-group {
  display: flex;
  align-items: stretch; /* 保证输入框和按钮等高 */
  gap: 12px;
  margin-top: 1rem;
  max-width: 650px;
}
.input-wrapper {
  position: relative;
  flex: 1; /* 自动撑开占据剩余空间 */
}
.key-icon {
  position: absolute;
  left: 14px;
  top: 50%;
  transform: translateY(-50%);
  opacity: 0.6;
  font-size: 1.1rem;
}
.license-input {
  width: 100%;
  height: 100%; /* 继承父级高度 */
  padding: 0.85rem 1rem 0.85rem 2.8rem; /* 给左侧图标留出绝美空间 */
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font-family: 'Consolas', monospace;
  font-size: 0.95rem;
  outline: none;
  transition: all 0.2s;
  background: white;
  box-sizing: border-box;
  box-shadow: inset 0 1px 3px rgba(0,0,0,0.03);
}
.license-input:focus {
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.15), inset 0 1px 3px rgba(0,0,0,0.03);
}
/* 🌟 核心修复：焊死按钮的宽度，绝对禁止它在动画时扩张 */
/* 🌟 核心修复：物理锁死按钮宽度，无论字数怎么变都绝对不许挤压输入框 */
.activate-btn {
  width: 150px !important;      /* 强制物理宽度 */
  min-width: 150px !important;  /* 拒绝被压缩 */
  max-width: 150px !important;  /* 拒绝被撑开 */
  flex-shrink: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
  box-sizing: border-box;
}
.spinner-svg {
  width: 1.2rem;
  height: 1.2rem;
  animation: spin 1s linear infinite;
  color: white;
}
.opacity-25 { opacity: 0.25; }
.opacity-75 { opacity: 0.75; }
@keyframes spin { 100% { transform: rotate(360deg); } }

/* ========================================= */
/* 🚀 性能调度器 (油门拉杆) */
/* ========================================= */
.slider-action { display: flex; align-items: center; gap: 20px; flex-wrap: wrap;}
.slider-wrapper { display: flex; align-items: center; gap: 10px; background: #f8fafc; padding: 8px 15px; border-radius: 30px; border: 1px solid #e2e8f0;}
.slider-min, .slider-max { font-size: 0.8rem; color: #94a3b8; font-weight: bold; font-family: monospace;}
.range-slider { -webkit-appearance: none; width: 160px; height: 6px; background: #cbd5e1; border-radius: 3px; outline: none; transition: background 0.2s; cursor: pointer;}
.range-slider::-webkit-slider-thumb { -webkit-appearance: none; appearance: none; width: 18px; height: 18px; border-radius: 50%; background: #2563eb; cursor: pointer; box-shadow: 0 2px 4px rgba(37,99,235,0.3); border: 2px solid white; transition: transform 0.1s;}
.range-slider::-webkit-slider-thumb:hover { transform: scale(1.15); }
.range-slider:active::-webkit-slider-thumb { transform: scale(0.95); }
.range-val-badge { background: #eff6ff; color: #1d4ed8; padding: 6px 12px; border-radius: 8px; font-size: 0.9rem; border: 1px solid #bfdbfe;}
.range-val-badge strong { font-size: 1.2rem; font-family: 'Consolas', monospace; }

/* ========================================= */
/* Toast 提示动画 */
/* ========================================= */
.toast-message { position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%); color: white; padding: 0.8rem 1.5rem; border-radius: 30px; font-size: 0.95rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.15); z-index: 9999; font-weight: 500;}
.toast-message.success { background: #10b981; border: 1px solid #059669; }
.toast-message.error { background: #ef4444; border: 1px solid #dc2626;}
.toast-message.info { background: #3b82f6; border: 1px solid #2563eb;}
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>