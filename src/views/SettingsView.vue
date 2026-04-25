<template>
  <div class="view-scroll-container">
    <div class="view-container">

      <header class="view-header-card">
        <div class="header-left">
          <div class="header-icon-box">
            <svg xmlns="http://www.w3.org/2000/svg" width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="#2563eb" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"></path>
              <circle cx="12" cy="12" r="3"></circle>
            </svg>
          </div>
          <div class="header-text">
            <h1 class="view-title">全局偏好设置</h1>
            <p class="view-subtitle">定制您的易剪工作区、性能引擎调度与授权状态。</p>
          </div>
        </div>
        <div class="header-right">
          <div class="system-status-badge">
            <span class="pulse-dot"></span>
            <span class="status-text">引擎核心已就绪</span>
          </div>
        </div>
      </header>

      <div class="settings-layout">

        <div class="card auth-card" :class="{'is-pro': authStore.isPro}">
          <div class="card-header">
            <span class="icon">👑</span>
            <h2>产品授权状态</h2>
          </div>
          <div class="card-body padded auth-body">
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
                    <code>已通过系统级安全凭证校验</code>
                  </div>
                </div>
                <div class="status-action">
                  <button
                      type="button"
                      class="secondary-btn danger-text"
                      @click="deactivateLicense"
                      :disabled="isSaving"
                      title="解绑当前设备的授权"
                  >
                    {{ isSaving ? '正在撤销...' : '注销设备' }}
                  </button>
                </div>
              </div>

              <div v-else class="free-status">
                <div class="status-info">
                  <h3>免费基础版</h3>
                  <p>当前享有基础音视频无损处理权益。输入由发卡平台获取的数字签名凭证，立即解锁生产力上限。</p>

                  <div class="activation-input-group">
                    <div class="input-wrapper">
                      <span class="key-icon">🔑</span>
                      <input
                          v-model="licenseKey"
                          type="text"
                          placeholder="请输入 64 位超级激活码..."
                          class="license-input"
                          :disabled="isSaving"
                          @keyup.enter="activateLicense"
                      />
                    </div>
                    <button class="primary-btn activate-btn" @click="activateLicense" :disabled="isSaving || !licenseKey.trim()">
                      <svg v-if="isSaving" class="spinner-svg" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      {{ isSaving ? '验证中...' : '立即激活' }}
                    </button>
                  </div>
                </div>
              </div>
            </transition>
          </div>
        </div>

        <div class="card">
          <div class="card-header">
            <span class="icon">🚀</span>
            <h2>底层引擎性能调度</h2>
          </div>
          <div class="card-body padded">
            <div class="setting-item">
              <div class="item-content">
                <label class="item-title">Tokio 异步并发线程数</label>
                <p class="item-desc">
                  <span v-if="!authStore.isPro" style="color: #ef4444; font-weight: bold; margin-right: 5px;">🔒 免费版锁定 1 线程。</span>
                  调高数值可压榨多核 CPU 和 SSD 极限吞吐量，极速缩短批量切割时间。配置较低的电脑请慎重拉满。
                </p>
              </div>
              <div class="item-action slider-action" :style="{ opacity: authStore.isPro ? 1 : 0.6 }">
                <div class="slider-wrapper">
                  <span class="slider-min">1</span>
                  <input
                      type="range"
                      min="1"
                      :max="realCpuCount"
                      step="1"
                      v-model.number="settingsStore.maxConcurrentTasks"
                      class="range-slider"
                      :disabled="!authStore.isPro"
                  >
                  <span class="slider-max">{{ realCpuCount }}</span>
                </div>
                <div class="range-val-badge">
                  <strong>{{ authStore.isPro ? settingsStore.maxConcurrentTasks : 1 }}</strong> 线程
                </div>
              </div>
            </div>
          </div>
        </div>

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
                <input type="text" v-model="workspaceStore.currentWorkspace" class="readonly-input path-input" placeholder="正在检测工作区..." />
                <button @click="isModalOpen = true" class="main-button">
                  点击机器码
                </button>

                <LongTestModel
                    :visible="isModalOpen"
                    title="机器码"
                    @close="isModalOpen = false"
                />
                <button class="compact-btn" @click="selectWorkspace" title="浏览并选择新的目录">
                  🔍 浏览...
                </button>
                <button class="compact-btn" @click="openWorkspace" title="在资源管理器中打开">
                  📂 打开
                </button>
              </div>
            </div>
          </div>
        </div>

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
                  <input type="checkbox" v-model="settingsStore.autoSave" />
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
                <select class="custom-select" v-model.number="settingsStore.trashDays">
                  <option :value="7">保留 7 天</option>
                  <option :value="15">保留 15 天</option>
                  <option :value="30">保留 30 天 (推荐)</option>
                  <option :value="60">保留 60 天</option>
                  <option :value="90">保留 90 天</option>
                  <option :value="0">永不自动清理</option>
                </select>
              </div>
            </div>

            <div class="setting-item">
              <div class="item-content">
                <label class="item-title">偏好设置实时应用</label>
                <p class="item-desc">开启后，您在此面板的任何更改将静默自动保存并全局生效。关闭则需手动确认修改。</p>
              </div>
              <div class="item-action sync-action-group">
                <transition name="btn-expand">
                  <div v-if="!enableAutoSync" class="save-btn-wrapper">
                    <button class="primary-btn compact-save-btn" @click="syncSettings" :disabled="isSaving">
                      <span style="margin-right: 4px;">💾</span> {{ isSaving ? '保存中...' : '应用并保存' }}
                    </button>
                  </div>
                </transition>

                <label class="toggle-switch">
                  <input type="checkbox" v-model="enableAutoSync" />
                  <span class="slider round"></span>
                </label>
              </div>
            </div>

          </div>
        </div>

      </div>

      <transition name="toast-fade">
        <div v-if="toast.show" class="toast-message" :class="toast.type">{{ toast.msg }}</div>
      </transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../stores/settings';
