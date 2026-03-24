<template>
  <div class="view-container">
    <header class="view-header">
      <h1 class="view-title">全局偏好设置</h1>
      <p class="view-subtitle">定制您的易剪工作区、数据安全策略与核心工作流。</p>
    </header>

    <div class="settings-layout">
      <div class="settings-card">
        <div class="card-header">
          <span class="icon">🗄️</span>
          <h2>数据与存储</h2>
        </div>
        <div class="card-body">
          <div class="setting-item">
            <div class="item-content">
              <label class="item-title">易剪工作区 (Workspace)</label>
              <p class="item-desc">所有配置文件、标记数据、自动日志均统一安全存放于此。</p>
            </div>
            <div class="item-action workspace-action">
              <input type="text" :value="workspacePath" readonly class="readonly-input path-input" />
              <button class="secondary-btn" @click="openWorkspace" title="在资源管理器中打开">
                📂 打开目录
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="settings-card">
        <div class="card-header">
          <span class="icon">⚡</span>
          <h2>工作流与剪辑</h2>
        </div>
        <div class="card-body">
          <div class="setting-item">
            <div class="item-content">
              <label class="item-title">开启自动保存 (推荐)</label>
              <p class="item-desc">在“智能打轴标记”中，任何新增、修改或删除操作将静默同步到硬盘，防止突发断电导致心血丢失。</p>
            </div>
            <div class="item-action">
              <label class="toggle-switch">
                <input type="checkbox" v-model="settings.autoSaveEnabled" @change="syncSettings" />
                <span class="slider round"></span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <div class="settings-card">
        <div class="card-header">
          <span class="icon">🗑️</span>
          <h2>数据安全与回收站</h2>
        </div>
        <div class="card-body">
          <div class="setting-item">
            <div class="item-content">
              <label class="item-title">废弃配置保留周期</label>
              <p class="item-desc">被清空的 JSON 标记文件将被移入回收站。软件将在每次启动时自动清理超过该期限的废件。</p>
            </div>
            <div class="item-action">
              <select class="custom-select" v-model.number="settings.trashRetentionDays" @change="syncSettings">
                <option :value="7">保留 7 天</option>
                <option :value="15">保留 15 天</option>
                <option :value="30">保留 30 天 (默认)</option>
                <option :value="90">保留 90 天</option>
                <option :value="0">永不自动清理 (需手动删)</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>

    <transition name="toast">
      <div v-if="toastMsg" class="toast-message">
        {{ toastMsg }}
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 数据契约必须与 Rust 后端 AppSettings 结构体严格保持一致
interface AppSettings {
  autoSaveEnabled: boolean;
  trashRetentionDays: number;
}

const workspacePath = ref('读取中...');
const settings = reactive<AppSettings>({
  autoSaveEnabled: true,
  trashRetentionDays: 30
});

const toastMsg = ref('');
let toastTimer: number | null = null;

// 初始化：拉取真实数据
onMounted(async () => {
  try {
    workspacePath.value = await invoke<string>('get_workspace_path');
    const res = await invoke<AppSettings>('get_app_settings');
    settings.autoSaveEnabled = res.autoSaveEnabled;
    settings.trashRetentionDays = res.trashRetentionDays;
  } catch (error) {
    console.error("加载设置失败:", error);
  }
});

// 即时同步：一旦用户操作界面，立刻落盘
async function syncSettings() {
  try {
    await invoke('update_app_settings', { settings: settings });
    showToast('✅ 偏好设置已更新');
  } catch (error) {
    showToast(`❌ 更新失败: ${error}`);
  }
}

// 打开系统资源管理器
async function openWorkspace() {
  try {
    await invoke('open_folder', { path: workspacePath.value });
  } catch (error) {
    showToast(`❌ 无法打开目录`);
  }
}

function showToast(msg: string) {
  toastMsg.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => { toastMsg.value = ''; }, 2000);
}
</script>

<style scoped>
/* 统一的页面容器规范 */
.view-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 2.5rem;
  box-sizing: border-box;
  animation: fadeIn 0.3s ease-in-out;
  max-width: 1000px;
  margin: 0 auto;
}

