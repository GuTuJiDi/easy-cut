<template>
  <div class="view-container">
    <header class="view-header">
      <div>
        <h1 class="view-title">智能打轴标记</h1>
        <p class="view-subtitle">利用快捷键极速标记：[I]入点, [O]出点, [F]全屏, [Z/X/C]微调倍速, [L]标记列表。</p>
      </div>
      <div class="header-actions">
        <div class="input-with-btn">
          <input v-model="videoPath" type="text" readonly placeholder="尚未加载视频..." class="file-input compact" />
          <button class="primary-btn" @click="selectVideo">
            <span class="icon">📁</span> 加载视频
          </button>
        </div>
      </div>
    </header>
    <div class="workspace-grid" v-if="videoSrc">

      <div class="card player-card" ref="playerContainerRef" :class="{ 'is-fullscreen': isFullscreen }">

        <div class="video-wrapper" @mousemove="handleMouseMove" @mouseleave="handleMouseLeave" @click="togglePlay" :class="{ 'hide-cursor': !showControls && isPlaying }">
          <video
              ref="videoRef" :src="videoSrc" class="video-element"
              @timeupdate="onTimeUpdate" @loadedmetadata="onLoadedMetadata"
              @play="isPlaying = true" @pause="isPlaying = false" @ended="isPlaying = false"
              @volumechange="onNativeVolumeChange"
          ></video>

          <transition name="fade">
            <div class="osd-message" v-if="osdVisible">{{ osdText }}</div>
          </transition>

          <div class="play-overlay" :class="{ 'is-playing': isPlaying }">
            <div class="play-icon-center">▶</div>
          </div>

          <transition name="slide-right">
            <div class="fullscreen-drawer" v-show="isFullscreen && showDrawer" @click.stop>
              <div class="drawer-header">
                <h3>标记列表 ({{ markers.length }})</h3>
                <button class="icon-btn" @click="showDrawer = false">✖</button>
              </div>
              <ul class="drawer-list">
                <li v-for="(m, index) in markers" :key="m.id" @click="seekTo(m.startTime)" :style="{ borderLeft: `4px solid ${getMarkerColor(index)}` }">
                  <span class="d-label">{{ m.label }}</span>
                  <span class="d-time">{{ formatTime(m.startTime) }}</span>
                </li>
              </ul>
            </div>
          </transition>

          <transition name="fade-controls">
            <div class="video-controls-overlay" v-show="showControls || !isPlaying" @click.stop>

              <div class="progress-bar-container" @mousedown="startDrag" @mousemove="onDrag" @mouseup="endDrag" @mouseleave="endDrag">
                <div class="progress-track" ref="trackRef">
                  <div class="progress-bg"></div>

                  <div v-for="(m, index) in markers" :key="'track-'+m.id">
                    <div class="chapter-segment"
                         :style="{ left: `${(m.startTime / duration) * 100}%`, width: `${((m.endTime - m.startTime) / duration) * 100}%`, backgroundColor: getMarkerColor(index) }"
                         :title="`${m.label} (${formatTime(m.startTime)} - ${formatTime(m.endTime)})`">
                    </div>
                    <div class="chapter-dot"
                         :style="{ left: `${(m.startTime / duration) * 100}%`, backgroundColor: getMarkerColor(index) }">
                    </div>
                  </div>

                  <div class="progress-marker-in" v-if="draftIn !== null" :style="{ left: `${(draftIn / duration) * 100}%` }"></div>

                  <div class="playhead-thumb" :style="{ left: `${(currentTime / (duration || 1)) * 100}%` }"></div>
                </div>
              </div>

              <div class="controls-dashboard">
                <div class="left-controls">
                  <button class="control-icon-btn" @click="togglePlay" :title="isPlaying ? '暂停 (空格)' : '播放 (空格)'">
                    {{ isPlaying ? '⏸' : '▶' }}
                  </button>

                  <div class="volume-control-horizontal" @mouseenter="isVolumeHovered = true" @mouseleave="isVolumeHovered = false">
                    <button class="control-icon-btn volume-btn" @click="toggleMute" title="静音 (M)">
                      {{ isMuted || volume === 0 ? '🔇' : volume < 0.5 ? '🔉' : '🔊' }}
                    </button>
                    <div class="volume-slider-wrapper" :class="{ 'expanded': isVolumeHovered }">
                      <input type="range" class="volume-slider" min="0" max="1" step="0.05" v-model="volume" @input="onVolumeInput" />
                    </div>
                  </div>

                  <div class="time-display">
                    <span class="current-time">{{ formatTime(currentTime) }}</span>
                    <span class="total-time">/ {{ formatTime(duration) }}</span>
                  </div>
                </div>

                <div class="right-controls">

                  <div class="speed-menu-wrapper" @mouseenter="showSpeedMenu = true" @mouseleave="showSpeedMenu = false">
                    <button class="speed-text">{{ playbackRate.toFixed(1) }}x</button>
                    <transition name="fade-fast">
                      <div class="speed-dropdown" v-show="showSpeedMenu">
                        <div v-for="spd in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0]" :key="spd"
                             class="speed-option" :class="{'active': playbackRate === spd}"
                             @click="setSpeed(spd)">
                          {{ spd.toFixed(2) }}x
                        </div>
                      </div>
                    </transition>
                  </div>

                  <div class="action-buttons">
                    <button class="mark-btn in-btn" @click="setInPoint" title="快捷键: I">
                      <span class="hotkey">I</span> 入点
                    </button>
                    <button class="mark-btn out-btn" @click="setOutPoint" title="快捷键: O">
                      出点 <span class="hotkey">O</span>
                    </button>
                  </div>

                  <button class="control-icon-btn fullscreen-btn" @click="toggleFullscreen" title="全屏 (F)">
                    <div class="f-box-icon" :class="{'exit': isFullscreen}">
                      {{ isFullscreen ? '><' : 'F' }}
                    </div>
                  </button>
                </div>
              </div>
            </div>
          </transition>

          <transition name="zoom-in">
            <div class="fullscreen-draft-modal" v-if="draftIn !== null && draftOut !== null" @click.stop>
              <h3>保存片段</h3>
              <p class="modal-time">{{ formatTime(draftIn) }} ➔ {{ formatTime(draftOut) }}</p>
              <div class="modal-input-group">
                <input ref="draftInputRef" v-model="draftLabel" type="text" placeholder="输入片段描述 (回车确认)" @keyup.enter="addMarker" />
                <button class="confirm-btn" @click="addMarker" :disabled="!draftLabel.trim()">添加</button>
              </div>
            </div>
          </transition>
        </div>

        <div class="draft-section" v-show="!isFullscreen" :class="{ 'active': draftIn !== null }">
          <div class="draft-time-visual">
            <div class="point-box">
              <span class="label">入点 (IN)</span>
              <span class="value" :class="{'highlight': draftIn !== null}">{{ draftIn !== null ? formatTime(draftIn) : '--:--.--' }}</span>
            </div>
            <div class="arrow">➔</div>
            <div class="point-box">
              <span class="label">出点 (OUT)</span>
              <span class="value" :class="{'highlight': draftOut !== null}">{{ draftOut !== null ? formatTime(draftOut) : '等待打点...' }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="card list-card" v-show="!isFullscreen">
        <div class="list-header">
          <h3 class="list-title">📋 标记列表 ({{ markers.length }})</h3>
          <div class="save-actions">
            <transition name="fade">
              <span v-if="autoSaveEnabled" class="auto-save-indicator">
                {{ isAutoSaving ? '☁️ 正在同步...' : (unsavedChanges ? '☁️ 等待保存' : '☁️ 已自动保存') }}
              </span>
            </transition>
            <button class="save-btn" @click="saveMarkers" :disabled="markers.length === 0" :class="{'pulse': unsavedChanges && !autoSaveEnabled}">
              💾 {{ autoSaveEnabled ? '手动备份' : '保存 JSON' }}
            </button>
          </div>
        </div>

        <div class="list-body">
          <div v-if="markers.length === 0" class="empty-state">
            <div class="empty-icon">🏷️</div>
            <h4>暂无片段</h4><p>按 <kbd>I</kbd> 设入点，<kbd>O</kbd> 设出点</p>
          </div>

          <ul class="marker-list" v-else>
            <transition-group name="list">
              <li v-for="(m, index) in markers" :key="m.id" class="marker-item" :style="{ borderLeftColor: getMarkerColor(index) }">
                <div class="marker-info">
                  <div class="marker-header">
                    <span class="marker-index" :style="{ backgroundColor: getMarkerColor(index), color: '#fff' }">#{{ index + 1 }}</span>
                    <span class="marker-label">{{ m.label }}</span>
                  </div>
                  <span class="marker-time">{{ formatTime(m.startTime) }} ➔ {{ formatTime(m.endTime) }}</span>
                </div>
                <div class="marker-actions">
                  <button class="icon-btn play-btn" @click="seekTo(m.startTime)" title="播放">▶</button>
                  <button class="icon-btn delete-btn" @click="removeMarker(index)" title="删除">✖</button>
                </div>
              </li>
            </transition-group>
          </ul>
        </div>
      </div>
    </div>

    <div class="global-empty" v-else>
      <div class="empty-icon-large">🎬</div>
      <h2>欢迎使用智能打轴系统</h2>
      <p>请加载视频，体验全屏沉浸式标记与多色彩进度条。</p>
    </div>

    <transition name="toast"><div v-if="saveStatus" class="toast-message">{{ saveStatus }}</div></transition>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'; // <--- 新增 watch
// 核心引用
const videoPath = ref('');
const videoSrc = ref('');
const videoRef = ref<HTMLVideoElement | null>(null);
const playerContainerRef = ref<HTMLDivElement | null>(null);
const draftInputRef = ref<HTMLInputElement | null>(null);

// 播放器状态
const isPlaying = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const playbackRate = ref(1.0);
const volume = ref(1.0);
const isMuted = ref(false);
const isVolumeHovered = ref(false);

// UI控制状态
const isFullscreen = ref(false);
const showControls = ref(true);
const showDrawer = ref(false);
const showSpeedMenu = ref(false);
let hideControlsTimer: number | null = null;

// 色彩管理系统 (供进度条和列表使用)
const colorPalette = [
  '#34d399', '#60a5fa', '#f472b6', '#fbbf24',
  '#c084fc', '#f87171', '#2dd4bf', '#818cf8'
];
function getMarkerColor(index: number) {
  return colorPalette[index % colorPalette.length];
}

// OSD 提示系统
const osdVisible = ref(false);
const osdText = ref('');
let osdTimer: number | null = null;

function triggerOSD(msg: string) {
  osdText.value = msg; osdVisible.value = true;
  if (osdTimer) clearTimeout(osdTimer);
  osdTimer = window.setTimeout(() => { osdVisible.value = false; }, 1500);
}

// 打轴数据
const draftIn = ref<number | null>(null);
const draftOut = ref<number | null>(null);
const draftLabel = ref('');
interface Marker { id: string; startTime: number; endTime: number; label: string; }
const markers = ref<Marker[]>([]);
const saveStatus = ref('');
const unsavedChanges = ref(false);
// --- 新增：自动保存相关状态 ---
const autoSaveEnabled = ref(true); // 默认假定开启，稍后从后端读取真实配置
const isAutoSaving = ref(false);
let autoSaveTimer: number | null = null;



function formatTime(seconds: number): string {
  if (isNaN(seconds) || seconds === null) return '00:00.00';
  const m = Math.floor(seconds / 60).toString().padStart(2, '0');
  const s = Math.floor(seconds % 60).toString().padStart(2, '0');
  const ms = Math.floor((seconds % 1) * 100).toString().padStart(2, '0');
  return `${m}:${s}.${ms}`;
}


// 在选择视频 (selectVideo) 时，顺便向后端拉取最新的全局偏好设置
async function selectVideo() {
  const selected = await open({ multiple: false, filters: [{ name: 'Videos', extensions: ['mp4', 'mkv', 'mov', 'avi'] }] });
  if (selected && typeof selected === 'string') {
    videoPath.value = selected; videoSrc.value = convertFileSrc(selected);
    draftIn.value = null; draftOut.value = null; draftLabel.value = '';
    saveStatus.value = ''; unsavedChanges.value = false; playbackRate.value = 1.0;

    try {
      // 1. 拉取历史标记
      const historyMarkers = await invoke<Marker[]>('load_markers', { videoPath: selected });
      markers.value = historyMarkers || [];
      if (markers.value.length > 0) showToast(`✅ 恢复 ${markers.value.length} 个历史标记`);

      // 2. 拉取全局设置 (获取用户是否开启了自动保存)
      const settings = await invoke<any>('get_app_settings');
      autoSaveEnabled.value = settings.autoSaveEnabled;
    } catch (e) {
      markers.value = [];
    }
  }
}
// --- 🌟 隐形守护者：深度监听 markers 数组 ---
watch(markers, (newVal, oldVal) => {
  // 只要数组发生变化，立刻标记为未保存状态
  unsavedChanges.value = true;

  // 如果未开启自动保存，或者当前没有有效视频，则跳过
  if (!autoSaveEnabled.value || !videoPath.value) return;

  // 防抖机制 (Debounce)：用户可能在 1 秒内连续按键，我们等用户停手 800 毫秒后再真正写入硬盘
  isAutoSaving.value = true;
  if (autoSaveTimer) window.clearTimeout(autoSaveTimer);

  autoSaveTimer = window.setTimeout(async () => {
    try {
      await invoke<string>('save_markers', { videoPath: videoPath.value, markers: markers.value });
      unsavedChanges.value = false; // 标记为已安全落盘
    } catch (err) {
      showToast(`❌ 自动保存异常: ${err}`);
    } finally {
      isAutoSaving.value = false;
    }
  }, 800);
}, { deep: true }); // deep: true 极其重要，确保对象内部属性修改也能被监听到


// --- 鼠标隐藏逻辑 ---
function handleMouseMove() { showControls.value = true; resetHideTimer(); }
function handleMouseLeave() { if (isPlaying.value) showControls.value = false; }
function resetHideTimer() {
  if (hideControlsTimer) clearTimeout(hideControlsTimer);
  hideControlsTimer = window.setTimeout(() => { if (isPlaying.value) showControls.value = false; }, 2500);
}

// --- 视频时间与自定义进度条拖拽逻辑 ---
let isDragging = false;
function onTimeUpdate() { if (videoRef.value && !isDragging) currentTime.value = videoRef.value.currentTime; }
function onLoadedMetadata() { if (videoRef.value) { duration.value = videoRef.value.duration; setSpeed(1.0); } }
function togglePlay() {
  if (!videoRef.value) return;
  if (videoRef.value.paused) { videoRef.value.play(); resetHideTimer(); triggerOSD("▶ 播放"); }
  else { videoRef.value.pause(); showControls.value = true; triggerOSD("⏸ 暂停"); }
}
function seekTo(time: number) {
  if (videoRef.value) {
    const safeTime = Math.max(0, Math.min(time, duration.value));
    videoRef.value.currentTime = safeTime; currentTime.value = safeTime;
    resetHideTimer();
  }
}

function startDrag(e: MouseEvent) { isDragging = true; updateProgressByMouse(e); }
function onDrag(e: MouseEvent) { if (isDragging) updateProgressByMouse(e); }
function endDrag(e: MouseEvent) {
  if (isDragging) { updateProgressByMouse(e); isDragging = false; seekTo(currentTime.value); }
}
function updateProgressByMouse(e: MouseEvent) {
  const track = e.currentTarget as HTMLElement;
  if (!track) return;
  const rect = track.getBoundingClientRect();
  let percent = (e.clientX - rect.left) / rect.width;
  percent = Math.max(0, Math.min(1, percent));
  currentTime.value = percent * duration.value;
}

// --- 音量与倍速 ---
function setSpeed(spd: number) {
  playbackRate.value = spd;
  if (videoRef.value) videoRef.value.playbackRate = spd;
  showSpeedMenu.value = false;
  triggerOSD(`倍速: ${spd.toFixed(2)}x`);
}
function toggleMute() {
  if (!videoRef.value) return;
  videoRef.value.muted = !videoRef.value.muted;
  isMuted.value = videoRef.value.muted;
  if (!isMuted.value && volume.value === 0) { volume.value = 0.5; videoRef.value.volume = 0.5; }
  triggerOSD(isMuted.value ? "🔇 静音" : `🔊 音量: ${Math.round(volume.value * 100)}%`);
}
function onVolumeInput() {
  if (!videoRef.value) return;
  videoRef.value.volume = volume.value;
  videoRef.value.muted = volume.value === 0;
  isMuted.value = videoRef.value.muted;
}
function onNativeVolumeChange() {
  if (videoRef.value) { volume.value = videoRef.value.volume; isMuted.value = videoRef.value.muted; }
}

// --- 全屏逻辑 ---
function toggleFullscreen() {
  if (!document.fullscreenElement) { playerContainerRef.value?.requestFullscreen().catch(err => console.log(err)); }
  else { document.exitFullscreen(); }
}
function handleFullscreenChange() {
  isFullscreen.value = !!document.fullscreenElement;
  if (!isFullscreen.value) showDrawer.value = false;
}

// --- 打轴逻辑 ---
function setInPoint() {
  draftIn.value = currentTime.value;
  if (draftOut.value !== null && draftOut.value <= draftIn.value) draftOut.value = null;
  triggerOSD(`[ 入点 ] ${formatTime(draftIn.value)}`);
}

function setOutPoint() {
  if (draftIn.value === null) {
    const lastEnd = markers.value.length > 0 ? markers.value[markers.value.length - 1].endTime : 0;
    if (currentTime.value <= lastEnd) { triggerOSD("⚠️ 无法打出点：时间有误"); return; }
    draftIn.value = lastEnd;
    triggerOSD(`智能连轴: ${formatTime(draftIn.value)}`);
  }

  if (currentTime.value <= draftIn.value) { triggerOSD("⚠️ 出点必须大于入点"); return; }

  draftOut.value = currentTime.value;
  if (videoRef.value) videoRef.value.pause();
  showControls.value = true;
  nextTick(() => { draftInputRef.value?.focus(); });
}

function addMarker() {
  if (draftIn.value !== null && draftOut.value !== null && draftLabel.value.trim() !== '') {
    markers.value.push({ id: Date.now().toString(), startTime: draftIn.value, endTime: draftOut.value, label: draftLabel.value.trim() });
    markers.value.sort((a, b) => a.startTime - b.startTime);
    // unsavedChanges.value = true;
    draftIn.value = null; draftOut.value = null; draftLabel.value = '';
    triggerOSD("✅ 已添加");
    if (videoRef.value && !isPlaying.value) videoRef.value.play();
  }
}

function removeMarker(index: number) {
  markers.value.splice(index, 1);
  // unsavedChanges.value = true;
}

async function saveMarkers() {
  try {
    await invoke<string>('save_markers', { videoPath: videoPath.value, markers: markers.value });
    unsavedChanges.value = false; showToast(`✅ JSON 已安全保存`);
  } catch (err) { showToast(`❌ 保存失败`); }
}

let toastTimer: number | null = null;
function showToast(msg: string) {
  saveStatus.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => saveStatus.value = '', 3000);
}