import { useAuthStore } from '../stores/auth';
import { useWorkspaceStore } from '../stores/workspace';
import { message, open,ask } from '@tauri-apps/plugin-dialog';
import LongTestModel from '../components/LongTestModel.vue';
const settingsStore = useSettingsStore();
const authStore = useAuthStore();
const workspaceStore = useWorkspaceStore();
const isModalOpen = ref(false);
const realCpuCount = ref(16);
const isSaving = ref(false);
const licenseKey = ref('');

const enableAutoSync = ref(true);
const toast = ref({ show: false, msg: '', type: 'success' });

onMounted(async () => {
  await settingsStore.fetchSettings();
  try {
    realCpuCount.value = await invoke<number>('get_hardware_capabilities');
  } catch (e) {
    console.warn("无法获取硬件信息，采用安全后备策略", e);
  }

  if (settingsStore.maxConcurrentTasks > realCpuCount.value) {
    settingsStore.maxConcurrentTasks = realCpuCount.value;
  }
});

let syncTimeout: any = null;
watch(
    () => [
      settingsStore.maxConcurrentTasks,
      settingsStore.autoSave,
      settingsStore.trashDays,
      workspaceStore.currentWorkspace
    ],
    () => {
      if (enableAutoSync.value) {
        clearTimeout(syncTimeout);
        syncTimeout = setTimeout(() => {
          syncSettings();
        }, 500);
      }
    },
    { deep: true }
);

function showToast(msg: string, type = 'success') {
  toast.value = { show: true, msg, type };
  setTimeout(() => toast.value.show = false, 3000);
}
async function openWorkspace() {
  try {
    if (!workspaceStore.currentWorkspace) {
      showToast("❌ 工作区路径未就绪", "error");
      return;
    }
    await invoke('open_folder', { path: workspaceStore.currentWorkspace });
  } catch (error) {
    showToast(`❌ 无法打开目录`, "error");
  }
}

async function selectWorkspace() {
  try {
    const selectedPath = await open({
      directory: true,
      multiple: false,
      title: '选择易剪数据工作区'
    });
    if (selectedPath && typeof selectedPath === 'string') {
      workspaceStore.currentWorkspace = selectedPath;
    }
  } catch (error) {
    showToast(`❌ 调起目录选择失败`, "error");
  }
}

async function syncSettings() {
  isSaving.value = true;
  try {
    let safeConcurrency = settingsStore.maxConcurrentTasks;

    if (!authStore.isPro && safeConcurrency > 1) {
      console.warn("🛡️ 铁穹安全警报：检测到非法的并发越权请求，已自动修正。");
      safeConcurrency = 1;
      settingsStore.maxConcurrentTasks = 1;
    }
    if (safeConcurrency > realCpuCount.value) {
      console.warn(`🛡️ 铁穹安全警报：请求并发(${safeConcurrency})超出硬件极限，已限制为上限。`);
      safeConcurrency = realCpuCount.value;
      settingsStore.maxConcurrentTasks = realCpuCount.value;
    }

    await settingsStore.saveSettings(
        settingsStore.autoSave,
        settingsStore.trashDays,
        safeConcurrency
    );
    showToast('✅ 偏好设置已全局更新', 'success');
  } catch (error) {
    showToast(`❌ 更新失败: ${error}`, 'error');
  } finally {
    isSaving.value = false;
  }
}

