<template>
  <div class="view-container">
    <header class="view-header">
      <h1 class="view-title">全局偏好设置</h1>
      <p class="view-subtitle">定制您的易剪工作区、数据安全策略与核心工作流。</p>
    </header>

    <div class="settings-layout">
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

      <div class="card">
        <div class="card-header">
          <span class="icon">⚡</span>
          <h2>工作流与剪辑</h2>
        </div>
        <div class="card-body padded">
          <div class="setting-item">
            <div class="item-content">
              <label class="item-title">开启自动保存 (推荐)</label>
              <p class="item-desc">在“智能打轴标记”中，任何新增、修改或删除操作将静默同步到硬盘，防止突发断电导致心血丢失。</p>
            </div>
            <div class="item-action toggle-action">
              <label class="toggle-switch">
                <input type="checkbox" v-model="settingsStore.autoSave" @change="syncSettings" />
                <span class="slider round"></span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <span class="icon">🗑️</span>
          <h2>数据安全与回收站</h2>
        </div>
        <div class="card-body padded">
          <div class="setting-item">
            <div class="item-content">
              <label class="item-title">废弃配置保留周期</label>
              <p class="item-desc">被清空的 JSON 标记文件将被移入回收站。软件将在每次启动时自动清理超过该期限的废件。</p>
            </div>
            <div class="item-action select-action">
              <select class="custom-select" v-model.number="settingsStore.trashDays" @change="syncSettings">
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
      <div v-if="toastMsg" class="toast-message">{{ toastMsg }}</div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../stores/settings';
import { useWorkspaceStore } from '../stores/workspace';

const settingsStore = useSettingsStore();
const workspaceStore = useWorkspaceStore();

const toastMsg = ref('');
let toastTimer: number | null = null;

async function syncSettings() {
  try {
    await settingsStore.saveSettings(settingsStore.autoSave, settingsStore.trashDays);
    showToast('✅ 偏合设置已全局更新');
  } catch (error) {
    showToast(`❌ 更新失败: ${error}`);
  }
}

async function openWorkspace() {
  try {
    if (!workspaceStore.currentWorkspace) { showToast("❌ 工作区路径未就绪"); return; }
    await invoke('open_folder', { path: workspaceStore.currentWorkspace });
  } catch (error) { showToast(`❌ 无法打开目录`); }
}

function showToast(msg: string) {
  toastMsg.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => { toastMsg.value = ''; }, 2000);
}
</script>

<style scoped>
/* ========================================= */
/* 🧱 设置页面独有样式 (已开启 Scoped，不再干扰全局) */
/* ========================================= */

/* 1. 响应式布局核心修复 (重要！基于 950px 的底线) */

/* 瀑布流布局不再需要垂直滚动，因为通用的 view-container 已经有了滚动 */
.settings-layout {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  flex: 1;
}

/* 核心修复：复用卡片规范，移除内部滚动 */
.card-header h2 { margin: 0; font-size: 1.1rem; color: #1f2937; font-weight: 700; }
.card-body.padded { padding: 1.5rem; }

/* 核心修复：使用 Flex 布局，并允许自动换行以应对极窄空间 */
.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start; /* 改为 flex-start，上下结构时文字对齐更舒服 */
  padding: 1rem 0;
  border-bottom: 1px solid #f3f4f6;
  gap: 1.5rem; /* 增加左右间距 */
  flex-wrap: wrap; /* 极其重要：在崩溃前允许操作区自动掉下去 */
}
.setting-item:last-child { border-bottom: none; }

/* 核心修复：文字区域约束 (解决图1截断) */
.item-content {
  flex: 1;
  min-width: 0; /* 允许 Flex 子元素收缩，核心！ */
  /* 不要在这里写硬硬的宽度，让 Flex 去分配 */
}
.item-title { display: block; font-size: 1.05rem; font-weight: 600; color: #374151; margin-bottom: 0.35rem; }
.item-desc {
  margin: 0;
  font-size: 0.85rem;
  color: #6b7280;
  line-height: 1.5;
  word-wrap: break-word; /* 允许单词换行，防撑破 */
  white-space: normal;  /* 允许自动换行 */
}

/* 核心修复：操作区域约束 (解决图1挤压) */
.item-action {
  flex-shrink: 0; /* 操作区不要被压缩 */
  /* 如果在上下堆叠状态下，操作区撑满整行 */
  width: auto;
}

/* 数据与存储的路径输入框区域美化 */
.workspace-action {
  display: flex;
  gap: 10px;
  align-items: center;
  /* 当左右布局时，输入框区域的最大宽度 */
  width: 100%;
  max-width: 450px;
  min-width: 0;
  flex-wrap: wrap; /* 允许打开目录按钮在输入框下换行（如果极挤） */
}

.readonly-input {
  flex: 1;
  min-width: 100px; /* 输入框的最小压缩宽度 */
  padding: 0.6rem 1rem; border: 1px solid #d1d5db; border-radius: 8px;
  background: #f3f4f6; color: #4b5563; font-family: monospace; font-size: 0.85rem; outline: none;
  text-overflow: ellipsis; /* 文字过长显示省略号 */
}
.secondary-btn {
  padding: 0.6rem 1rem; background: white; border: 1px solid #d1d5db; border-radius: 8px;
  color: #374151; font-weight: 600; cursor: pointer; white-space: nowrap;
}
.secondary-btn:hover { background: #f9fafb;}

.custom-select {
  padding: 0.6rem 1rem; border: 1px solid #d1d5db; border-radius: 8px; background: white;
  color: #1f2937; font-size: 0.95rem; font-weight: 500; cursor: pointer; outline: none; min-width: 160px;
}

/* iOS 开关 */
.toggle-switch { position: relative; display: inline-block; width: 50px; height: 28px; }
.toggle-switch input { opacity: 0; width: 0; height: 0; }
.slider { position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: #cbd5e1; transition: .3s; border-radius: 34px;}
.slider:before { position: absolute; content: ""; height: 20px; width: 20px; left: 4px; bottom: 4px; background-color: white; transition: .3s; box-shadow: 0 2px 4px rgba(0,0,0,0.2); border-radius: 50%;}
input:checked + .slider { background-color: #10b981; }
input:checked + .slider:before { transform: translateX(22px); }

/* 轻提示样式复用 */
.toast-message {
  position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%);
  background: #1f2937; color: white; padding: 0.75rem 1.5rem; border-radius: 30px;
  font-size: 0.95rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); z-index: 9999;
}
</style>