.view-header { margin-bottom: 2rem; flex-shrink: 0; }
.view-title { font-size: 1.85rem; font-weight: 800; color: #111827; margin: 0 0 0.4rem 0; letter-spacing: -0.5px;}
.view-subtitle { color: #6b7280; margin: 0; font-size: 1rem; }

/* 瀑布流布局 */
.settings-layout {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  flex: 1;
  overflow-y: auto;
  padding-right: 10px; /* 为滚动条留白 */
}

.settings-card {
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #e5e7eb;
  box-shadow: 0 1px 3px rgba(0,0,0,0.02);
  overflow: hidden;
}

.card-header {
  padding: 1.25rem 1.5rem;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  display: flex;
  align-items: center;
  gap: 10px;
}

.card-header h2 { margin: 0; font-size: 1.1rem; color: #1f2937; font-weight: 700; }
.card-header .icon { font-size: 1.2rem; }

.card-body { padding: 0 1.5rem; }

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem 0;
  border-bottom: 1px solid #f3f4f6;
}
.setting-item:last-child { border-bottom: none; }

.item-content { flex: 1; padding-right: 2rem; }
.item-title { display: block; font-size: 1.05rem; font-weight: 600; color: #374151; margin-bottom: 0.35rem; }
.item-desc { margin: 0; font-size: 0.85rem; color: #6b7280; line-height: 1.5; }

.item-action { flex-shrink: 0; }
.workspace-action { display: flex; gap: 10px; align-items: center; width: 400px; }

/* 表单元素美化 */
.readonly-input {
  flex: 1; padding: 0.6rem 1rem; border: 1px solid #d1d5db; border-radius: 8px;
  background: #f3f4f6; color: #4b5563; font-family: monospace; font-size: 0.85rem; outline: none;
}
.secondary-btn {
  padding: 0.6rem 1rem; background: white; border: 1px solid #d1d5db; border-radius: 8px;
  color: #374151; font-weight: 600; cursor: pointer; transition: all 0.2s; white-space: nowrap;
}
.secondary-btn:hover { background: #f9fafb; border-color: #9ca3af; box-shadow: 0 1px 2px rgba(0,0,0,0.05);}

.custom-select {
  padding: 0.6rem 1rem; border: 1px solid #d1d5db; border-radius: 8px; background: white;
  color: #1f2937; font-size: 0.95rem; font-weight: 500; cursor: pointer; outline: none; min-width: 160px;
  appearance: none; background-image: url("data:image/svg+xml;charset=US-ASCII,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%22292.4%22%20height%3D%22292.4%22%3E%3Cpath%20fill%3D%22%236b7280%22%20d%3D%22M287%2069.4a17.6%2017.6%200%200%200-13-5.4H18.4c-5%200-9.3%201.8-12.9%205.4A17.6%2017.6%200%200%200%200%2082.2c0%205%201.8%209.3%205.4%2012.9l128%20127.9c3.6%203.6%207.8%205.4%2012.8%205.4s9.2-1.8%2012.8-5.4L287%2095c3.5-3.5%205.4-7.8%205.4-12.8%200-5-1.9-9.2-5.5-12.8z%22%2F%3E%3C%2Fsvg%3E");
  background-repeat: no-repeat; background-position: right 1rem top 50%; background-size: 0.65rem auto;
}
.custom-select:focus { border-color: #3b82f6; box-shadow: 0 0 0 3px rgba(59,130,246,0.1); }

/* 拟物化 iOS 开关 */
.toggle-switch { position: relative; display: inline-block; width: 50px; height: 28px; }
.toggle-switch input { opacity: 0; width: 0; height: 0; }
.slider { position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: #cbd5e1; transition: .3s; }
.slider:before { position: absolute; content: ""; height: 20px; width: 20px; left: 4px; bottom: 4px; background-color: white; transition: .3s; box-shadow: 0 2px 4px rgba(0,0,0,0.2); }
input:checked + .slider { background-color: #10b981; } /* 成功绿 */
input:focus + .slider { box-shadow: 0 0 1px #10b981; }
input:checked + .slider:before { transform: translateX(22px); }
.slider.round { border-radius: 34px; }
.slider.round:before { border-radius: 50%; }

/* 轻提示 */
.toast-message {
  position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%);
  background: #1f2937; color: white; padding: 0.75rem 1.5rem; border-radius: 30px;
  font-size: 0.95rem; font-weight: 500; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); z-index: 50;
}

@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
.toast-enter-active, .toast-leave-active { transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275); }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translate(-50%, 20px); }

/* 滚动条美化 */
.settings-layout::-webkit-scrollbar { width: 6px; }
.settings-layout::-webkit-scrollbar-track { background: transparent; }
.settings-layout::-webkit-scrollbar-thumb { background-color: #d1d5db; border-radius: 10px; }
</style>