async function activateLicense() {
  if (!licenseKey.value) return;
  isSaving.value = true;
  try {
    await authStore.activatePro(licenseKey.value);
    showToast('✅ PRO 旗舰版激活成功！', 'success');
    licenseKey.value = '';

    settingsStore.maxConcurrentTasks = Math.min(4, realCpuCount.value);
    await syncSettings();
  } catch (error) {
    await message(String(error), { title: '激活失败', kind: 'error' });
  } finally {
    isSaving.value = false;
  }
}

async function deactivateLicense() {
  // 1. 状态锁：防止由于鼠标连击导致同时弹出多个确认框
  if (isSaving.value) return;

  // 2. 🌟 调用 Tauri v2 插件 API：它会严格返回 true (确认) 或 false (取消/关闭)
  const confirmed = await ask("解除授权后该设备将恢复至免费版，确定继续吗？", {
    title: "安全提醒",
    kind: "warning" // v2 中支持 'info' | 'warning' | 'error'
  });

  // 3. 🌟 强制全等校验 (===)，彻底终结由于 JS 隐式真值转换引发的误判
  if (confirmed === true) {
    isSaving.value = true;
    try {
      // 所有的业务复杂性都交由 Store 层处理
      await authStore.deactivatePro();

      // 本地后续处理
      showToast('✅ 设备授权已安全解除', 'success');
      settingsStore.maxConcurrentTasks = 1;
      // await syncSettings();

    } catch (error: any) {
      showToast(`❌ 解除失败: ${error.message || error}`, 'error');
    } finally {
      isSaving.value = false;
    }
  } else {
    // 点击取消或叉掉弹窗，静默拦截
    console.info("用户取消了解绑流程");
  }
}
</script>

<style>
/* 针对 Webkit 内核 (Chrome, Safari, Edge，以及基于 Chromium 的 Tauri/Electron 窗口) */
::-webkit-scrollbar {
  width: 0 !important;
  height: 0 !important;
  display: none !important;
  background: transparent !important;
}

/* 针对全局容器的兼容性隐藏 */
html, body, #app {
  scrollbar-width: none !important; /* Firefox */
  -ms-overflow-style: none !important; /* IE/Edge */
}
</style>

<style scoped>
/* ========================================= */
/* 🧱 基础布局控制 */
/* ========================================= */
.view-scroll-container {
  height: 100%;
  width: 100%;
  overflow-y: auto;
  scrollbar-width: none !important;
  -ms-overflow-style: none !important;
}
.view-scroll-container::-webkit-scrollbar {
  display: none !important;
  width: 0 !important;
}

.view-container {
  padding: 40px 50px;
  max-width: 1280px;
  margin: 0 auto;
  animation: fadeIn 0.3s;
  padding-bottom: 80px;
}

/* ========================================= */
/* 🌟 高级感 Header 卡片 */
/* ========================================= */
.view-header-card {
  background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
  border: 1px solid rgba(226, 232, 240, 0.8);
  border-radius: 16px;
  padding: 24px 32px;
  margin-bottom: 30px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 10px 30px -10px rgba(15, 23, 42, 0.04), 0 1px 3px rgba(15, 23, 42, 0.02);
}

