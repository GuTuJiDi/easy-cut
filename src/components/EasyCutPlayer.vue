<template>
  <!-- 播放器外壳：也是全屏触发的容器 -->
  <div class="player-card card" ref="playerContainerRef" @mousemove="resetHideTimer" @mouseleave="handleMouseLeave">

    <div class="video-wrapper" :class="{ 'hide-cursor': !showControls && isPlaying }" @click="togglePlay">
      <video
          ref="videoRef"
          class="video-element"
          :src="src"
          @timeupdate="handleInternalTimeUpdate"
          @loadedmetadata="handleLoadedMetadata"
          @play="isPlaying = true"
          @pause="isPlaying = false"
          @ended="isPlaying = false"
      ></video>

      <transition name="fade">
        <div class="osd-message" v-if="osdVisible">{{ osdText }}</div>
      </transition>

      <div class="play-overlay" :class="{ 'is-playing': isPlaying }">
        <div class="play-icon-center">▶</div>
      </div>
    </div>

    <!-- 🌟 核心：为父组件预留的插槽，保证父组件的打轴输入框在全屏时也能显示 -->
    <slot></slot>
    <!-- 🌟 恢复：全屏状态下的右侧标记列表 (Drawer) -->
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
    <!-- 悬浮控制条 -->
    <transition name="fade-controls">
      <div class="video-controls-overlay" v-show="showControls || !isPlaying" @click.stop>

        <div class="progress-bar-container" @mousedown="startDrag" @mousemove="onDrag" @mouseup="endDrag" @mouseleave="endDrag">
          <div class="progress-track" ref="trackRef">
            <div class="progress-bg"></div>

            <!-- 彩色章节块 -->
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
            <div class="playhead-thumb" :style="{ left: `${(internalCurrentTime / (duration || 1)) * 100}%` }"></div>
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
                <input type="range" class="volume-slider" min="0" max="1" step="0.05" :value="volume" @input="handleVolumeInput" />
              </div>
            </div>

            <div class="time-display">
              <span class="current-time">{{ formatTime(internalCurrentTime) }}</span>
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

            <button class="control-icon-btn fullscreen-btn" @click="toggleFullscreen" title="全屏 (F)">
              <div class="f-box-icon" :class="{'exit': isFullscreen}">
                {{ isFullscreen ? '><' : 'F' }}
              </div>
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

// ==========================================
// 1. 类型约束与 Props/Emits
// ==========================================
interface Marker { id: string; startTime: number; endTime: number; label: string; }

const props = defineProps<{
  src: string;
  markers: Marker[];
  draftIn: number | null;
  draftOut: number | null;
}>();

const emit = defineEmits<{
  (e: 'timeupdate', time: number): void;
  (e: 'durationchange', duration: number): void;
  (e: 'fullscreenchange', isFullscreen: boolean): void;
}>();

// ==========================================
// 2. 内部状态
// ==========================================
const videoRef = ref<HTMLVideoElement | null>(null);
const playerContainerRef = ref<HTMLDivElement | null>(null);
const trackRef = ref<HTMLDivElement | null>(null);

const isPlaying = ref(false);
const internalCurrentTime = ref(0);
const duration = ref(0);
const playbackRate = ref(1.0);
const volume = ref(1.0);
const isMuted = ref(false);
const isFullscreen = ref(false);
const isVolumeHovered = ref(false);

const showControls = ref(true);
const showSpeedMenu = ref(false);
const osdVisible = ref(false);
const osdText = ref('');
let hideControlsTimer: number | null = null;
let osdTimer: number | null = null;
let isDragging = false;

const showDrawer = ref(false); // 控制全屏列表开关
function toggleDrawer() {
  if (isFullscreen.value) {
    showDrawer.value = !showDrawer.value;
  } else {
    triggerOSD("⚠️ 请先进入全屏模式 (F)");
  }
}
// ==========================================
// 3. 核心工具函数
// ==========================================
function formatTime(seconds: number): string {
  if (isNaN(seconds) || seconds === null) return '00:00.00';
  const m = Math.floor(seconds / 60).toString().padStart(2, '0');
  const s = Math.floor(seconds % 60).toString().padStart(2, '0');
  const ms = Math.floor((seconds % 1) * 100).toString().padStart(2, '0');
  return `${m}:${s}.${ms}`;
}

const colorPalette = ['#34d399', '#60a5fa', '#f472b6', '#fbbf24', '#c084fc', '#f87171', '#2dd4bf', '#818cf8'];
function getMarkerColor(index: number) {
  return colorPalette[index % colorPalette.length];
}

function triggerOSD(msg: string) {
  osdText.value = msg; osdVisible.value = true;
  if (osdTimer) clearTimeout(osdTimer);
  osdTimer = window.setTimeout(() => { osdVisible.value = false; }, 1500);
}