// --- 全局快捷键加强版 ---
function handleKeyDown(e: KeyboardEvent) {
  const activeTag = document.activeElement?.tagName.toLowerCase();
  if (activeTag === 'input' || activeTag === 'textarea') return;
  if (!videoSrc.value) return;

  switch(e.key.toLowerCase()) {
    case 'i': case '[': e.preventDefault(); setInPoint(); break;
    case 'o': case ']': e.preventDefault(); setOutPoint(); break;
    case ' ': e.preventDefault(); togglePlay(); break;
    case 'f': e.preventDefault(); toggleFullscreen(); break;
    case 'm': e.preventDefault(); toggleMute(); break;
    case 'l': e.preventDefault(); showDrawer.value = !showDrawer.value; break;
    case 'z': e.preventDefault(); setSpeed(Math.max(0.1, playbackRate.value - 0.1)); break;
    case 'x': e.preventDefault(); setSpeed(Math.min(5.0, playbackRate.value + 0.1)); break;
    case 'c': e.preventDefault(); setSpeed(1.0); break;
    case 'arrowleft':
      e.preventDefault(); seekTo(currentTime.value - (e.shiftKey ? 1 : 5));
      showControls.value = true; resetHideTimer(); triggerOSD(e.shiftKey ? "⏪ -1s" : "⏪ -5s"); break;
    case 'arrowright':
      e.preventDefault(); seekTo(currentTime.value + (e.shiftKey ? 1 : 5));
      showControls.value = true; resetHideTimer(); triggerOSD(e.shiftKey ? "⏩ +1s" : "⏩ +5s"); break;

      // === 新增：键盘音量控制 (上/下箭头) ===
    case 'arrowup':
      e.preventDefault();
      volume.value = Math.min(1.0, volume.value + 0.05);
      onVolumeInput(); // 复用已有的音量更新函数
      triggerOSD(`🔊 音量: ${Math.round(volume.value * 100)}%`);
      showControls.value = true; resetHideTimer();
      break;
    case 'arrowdown':
      e.preventDefault();
      volume.value = Math.max(0.0, volume.value - 0.05);
      onVolumeInput();
      triggerOSD(volume.value === 0 ? "🔇 静音" : `🔉 音量: ${Math.round(volume.value * 100)}%`);
      showControls.value = true; resetHideTimer();
      break;
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  document.addEventListener('fullscreenchange', handleFullscreenChange);
});
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  document.removeEventListener('fullscreenchange', handleFullscreenChange);
  if (hideControlsTimer) clearTimeout(hideControlsTimer);
  if (osdTimer) clearTimeout(osdTimer);
});
</script>

