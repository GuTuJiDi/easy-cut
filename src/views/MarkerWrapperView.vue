<template>
  <div class="wrapper-container">
    <div class="tabs-header">
      <div class="tabs-nav">
        <button
            class="tab-btn"
            :class="{ active: activeTab === 'manual' }"
            @click="activeTab = 'manual'"
        >
          <span class="icon">⌨️</span> 手工精确打轴
        </button>

        <button
            class="tab-btn"
            :class="{ active: activeTab === 'ai', disabled: !authStore.isPro }"
            @click="handleAITabClick"
            :title="authStore.isPro ? '进入 AI 智能打轴' : 'PRO 旗舰版专属功能'"
        >
          <span class="icon">🤖</span> AI 智能识别打轴 <span class="pro-tag">PRO</span>
        </button>
      </div>

      <div class="header-tools">
        <button
            class="widget-toggle-btn"
            :class="{ 'is-active': isWidgetOpen }"
            @click="handleToggleWidget"
            title="全局快捷键: Alt+W 开关 | Ctrl+Shift+M 盲打"
        >
          <span class="status-dot" :class="isWidgetOpen ? 'on' : 'off'"></span>
          <span class="btn-text">{{ isWidgetOpen ? '伴随模式运行中' : '开启桌面悬浮伴随' }}</span>
        </button>
      </div>
    </div>

    <div class="module-content">
      <KeepAlive>
        <ManualMarker v-if="activeTab === 'manual'" />
        <AIMarker v-else-if="activeTab === 'ai'" />
      </KeepAlive>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import ManualMarker from '../components/ManualMarker.vue';
import AIMarker from '../components/AIMarker.vue';
import { useAuthStore } from '../stores/auth';

const authStore = useAuthStore();
const activeTab = ref<'manual' | 'ai'>('manual');

// 🌟 悬浮球状态管理
const isWidgetOpen = ref(false);
let unlistenWidgetState: UnlistenFn | null = null;

function handleAITabClick() {
  if (!authStore.isPro) {
    alert('AI 智能识别为 PRO 旗舰版专属能力，请先前往【全局偏好设置】激活！');
  } else {
    activeTab.value = 'ai';
  }
}

// 🌟 调用 Rust 后端开关指令
async function handleToggleWidget() {
  try {
    const status = await invoke<string>('toggle_widget');
    isWidgetOpen.value = (status === 'opened');
  } catch (err) {
    console.error("悬浮球操作失败:", err);
  }
}

onMounted(async () => {
  // 🌟 监听 Rust 发来的窗口状态变更
  // 确保按下 Alt+W 快捷键时，此处的 UI 按钮也能同步变色
  unlistenWidgetState = await listen<string>('widget-state-changed', (event) => {
    isWidgetOpen.value = (event.payload === 'opened');
  });
});

onUnmounted(() => {
  if (unlistenWidgetState) unlistenWidgetState();
});
</script>

<style scoped>
.wrapper-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 1.5rem 2rem;
  box-sizing: border-box;
  background: #f4f6f8;
}

.tabs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 1.5rem;
  flex-shrink: 0;
}

.tabs-nav {
  display: inline-flex;
  background: #e2e8f0;
  padding: 5px;
  border-radius: 12px;
  gap: 4px;
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);
}

.tab-btn {
  padding: 8px 24px;
  border: none;
  background: transparent;
  border-radius: 8px;
  font-size: 0.95rem;
  font-weight: 600;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 8px;
}

.tab-btn.active {
  background: white;
  color: #0f172a;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.tab-btn.disabled { opacity: 0.5; cursor: not-allowed; }

.pro-tag {
  font-size: 0.65rem;
  background: linear-gradient(135deg, #3b82f6, #1d4ed8);
  color: white;
  padding: 2px 6px;
  border-radius: 4px;
}

/* 🌟 悬浮球按钮样式 */
.widget-toggle-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 18px;
  border-radius: 20px;
  border: 1px solid #cbd5e1;
  background: white;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 0.9rem;
  font-weight: 600;
  color: #475569;
}

.widget-toggle-btn:hover {
  border-color: #94a3b8;
  transform: translateY(-1px);
  background: #f8fafc;
}

.widget-toggle-btn.is-active {
  background: #eff6ff;
  border-color: #3b82f6;
  color: #1e40af;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  transition: all 0.3s;
}
.status-dot.off { background: #94a3b8; }
.status-dot.on {
  background: #10b981;
  box-shadow: 0 0 8px #34d399;
  animation: pulse-dot 2s infinite;
}

@keyframes pulse-dot {
  0% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.3); opacity: 0.7; }
  100% { transform: scale(1); opacity: 1; }
}

.module-content { flex: 1; min-height: 0; position: relative; }
</style>