// ==========================================
// 4. 视频核心逻辑
// ==========================================
function handleLoadedMetadata() {
  if (videoRef.value) {
    duration.value = videoRef.value.duration;
    emit('durationchange', duration.value);
  }
}

function handleInternalTimeUpdate() {
  if (videoRef.value && !isDragging) {
    internalCurrentTime.value = videoRef.value.currentTime;
    emit('timeupdate', internalCurrentTime.value); // 同步给父组件打轴用
  }
}

function togglePlay() {
  if (!videoRef.value) return;
  if (videoRef.value.paused) { videoRef.value.play(); resetHideTimer(); triggerOSD("▶ 播放"); }
  else { videoRef.value.pause(); showControls.value = true; triggerOSD("⏸ 暂停"); }
}

function seekTo(time: number) {
  if (videoRef.value) {
    const safeTime = Math.max(0, Math.min(time, duration.value));
    videoRef.value.currentTime = safeTime;
    internalCurrentTime.value = safeTime;
    resetHideTimer();
  }
}

function setSpeed(spd: number) {
  playbackRate.value = spd;
  if (videoRef.value) videoRef.value.playbackRate = spd;
  showSpeedMenu.value = false;
  triggerOSD(`倍速: ${spd.toFixed(2)}x`);
}

// ==========================================
// 5. 音量与全屏 (供父组件键盘调用的严格接口)
// ==========================================
function setVolume(v: number) {
  volume.value = Math.max(0, Math.min(1, v));
  if (videoRef.value) {
    videoRef.value.volume = volume.value;
    videoRef.value.muted = volume.value === 0;
    isMuted.value = videoRef.value.muted;
  }
}

function toggleMute() {
  if (!videoRef.value) return;
  videoRef.value.muted = !videoRef.value.muted;
  isMuted.value = videoRef.value.muted;
  if (!isMuted.value && volume.value === 0) setVolume(0.5);
  triggerOSD(isMuted.value ? "🔇 静音" : `🔊 音量: ${Math.round(volume.value * 100)}%`);
}

function handleVolumeInput(e: Event) {
  const target = e.target as HTMLInputElement;
  setVolume(parseFloat(target.value));
}

function toggleFullscreen() {
  if (!document.fullscreenElement) { playerContainerRef.value?.requestFullscreen().catch(err => console.log(err)); }
  else { document.exitFullscreen(); }
}

function handleFullscreenChange() {
  isFullscreen.value = !!document.fullscreenElement;
  emit('fullscreenchange', isFullscreen.value);
}

// ==========================================
// 6. UI 控制与拖拽
// ==========================================
function resetHideTimer() {
  showControls.value = true;
  if (hideControlsTimer) clearTimeout(hideControlsTimer);
  hideControlsTimer = window.setTimeout(() => { if (isPlaying.value) showControls.value = false; }, 2500);
}
function handleMouseLeave() { if (isPlaying.value) showControls.value = false; }

function startDrag(e: MouseEvent) { isDragging = true; updateProgressByMouse(e); }
function onDrag(e: MouseEvent) { if (isDragging) updateProgressByMouse(e); }
function endDrag(e: MouseEvent) {
  if (isDragging) { updateProgressByMouse(e); isDragging = false; seekTo(internalCurrentTime.value); }
}
function updateProgressByMouse(e: MouseEvent) {
  const track = e.currentTarget as HTMLElement;
  if (!track) return;
  const rect = track.getBoundingClientRect();
  let percent = (e.clientX - rect.left) / rect.width;
  percent = Math.max(0, Math.min(1, percent));
  internalCurrentTime.value = percent * duration.value;
}

// 生命周期
onMounted(() => document.addEventListener('fullscreenchange', handleFullscreenChange));
onUnmounted(() => {
  document.removeEventListener('fullscreenchange', handleFullscreenChange);
  if (hideControlsTimer) clearTimeout(hideControlsTimer);
  if (osdTimer) clearTimeout(osdTimer);
});

// 🌟 核心防报错设计：将严格类型的方法暴露给父组件，父组件无需碰触原生 DOM
defineExpose({
  seekTo,
  togglePlay,
  toggleFullscreen,
  toggleMute,
  setSpeed,
  setVolume,
  getVolume: () => volume.value,
  getPlaybackRate: () => playbackRate.value,
  triggerOSD,
  toggleDrawer
});
</script>