<style scoped>
/* 基础容器 */
.view-container { display: flex; flex-direction: column; height: 100%; overflow: hidden; animation: fadeIn 0.3s ease; }
.view-header { display: flex; justify-content: space-between; align-items: center; flex-shrink: 0; margin-bottom: 1.5rem; }
.view-title { font-size: 1.75rem; font-weight: 700; color: #111827; margin: 0 0 0.25rem 0; }
.view-subtitle { color: #6b7280; margin: 0; font-size: 0.95rem; }

.header-actions { width: 400px; }
.input-with-btn { display: flex; gap: 0.5rem; }
.file-input { flex: 1; padding: 0.5rem 0.75rem; border: 1px solid #d1d5db; border-radius: 6px; background: #f9fafb; font-size: 0.85rem; outline: none; }
.primary-btn { padding: 0.5rem 1rem; background: #2563eb; color: white; border: none; border-radius: 6px; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 0.4rem; white-space: nowrap; transition: background 0.2s;}
.primary-btn:hover { background: #1d4ed8; }

.workspace-grid { display: grid; grid-template-columns: 1fr 320px; gap: 1.5rem; flex: 1; min-height: 0; }
.card { background: #ffffff; border-radius: 12px; border: 1px solid #e5e7eb; overflow: hidden; display: flex; flex-direction: column;}

/* === 播放台 (支持全屏) === */
.player-card { background-color: #0f172a; border-color: #1e293b; transition: all 0.3s; position: relative;}
.player-card.is-fullscreen { border-radius: 0; border: none; }

.video-wrapper {
  flex: 1; min-height: 0; background: #000; position: relative; display: flex; align-items: center; justify-content: center; overflow: hidden;
}
.video-wrapper.hide-cursor { cursor: none; }
.video-element { width: 100%; height: 100%; object-fit: contain; outline: none; }

.osd-message {
  position: absolute; top: 30px; left: 30px; background: rgba(0,0,0,0.6); color: white; padding: 10px 20px;
  border-radius: 8px; font-size: 1.5rem; font-weight: bold; pointer-events: none; backdrop-filter: blur(4px); z-index: 20;
}

.play-overlay { position: absolute; inset: 0; display: flex; justify-content: center; align-items: center; pointer-events: none; transition: all 0.2s ease-out; z-index: 5;}
.play-overlay.is-playing { opacity: 0; transform: scale(1.2); }
.play-icon-center { width: 70px; height: 70px; background: rgba(0,0,0,0.4); backdrop-filter: blur(8px); border-radius: 50%; display: flex; justify-content: center; align-items: center; font-size: 2.5rem; color: white; padding-left: 8px; border: 1px solid rgba(255,255,255,0.2); }

/* 全屏抽屉 */
.fullscreen-drawer {
  position: absolute; top: 0; right: 0; bottom: 0; width: 300px; background: rgba(15,23,42,0.85); backdrop-filter: blur(10px);
  border-left: 1px solid #334155; display: flex; flex-direction: column; z-index: 30;
}
.drawer-header { display: flex; justify-content: space-between; align-items: center; padding: 15px 20px; border-bottom: 1px solid #334155; color: white; }
.drawer-header h3 { margin: 0; font-size: 1.1rem; }
.drawer-list { list-style: none; padding: 0; margin: 0; overflow-y: auto; flex: 1; }
.drawer-list li { padding: 12px 20px; border-bottom: 1px solid #1e293b; cursor: pointer; display: flex; justify-content: space-between; color: #cbd5e1; transition: background 0.2s;}
.drawer-list li:hover { background: rgba(255,255,255,0.1); color: white; }
.d-label { font-weight: 600; }
.d-time { font-family: monospace; color: #94a3b8; }

/* === 悬浮式控制栏 === */
.video-controls-overlay {
  position: absolute; bottom: 0; left: 0; right: 0; padding: 30px 20px 15px 20px;
  background: linear-gradient(to top, rgba(0,0,0,0.85) 0%, rgba(0,0,0,0.5) 60%, transparent 100%);
  display: flex; flex-direction: column; z-index: 10;
}

/* 极致多色彩进度条 */
.progress-bar-container { width: 100%; height: 20px; display: flex; align-items: center; cursor: pointer; margin-bottom: 5px; position: relative; }
.progress-track { width: 100%; height: 4px; border-radius: 2px; position: relative; transition: height 0.1s; }
.progress-bar-container:hover .progress-track { height: 6px; }

/* 底层透明白边 */
.progress-bg { position: absolute; inset: 0; background: rgba(255,255,255,0.2); border-radius: 2px; }

/* 标记的章节色块 */
.chapter-segment { position: absolute; height: 100%; border-radius: 2px; opacity: 0.8; transition: opacity 0.2s; z-index: 1; }
.chapter-segment:hover { opacity: 1; cursor: crosshair; }

/* 起始圆点 */
.chapter-dot { position: absolute; width: 6px; height: 6px; border-radius: 50%; top: 50%; transform: translate(-50%, -50%); z-index: 2; border: 1px solid rgba(0,0,0,0.5); }

/* 草稿入点 */
.progress-marker-in { position: absolute; width: 4px; height: 140%; background: #fff; top: -20%; border-radius: 2px; z-index: 2; box-shadow: 0 0 4px rgba(0,0,0,0.8);}

/* 纯白播放游标，不再有覆盖颜色 */
.playhead-thumb { position: absolute; width: 14px; height: 14px; background: #fff; border-radius: 50%; top: 50%; transform: translate(-50%, -50%); z-index: 3; box-shadow: 0 0 4px rgba(0,0,0,0.6); pointer-events: none;}

.controls-dashboard { display: flex; justify-content: space-between; align-items: center; margin-top: 8px;}
.left-controls { display: flex; align-items: center; gap: 1rem; }
.right-controls { display: flex; align-items: center; gap: 0.75rem; }

.control-icon-btn { background: transparent; border: none; color: white; font-size: 1.4rem; cursor: pointer; transition: color 0.2s;}
.control-icon-btn:hover { color: #38bdf8; }

.volume-control-horizontal { display: flex; align-items: center; gap: 5px; }
.volume-slider-wrapper { width: 0; overflow: hidden; transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1); display: flex; align-items: center;}
.volume-slider-wrapper.expanded { width: 80px; }
.volume-slider { -webkit-appearance: none; width: 75px; height: 4px; background: rgba(255,255,255,0.3); border-radius: 2px; outline: none; margin: 0; cursor: pointer; }
.volume-slider::-webkit-slider-thumb { -webkit-appearance: none; width: 12px; height: 12px; border-radius: 50%; background: #fff; box-shadow: 0 0 2px rgba(0,0,0,0.5); }

.time-display { font-family: monospace; font-size: 1.1rem; text-shadow: 0 1px 2px rgba(0,0,0,0.8); margin-left: 10px;}
.current-time { color: #f8fafc; font-weight: 500; }
.total-time { color: #cbd5e1; margin-left: 4px; }

/* 悬浮倍速菜单 */
.speed-menu-wrapper { position: relative; }
.speed-text { background: rgba(255,255,255,0.1); color: white; border: 1px solid rgba(255,255,255,0.2); padding: 4px 10px; border-radius: 6px; font-size: 0.9rem; font-weight: bold; cursor: pointer; outline: none; backdrop-filter: blur(4px);}
.speed-dropdown { position: absolute; bottom: calc(100% + 10px); left: 50%; transform: translateX(-50%); background: rgba(15,23,42,0.9); border: 1px solid #334155; border-radius: 8px; padding: 4px 0; display: flex; flex-direction: column; backdrop-filter: blur(8px); z-index: 20;}
.speed-option { padding: 8px 16px; color: #cbd5e1; font-size: 0.85rem; font-weight: bold; cursor: pointer; transition: background 0.2s; white-space: nowrap;}
.speed-option:hover { background: rgba(255,255,255,0.1); color: white; }
.speed-option.active { color: #38bdf8; }

.action-buttons { display: flex; gap: 0.75rem; margin: 0 10px;}
.mark-btn { background: rgba(255,255,255,0.15); color: #f8fafc; border: 1px solid rgba(255,255,255,0.3); padding: 0.4rem 0.8rem; border-radius: 6px; font-weight: 500; cursor: pointer; display: flex; align-items: center; gap: 0.5rem; transition: all 0.2s; backdrop-filter: blur(4px);}
.mark-btn:hover { background: rgba(255,255,255,0.25); border-color: #fff; }
.hotkey { background: rgba(0,0,0,0.4); padding: 2px 6px; border-radius: 4px; font-size: 0.75rem; font-family: monospace; color: #e2e8f0; }

/* 定制化方框 F 全屏图标 */
.f-box-icon { width: 22px; height: 22px; border: 2px solid white; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 900; font-family: 'Segoe UI', sans-serif; transition: all 0.2s;}
.fullscreen-btn:hover .f-box-icon { border-color: #38bdf8; color: #38bdf8;}

/* 弹性自适应全屏弹窗 */
.fullscreen-draft-modal {
  position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
  width: 90%; max-width: 450px; /* 完美解决超出和换行 */
  background: rgba(15,23,42,0.85); border: 1px solid #38bdf8; padding: 2rem; border-radius: 16px;
  backdrop-filter: blur(12px); box-shadow: 0 25px 50px -12px rgba(0,0,0,0.6); text-align: center; z-index: 50;
}
.fullscreen-draft-modal h3 { color: white; margin: 0 0 0.5rem 0; font-size: 1.5rem;}
.modal-time { color: #94a3b8; font-family: monospace; font-size: 1.3rem; margin-bottom: 1.5rem; }
.modal-input-group { display: flex; gap: 0.75rem; width: 100%;}
.modal-input-group input { flex: 1; min-width: 0; padding: 0.8rem 1rem; border-radius: 8px; border: 1px solid #475569; background: #1e293b; color: white; font-size: 1rem; outline: none;}
.modal-input-group input:focus { border-color: #38bdf8; }
.modal-input-group button { flex-shrink: 0; padding: 0 1.5rem; background: #38bdf8; border: none; border-radius: 8px; font-weight: bold; font-size: 1rem; cursor: pointer; color: #0f172a; white-space: nowrap;}

/* 窗口模式底部的指示区 (精简) */
.draft-section { flex-shrink: 0; padding: 1rem; background: #0f172a; border-top: 1px solid #1e293b; display: flex; justify-content: center; height: 60px;}
.draft-time-visual { display: flex; align-items: center; gap: 1rem; }
.point-box { background: transparent; border-radius: 8px; text-align: center; }
.point-box .label { font-size: 0.75rem; color: #64748b; margin-right: 8px;}
.point-box .value { font-family: monospace; font-size: 1.1rem; color: #475569; }
.point-box .value.highlight { color: #f8fafc; font-weight: bold; }
.arrow { color: #475569; font-size: 1rem; }

/* 右侧列表卡片 */
.list-card { background: #ffffff; }
.list-header { flex-shrink: 0; padding: 1.25rem 1.5rem; border-bottom: 1px solid #e5e7eb; display: flex; justify-content: space-between; align-items: center; background: #f9fafb;}
.list-title { margin: 0; font-size: 1.05rem; font-weight: 700; color: #111827; }
.save-btn { padding: 0.4rem 0.8rem; background: white; border: 1px solid #d1d5db; border-radius: 6px; cursor: pointer; font-weight: 600; color: #374151;}
.save-btn.pulse { border-color: #3b82f6; color: #2563eb; background: #eff6ff; animation: subtle-pulse 2s infinite; }
.list-body { flex: 1; overflow-y: auto; padding: 1rem; }
.empty-state { text-align: center; color: #6b7280; margin-top: 2rem;}
.marker-list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.75rem; }

/* 标记列表的颜色联动 */
.marker-item { display: flex; justify-content: space-between; padding: 0.85rem; background: #ffffff; border: 1px solid #e5e7eb; border-radius: 8px; border-left-width: 4px;}
.marker-info { display: flex; flex-direction: column; gap: 6px; }
.marker-index { color: white; font-size: 0.7rem; padding: 2px 6px; border-radius: 4px; font-weight: bold;}
.marker-label { font-weight: 600; color: #1f2937; margin-left: 6px;}
.marker-time { font-family: monospace; font-size: 0.85rem; color: #6b7280; }
.marker-actions { display: flex; gap: 0.25rem; align-items: center;}
.icon-btn { background: transparent; border: none; cursor: pointer; color: #9ca3af; font-size: 1.1rem;}
.icon-btn:hover { color: white; }
.play-btn { color: #10b981; }
.delete-btn { color: #ef4444; }

.global-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px dashed #d1d5db; color: #6b7280; border-radius: 12px;}
.toast-message { position: fixed; bottom: 30px; left: 50%; transform: translateX(-50%); background: #1f2937; color: white; padding: 0.75rem 1.5rem; border-radius: 30px; z-index: 9999; }

@keyframes subtle-pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.7; } }
.fade-controls-enter-active, .fade-controls-leave-active { transition: opacity 0.3s ease; }
.fade-controls-enter-from, .fade-controls-leave-to { opacity: 0; }
.zoom-in-enter-active, .zoom-in-leave-active { transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1); }
.zoom-in-enter-from, .zoom-in-leave-to { opacity: 0; transform: translate(-50%, -50%) scale(0.9); }
.list-enter-active, .list-leave-active { transition: all 0.3s ease; }
.list-enter-from, .list-leave-to { opacity: 0; transform: translateX(20px); }
.slide-right-enter-active, .slide-right-leave-active { transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.slide-right-enter-from, .slide-right-leave-to { opacity: 0; transform: translateX(100%); }
.fade-fast-enter-active, .fade-fast-leave-active { transition: opacity 0.15s; }
.fade-fast-enter-from, .fade-fast-leave-to { opacity: 0; transform: translateY(5px); }
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }


/* ... */
.save-actions { display: flex; align-items: center; gap: 10px; }
.auto-save-indicator {
  font-size: 0.75rem;
  color: #6b7280;
  font-weight: 500;
  background: #f3f4f6;
  padding: 4px 8px;
  border-radius: 4px;
  animation: fadeIn 0.3s ease;
}
/* ... */
</style>