.header-left { display: flex; align-items: center; gap: 20px; }
.header-icon-box {
  width: 54px;
  height: 54px;
  background: linear-gradient(135deg, #eff6ff 0%, #bfdbfe 100%);
  border: 1px solid #93c5fd;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15), inset 0 2px 4px rgba(255, 255, 255, 0.6);
}

.header-text { display: flex; flex-direction: column; gap: 4px; }
.view-title { font-size: 1.55rem; font-weight: 800; color: #1e293b; margin: 0; letter-spacing: -0.5px; }
.view-subtitle { color: #64748b; font-size: 0.95rem; margin: 0; font-weight: 500; }

.system-status-badge {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  background: #ffffff;
  padding: 8px 16px;
  border-radius: 30px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 2px 6px rgba(0,0,0,0.02);
}
.status-text { font-size: 0.85rem; color: #475569; font-weight: 600; }
.pulse-dot { width: 8px; height: 8px; background-color: #10b981; border-radius: 50%; box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.4); animation: pulse 2s infinite; }
@keyframes pulse {
  0% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.4); }
  70% { box-shadow: 0 0 0 6px rgba(16, 185, 129, 0); }
  100% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0); }
}

/* ========================================= */
/* 卡片与表单基础样式 */
/* ========================================= */
.settings-layout { display: flex; flex-direction: column; gap: 1.5rem; flex: 1; }
.card { background: white; border-radius: 12px; box-shadow: 0 4px 12px rgba(0,0,0,0.03); border: 1px solid #e2e8f0; overflow: hidden; }
.card-header { padding: 18px 24px; border-bottom: 1px solid #f1f5f9; display: flex; align-items: center; gap: 12px; background: #f8fafc; }
.card-header h2 { margin: 0; font-size: 1.1rem; color: #1f2937; font-weight: 700; }
.card-body.padded { padding: 1.5rem 2rem; }

.setting-item { display: flex; justify-content: space-between; align-items: flex-start; padding: 1rem 0; border-bottom: 1px dashed #e5e7eb; gap: 1.5rem; flex-wrap: wrap; }
.setting-item:last-child { border-bottom: none; padding-bottom: 0; }
.item-content { flex: 1; min-width: 0; padding-top: 4px;}
.item-title { display: block; font-size: 1rem; font-weight: 700; color: #1e293b; margin-bottom: 0.4rem; letter-spacing: 0.3px;}
.item-desc { margin: 0; font-size: 0.85rem; color: #64748b; line-height: 1.65; word-wrap: break-word; white-space: normal; font-weight: 400;}
.item-action { flex-shrink: 0; width: auto; display: flex; align-items: center; }

/* 表单组件 */
.workspace-action { gap: 10px; width: 100%; max-width: 700px; flex-wrap: nowrap; }
.readonly-input { flex: 1; min-width: 100px; padding: 0.5rem 1rem; border: 1px solid #cbd5e1; border-radius: 8px; background: #f8fafc; color: #475569; font-family: 'Consolas', monospace; font-size: 0.85rem; outline: none; text-overflow: ellipsis; transition: all 0.2s;}
.readonly-input:focus { border-color: #3b82f6; background: #fff; box-shadow: 0 0 0 3px rgba(59,130,246,0.1); }

.compact-btn { padding: 0.5rem 0.85rem; background: #ffffff; border: 1px solid #cbd5e1; border-radius: 8px; color: #475569; font-size: 0.85rem; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 4px; transition: all 0.2s; white-space: nowrap;}
.compact-btn:hover { background: #f8fafc; border-color: #94a3b8; color: #0f172a; box-shadow: 0 2px 4px rgba(0,0,0,0.02);}

.secondary-btn { padding: 0.6rem 1.2rem; background: white; border: 1px solid #d1d5db; border-radius: 8px; color: #374151; font-weight: 600; cursor: pointer; white-space: nowrap; transition: all 0.2s; }
.secondary-btn:hover { background: #f1f5f9; border-color: #cbd5e1;}

.primary-btn { padding: 0.6rem 1.5rem; background: linear-gradient(135deg, #2563eb, #1d4ed8); color: white; border: none; border-radius: 8px; font-weight: 600; cursor: pointer; white-space: nowrap; transition: all 0.2s; box-shadow: 0 4px 6px -1px rgba(37, 99, 235, 0.2);}
.primary-btn:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 6px 8px -1px rgba(37, 99, 235, 0.3); }
.primary-btn:disabled { background: #94a3b8; cursor: not-allowed; box-shadow: none; opacity: 0.8;}

.custom-select {
  padding: 0.55rem 2.2rem 0.55rem 1rem;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  background-color: #f8fafc;
  color: #1e293b;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  outline: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%2364748b'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'%3E%3C/path%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  background-size: 16px;
  transition: all 0.2s;
  min-width: 170px;
}
.custom-select:hover { border-color: #94a3b8; }
.custom-select:focus { border-color: #3b82f6; background-color: #ffffff; box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1); }


/* ========================================= */
/* 🌟 核心优化2：彻底解决抖动，引入丝滑弹性动画 */
/* ========================================= */
.sync-action-group {
  display: flex;
  align-items: center;
  /* 不使用 gap，防止动画过程中布局跳变 */
}

.save-btn-wrapper {
  overflow: hidden;
  white-space: nowrap;
  padding-right: 16px; /* 代替 gap 充当间距 */
  transform-origin: right center; /* 设定动画轴心：贴紧右侧开关 */
}

.compact-save-btn {
  padding: 0.5rem 1.2rem;
  font-size: 0.85rem;
  border-radius: 6px;
  white-space: nowrap; /* 绝对禁止内部文字在动画时换行折叠 */
}

/* 带有 Apple 阻尼感的丝滑复合过渡 */
.btn-expand-enter-active, .btn-expand-leave-active {
  transition:
      max-width 0.35s cubic-bezier(0.2, 0.8, 0.2, 1),
      padding-right 0.35s cubic-bezier(0.2, 0.8, 0.2, 1),
      opacity 0.25s ease-out,
      transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1); /* 具有弹性的形变曲线 */
  max-width: 160px; /* 安全撑开的物理极限 */
  opacity: 1;
  transform: scale(1);
}

.btn-expand-enter-from, .btn-expand-leave-to {
  max-width: 0;
  opacity: 0;
  padding-right: 0; /* 收缩至 0，无缝消失 */
  transform: scale(0.85); /* 轻微收缩增强空间立体感 */
}

/* ========================================= */
/* iOS 风格平滑开关 */
/* ========================================= */
.toggle-switch { position: relative; display: inline-block; width: 46px; height: 26px; flex-shrink: 0;}
.toggle-switch input { opacity: 0; width: 0; height: 0; }
.slider { position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: #cbd5e1; transition: .3s; border-radius: 34px;}
.slider:before { position: absolute; content: ""; height: 18px; width: 18px; left: 4px; bottom: 4px; background-color: white; transition: .3s; box-shadow: 0 2px 4px rgba(0,0,0,0.2); border-radius: 50%;}
input:checked + .slider { background-color: #10b981; }
input:checked + .slider:before { transform: translateX(20px); }

/* ========================================= */
/* PRO 商业化授权卡片 (原有逻辑全保留) */
/* ========================================= */
.auth-card { transition: all 0.4s ease; border: 2px solid transparent; }
.auth-card.is-pro { border-color: #c084fc; box-shadow: 0 10px 25px -5px rgba(168, 85, 247, 0.15); }
.auth-card.is-pro .card-header { background: linear-gradient(to right, #faf5ff, white); }
.auth-body { padding-top: 0.5rem; }

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

.free-status { display: flex; flex-direction: column; gap: 1rem; background: #f8fafc; padding: 1.5rem; border-radius: 12px; border: 1px dashed #cbd5e1;}
.free-status h3 { margin: 0 0 0.5rem 0; color: #334155; font-size: 1.15rem;}
.free-status p { margin: 0; color: #64748b; font-size: 0.95rem; }

.activation-input-group { display: flex; align-items: stretch; gap: 12px; margin-top: 1rem; max-width: 750px; }
.input-wrapper { position: relative; flex: 1; }
.key-icon { position: absolute; left: 14px; top: 50%; transform: translateY(-50%); opacity: 0.6; font-size: 1.1rem; }
.license-input { width: 100%; height: 100%; padding: 0.85rem 1rem 0.85rem 2.8rem; border: 1px solid #cbd5e1; border-radius: 8px; font-family: 'Consolas', monospace; font-size: 0.95rem; outline: none; transition: all 0.2s; background: white; box-sizing: border-box; box-shadow: inset 0 1px 3px rgba(0,0,0,0.03); }
.license-input:focus { border-color: #2563eb; box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.15), inset 0 1px 3px rgba(0,0,0,0.03); }

.activate-btn { width: 150px !important; min-width: 150px !important; max-width: 150px !important; flex-shrink: 0; display: flex; justify-content: center; align-items: center; gap: 8px; box-sizing: border-box; }
.spinner-svg { width: 1.2rem; height: 1.2rem; animation: spin 1s linear infinite; color: white; }
.opacity-25 { opacity: 0.25; }
.opacity-75 { opacity: 0.75; }
@keyframes spin { 100% { transform: rotate(360deg); } }

/* ========================================= */
/* 🚀 性能调度拉杆 */
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
.toast-message { position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%); color: white; padding: 0.8rem 1.8rem; border-radius: 30px; font-size: 0.95rem; box-shadow: 0 10px 25px -5px rgba(0,0,0,0.2); z-index: 9999; font-weight: 600;}
.toast-message.success { background: #10b981; border: 1px solid #059669; }
.toast-message.error { background: #ef4444; border: 1px solid #dc2626;}
.toast-message.info { background: #3b82f6; border: 1px solid #2563eb;}
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
</style>