<style scoped>
/* 包含播放器特有的所有样式 */
.player-card { background-color: #0f172a; border-color: #1e293b; transition: all 0.3s; position: relative; display: flex; flex-direction: column; width: 100%; height: 100%;}
.video-wrapper { flex: 1; min-height: 0; background: transparent; position: relative; display: flex; align-items: center; justify-content: center; overflow: hidden; }
.video-wrapper.hide-cursor { cursor: none; }
.video-element { width: 100%; height: 100%; object-fit: contain; outline: none; }
.osd-message { position: absolute; top: 30px; left: 30px; background: rgba(0,0,0,0.6); color: white; padding: 10px 20px; border-radius: 8px; font-size: 1.5rem; font-weight: bold; pointer-events: none; backdrop-filter: blur(4px); z-index: 20; }
.play-overlay { position: absolute; inset: 0; display: flex; justify-content: center; align-items: center; pointer-events: none; transition: all 0.2s ease-out; z-index: 5;}
.play-overlay.is-playing { opacity: 0; transform: scale(1.2); }
.play-icon-center { width: 70px; height: 70px; background: rgba(0,0,0,0.4); backdrop-filter: blur(8px); border-radius: 50%; display: flex; justify-content: center; align-items: center; font-size: 2.5rem; color: white; padding-left: 8px; border: 1px solid rgba(255,255,255,0.2); }
.video-controls-overlay { position: absolute; bottom: 0; left: 0; right: 0; padding: 30px 20px 15px 20px; background: linear-gradient(to top, rgba(0,0,0,0.85) 0%, rgba(0,0,0,0.5) 60%, transparent 100%); display: flex; flex-direction: column; z-index: 10; }
.progress-bar-container { width: 100%; height: 20px; display: flex; align-items: center; cursor: pointer; margin-bottom: 5px; position: relative; }
.progress-track { width: 100%; height: 4px; border-radius: 2px; position: relative; transition: height 0.1s; }
.progress-bar-container:hover .progress-track { height: 6px; }
.progress-bg { position: absolute; inset: 0; background: rgba(255,255,255,0.2); border-radius: 2px; }
.chapter-segment { position: absolute; height: 100%; border-radius: 2px; opacity: 0.8; transition: opacity 0.2s; z-index: 1; }
.chapter-segment:hover { opacity: 1; cursor: crosshair; }
.chapter-dot { position: absolute; width: 6px; height: 6px; border-radius: 50%; top: 50%; transform: translate(-50%, -50%); z-index: 2; border: 1px solid rgba(0,0,0,0.5); }
.progress-marker-in { position: absolute; width: 4px; height: 140%; background: #fff; top: -20%; border-radius: 2px; z-index: 2; box-shadow: 0 0 4px rgba(0,0,0,0.8);}
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
.speed-menu-wrapper { position: relative; }
.speed-text { background: rgba(255,255,255,0.1); color: white; border: 1px solid rgba(255,255,255,0.2); padding: 4px 10px; border-radius: 6px; font-size: 0.9rem; font-weight: bold; cursor: pointer; outline: none; backdrop-filter: blur(4px);}
.speed-dropdown { position: absolute; bottom: calc(100% + 10px); left: 50%; transform: translateX(-50%); background: rgba(15,23,42,0.9); border: 1px solid #334155; border-radius: 8px; padding: 4px 0; display: flex; flex-direction: column; backdrop-filter: blur(8px); z-index: 20;}
.speed-option { padding: 8px 16px; color: #cbd5e1; font-size: 0.85rem; font-weight: bold; cursor: pointer; transition: background 0.2s; white-space: nowrap;}
.speed-option:hover { background: rgba(255,255,255,0.1); color: white; }
.speed-option.active { color: #38bdf8; }
.f-box-icon { width: 22px; height: 22px; border: 2px solid white; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 900; font-family: 'Segoe UI', sans-serif; transition: all 0.2s;}
.fullscreen-btn:hover .f-box-icon { border-color: #38bdf8; color: #38bdf8;}
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.fade-controls-enter-active, .fade-controls-leave-active { transition: opacity 0.3s ease; }
.fade-controls-enter-from, .fade-controls-leave-to { opacity: 0; }
.fade-fast-enter-active, .fade-fast-leave-active { transition: opacity 0.15s; }
.fade-fast-enter-from, .fade-fast-leave-to { opacity: 0; transform: translateY(5px); }

/* 全屏抽屉列表 */
.fullscreen-drawer {
  position: absolute; top: 0; right: 0; bottom: 0; width: 300px; background: rgba(15,23,42,0.85); backdrop-filter: blur(10px);
  border-left: 1px solid #334155; display: flex; flex-direction: column; z-index: 30;
}
.drawer-header { display: flex; justify-content: space-between; align-items: center; padding: 15px 20px; border-bottom: 1px solid #334155; color: white; }
.drawer-header h3 { margin: 0; font-size: 1.1rem; }
.drawer-list { list-style: none; padding: 0; margin: 0; overflow-y: auto; flex: 1; }
.drawer-list li { padding: 12px 20px; border-bottom: 1px solid #1e293b; cursor: pointer; display: flex; justify-content: space-between; color: #cbd5e1; transition: background 0.2s;}
.drawer-list li:hover { background: rgba(255,255,255,0.1); color: white; }
.d-label { font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 60%;}
.d-time { font-family: monospace; color: #94a3b8; }
.slide-right-enter-active, .slide-right-leave-active { transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.slide-right-enter-from, .slide-right-leave-to { opacity: 0; transform: translateX(100%); }
</style>