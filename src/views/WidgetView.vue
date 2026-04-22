<template>
  <div
      class="widget-container"
      :class="{ 'is-minimized': isMinimized }"
      @mousedown="startDrag"
  >
    <div class="card-wrapper">
      <div class="glow-card" :class="{ 'pulse': isRecording }">

        <transition name="fade-scale" mode="out-in">
          <div v-if="!isMinimized" class="expanded-content">
            <div class="timer-wrapper">
              <span class="status-indicator"></span>
              <span class="time-text">{{ formattedTime }}</span>
            </div>
            <div class="hint-text">Alt+W 显隐 | Ctrl+Shift+M 盲打</div>
          </div>

          <div v-else class="minimized-content">
            <div class="mini-dot"></div>
          </div>
        </transition>

      </div>

      <transition-group name="pop">
        <div
            v-for="f in feedbacks"
            :key="f.id"
            class="marker-feedback"
        >+1</div>
      </transition-group>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

// --- 状态管理 ---
const elapsedTime = ref(0);
const isRecording = ref(false);
const isMinimized = ref(false);
const feedbacks = ref<{ id: number }[]>([]);
let timerId: number | null = null;
let unlistens: UnlistenFn[] = [];

// --- 格式化 HH:MM:SS ---
const formattedTime = computed(() => {
  const s = Math.floor(elapsedTime.value);
  const hrs = Math.floor(s / 3600).toString().padStart(2, '0');
  const mins = Math.floor((s % 3600) / 60).toString().padStart(2, '0');
  const secs = (s % 60).toString().padStart(2, '0');
  return `${hrs}:${mins}:${secs}`;
});

// --- 计时器控制 ---
const startTimer = () => {
  if (timerId) clearInterval(timerId);
  elapsedTime.value = 0;
  isRecording.value = true;
  timerId = window.setInterval(() => elapsedTime.value++, 1000);
};

const stopTimer = () => {
  if (timerId) clearInterval(timerId);
  timerId = null;
  isRecording.value = false;
};

// --- 100% 可靠的系统级拖拽 API ---
const startDrag = async (e: MouseEvent) => {
  if (e.target instanceof HTMLElement && e.target.tagName.toLowerCase() === 'button') return;
  try {
    await appWindow.startDragging();
  } catch (err) {
    console.error('Drag failed', err);
  }
};

// --- 纯 CSS 缩放 ---
const handleFocusChange = (focused: boolean) => {
  isMinimized.value = !focused;
};

onMounted(async () => {
  // 彻底消灭 Windows 白边与阴影
  try {
    await appWindow.setShadow(false);
    await appWindow.setDecorations(false);
  } catch (e) {}

  // 终极穿透样式注入
  document.documentElement.style.setProperty('background', 'transparent', 'important');
  document.body.style.setProperty('background', 'transparent', 'important');
  const appEl = document.getElementById('app');
  if (appEl) appEl.style.setProperty('background', 'transparent', 'important');

  startTimer();

  // 独立挂载监听器
  const safeListen = async (eventName: string, handler: any) => {
    try {
      const unlisten = await listen(eventName, handler);
      unlistens.push(unlisten);
    } catch (e) {
      console.error(`Failed to listen to ${eventName}`, e);
    }
  };

  await safeListen('widget-session-start', startTimer);
  await safeListen('widget-session-stop', stopTimer);
  await safeListen('widget-state-changed', (e: any) => {
    if (e.payload === 'opened') startTimer();
    else stopTimer();
  });
  await safeListen('widget-marker-added', () => {
    // 🌟 核心改进 1: 确保随机唯一ID，防止多设备事件并发导致重绘抖动
    const id = `${Date.now()}-${Math.random()}`;
    feedbacks.value.push({ id });
    setTimeout(() => {
      feedbacks.value = feedbacks.value.filter(f => f.id !== id);
    }, 1000);
  });

  try {
    const unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
      handleFocusChange(focused);
    });
    unlistens.push(unlistenFocus);
  } catch (e) {
    console.error("Focus listener failed", e);
  }
});

onUnmounted(() => {
  if (timerId) clearInterval(timerId);
  unlistens.forEach(fn => fn());
});
</script>

<style>
html, body, #app {
  background: transparent !important;
  background-color: transparent !important;
  margin: 0 !important; padding: 0 !important; overflow: hidden !important;
}
</style>

<style scoped>
.widget-container {
  width: 100vw; height: 100vh;
  display: flex; align-items: center; justify-content: center;
  background: transparent !important;
  user-select: none;
  cursor: grab;
}
.widget-container:active {
  cursor: grabbing;
}

.card-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.glow-card {
  position: relative;
  width: 220px;
  height: 60px;
  background: rgba(15, 23, 42, 0.85);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(56, 189, 248, 0.4);
  border-radius: 30px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  transition: width 0.4s cubic-bezier(0.34, 1.56, 0.64, 1),
  height 0.4s cubic-bezier(0.34, 1.56, 0.64, 1),
  background 0.4s, border-color 0.4s, border-radius 0.4s;
  will-change: width, height;
  overflow: hidden;
}

.pulse { animation: breathe 3s infinite ease-in-out; }
@keyframes breathe {
  0%, 100% { box-shadow: 0 0 15px rgba(56, 189, 248, 0.2); }
  50% { box-shadow: 0 0 25px rgba(56, 189, 248, 0.5); }
}

.expanded-content, .minimized-content {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.timer-wrapper {
  display: flex; align-items: center; gap: 10px;
}
.status-indicator {
  width: 8px; height: 8px; border-radius: 50%;
  background: #10b981; box-shadow: 0 0 8px #10b981;
  flex-shrink: 0;
}
.time-text {
  color: #f8fafc; font-family: 'Consolas', monospace;
  font-size: 1.5rem; font-weight: 700; letter-spacing: 1px;
  white-space: nowrap;
}
.hint-text {
  font-size: 0.65rem; color: rgba(148, 163, 184, 0.8);
  margin-top: -2px; font-weight: 600; white-space: nowrap;
}

.is-minimized .glow-card {
  width: 44px;
  height: 44px;
  border-radius: 22px;
  background: rgba(15, 23, 42, 0.7);
  border-color: rgba(56, 189, 248, 0.3);
  box-shadow: none;
}
.is-minimized:hover .glow-card {
  border-color: rgba(56, 189, 248, 0.8);
  background: rgba(15, 23, 42, 0.9);
}

.mini-dot {
  width: 10px; height: 10px; border-radius: 50%;
  background: #38bdf8; animation: blink 2s infinite;
}
@keyframes blink { 0%, 100% { opacity: 0.4; } 50% { opacity: 1; } }

.fade-scale-enter-active, .fade-scale-leave-active { transition: all 0.2s ease; }
.fade-scale-enter-from, .fade-scale-leave-to { opacity: 0; transform: scale(0.9); }

/* 🌟 核心修复 2: 重写打轴反馈动画 (消除卡顿/抖动) */

/* 1. 进入动画：更优雅的弹出效果 */
.pop-enter-active {
  animation: pop-in 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  will-change: transform, opacity;
}

/* 2. 🌟 新增：离开动画 (离开不再立刻消失，而是流畅的淡出) */
.pop-leave-active {
  transition: all 0.4s cubic-bezier(0.165, 0.84, 0.44, 1);
  /* will-change 在离开时同样重要，防止 GPU 重新排版导致抖动 */
  will-change: transform, opacity;
  position: absolute; /* 🌟 关键：脱离文档流，不干扰新气泡的弹出定位 */
}

@keyframes pop-in {
  0% { transform: scale(0.5) translateY(10px); opacity: 0; }
  100% { transform: scale(1) translateY(0); opacity: 1; }
}

/* 3. 进入前和离开后的样式状态 */
.pop-enter-from, .pop-leave-to {
  opacity: 0;
  transform: scale(0.8) translateY(-30px); /* 流畅地向左上方淡出 */
}

/* 4. 打轴反馈气泡本身的基础样式调整 */
.marker-feedback {
  position: absolute;
  /* 调整初始位置，防止弹出时触碰glow-card边界 */
  right: -5px; top: -5px;
  background: #38bdf8;
  color: white;
  padding: 4px 10px;
  border-radius: 14px;
  font-size: 0.9rem;
  font-weight: 800;
  pointer-events: none;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(56, 189, 248, 0.3);
}
</style>