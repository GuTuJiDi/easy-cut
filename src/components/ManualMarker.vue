<template>
  <div class="manual-marker-wrapper" v-if="videoPath">
    <header class="view-header" style="margin-bottom: 2rem;">
      <div class="header-titles">
        <h1 class="view-title">智能打轴标记</h1>
        <p class="view-subtitle">利用快捷键极速标记：[I]入点, [O]出点, [F]全屏, [↑/↓]音量, [Z/X/C]倍速, [L]标记列表</p>
      </div>
      <div class="header-actions">
        <div class="active-file-badge">
          <span class="file-icon">🎬</span>
          <span class="file-name" :title="videoPath">{{ videoFileName }}</span>
          <div class="badge-actions">
            <button class="icon-btn" @click="selectVideo" title="更换视频">🔄</button>
            <button class="icon-btn" @click="closeProject" title="关闭视频">✖</button>
          </div>
        </div>
      </div>
    </header>

    <div class="marker-container" :class="{'is-fullscreen': isPlayerFullscreen}">
      <EasyCutPlayer
          ref="playerRef"
          :src="videoSrc"
          :markers="activeMarkers"
          :draftIn="draftIn"
          :draftOut="draftOut"
          @timeupdate="currentVideoTime = $event"
          @fullscreenchange="isPlayerFullscreen = $event"
          style="flex: 1;"
      >
        <Teleport :to="isPlayerFullscreen ? '.player-card' : 'body'" v-if="isMounted">
          <transition name="slide-up-fade">
            <div
                class="fullscreen-draft-modal"
                v-if="showDraftModal && draftIn !== null && draftOut !== null"
                @click.stop
                :style="{ transform: `translate(calc(-50% + ${modalOffsetX}px), ${modalOffsetY}px)` }"
            >
              <div class="modal-drag-handle" @mousedown="startDragModal">
                <h3 class="modal-title">{{ selectedMarkerId ? '编辑片段描述' : '保存片段' }} <span class="drag-hint">(可拖拽)</span></h3>
                <button class="modal-close-btn" @click="cancelDraft" title="收起弹窗 (ESC)">✖</button>
              </div>

              <div class="modal-time-visual">
                <div class="point-box">
                  <span class="label">入点 (IN)</span>
                  <div class="time-editor-wrapper">
                    <button class="nudge-btn" @click="nudgeIn(-0.1)" title="微调后退 0.1秒">-</button>
                    <input type="text" class="time-input" :value="draftIn !== null ? formatTime(draftIn) : ''" @change="updateDraftIn" @keyup.enter="($event.target as HTMLInputElement).blur()"/>
                    <button class="nudge-btn" @click="nudgeIn(0.1)" title="微调前进 0.1秒">+</button>
                  </div>
                </div>
                <div class="arrow">➔</div>
                <div class="point-box">
                  <span class="label">出点 (OUT)</span>
                  <div class="time-editor-wrapper">
                    <button class="nudge-btn" @click="nudgeOut(-0.1)" title="微调后退 0.1秒">-</button>
                    <input type="text" class="time-input" :value="draftOut !== null ? formatTime(draftOut) : ''" @change="updateDraftOut" @keyup.enter="($event.target as HTMLInputElement).blur()"/>
                    <button class="nudge-btn" @click="nudgeOut(0.1)" title="微调前进 0.1秒">+</button>
                  </div>
                </div>
              </div>

              <div class="modal-input-group">
                <input ref="draftInputRef" v-model="draftLabel" type="text" placeholder="输入片段描述 (回车确认, ESC收起)" @keyup.enter="saveMarker" @keyup.esc="onInputEsc" />
                <button class="confirm-btn" :class="{'is-editing': selectedMarkerId}" @click="saveMarker" :disabled="!draftLabel.trim()">
                  {{ selectedMarkerId ? '保存修改' : '确认添加' }}
                </button>
              </div>
            </div>
          </transition>
        </Teleport>
      </EasyCutPlayer>

      <div class="card list-card" v-show="!isPlayerFullscreen">
        <div class="list-header">
          <h3 class="list-title">📋 标记 ({{ activeMarkers.length }})</h3>
          <div class="save-actions">
            <transition name="fade">
              <span v-if="settingsStore.autoSave" class="auto-save-indicator" :class="{'saving': isAutoSaving}">
                {{ isAutoSaving ? '☁️ 同步中...' : (unsavedChanges ? '☁️ 待同步' : '☁️ 已保存') }}
              </span>
            </transition>

            <div class="action-btn-group">
              <button class="icon-btn danger-icon" @click="clearAllMarkers" v-if="activeMarkers.length > 0" :disabled="isExporting" title="清空并移入回收站">🗑️</button>
              <button v-if="!settingsStore.autoSave" class="icon-btn save-icon" @click="saveMarkers" :disabled="activeMarkers.length===0 || isExporting" title="手动保存" :class="{'pulse': unsavedChanges}">💾</button>
            </div>
          </div>
        </div>

        <div class="list-body">
          <div v-if="activeMarkers.length === 0" class="empty-state">
            <span class="empty-icon" style="font-size: 2.5rem; display: block;">🏷️</span>
            <h4>暂无片段</h4><p class="hint">按 <kbd>I</kbd> 设入点，<kbd>O</kbd> 设出点</p>
          </div>

          <ul class="marker-list" v-else>
            <transition-group name="list">
              <li
                  v-for="(m, index) in activeMarkers"
                  :key="m.id"
                  class="marker-item"
                  :class="{ 'is-selected': selectedMarkerId === m.id }"
                  :style="{ borderLeftColor: getMarkerColor(index) }"
                  @click="selectMarker(m)"
              >
                <div class="marker-info">
                  <div class="marker-header">
                    <span class="marker-index" :style="{ backgroundColor: getMarkerColor(index), color: '#fff' }">#{{ index + 1 }}</span>
                    <span class="marker-label" :title="m.label">{{ m.label }}</span>
                  </div>
                  <span class="marker-time">{{ formatTime(m.startTime) }} ➔ {{ formatTime(m.endTime) }}</span>
                </div>
                <div class="marker-actions">
                  <button class="icon-btn play-btn" @click.stop="playSegment(m)" title="从入点强制播放">▶</button>
                  <button class="icon-btn split-btn" @click.stop="handleSubTask(m, '/split')" title="抽出此片段并去分割">✂️</button>
                  <button class="icon-btn marker-btn" @click.stop="handleSubTask(m, '/marker')" title="抽出此片段并去打轴">🏷️</button>
                  <button class="icon-btn delete-btn" @click.stop="removeMarker(m.id)" title="删除">✖</button>
                </div>
              </li>
            </transition-group>
          </ul>
        </div>

        <div class="draft-section">
          <div class="draft-time-visual">
            <div class="point-box">
              <span class="label">入点 (IN)</span>
              <div class="time-editor-wrapper">
                <button class="nudge-btn" @click="nudgeIn(-0.1)" title="微调后退 0.1秒">-</button>
                <input type="text" class="time-input" :class="{'highlight': draftIn !== null}" :value="draftIn !== null ? formatTime(draftIn) : ''" placeholder="--:--.--" @change="updateDraftIn" @keyup.enter="($event.target as HTMLInputElement).blur()"/>
                <button class="nudge-btn" @click="nudgeIn(0.1)" title="微调前进 0.1秒">+</button>
              </div>
            </div>

            <div class="arrow">➔</div>

            <div class="point-box">
              <span class="label">出点 (OUT)</span>
              <div class="time-editor-wrapper">
                <button class="nudge-btn" @click="nudgeOut(-0.1)" title="微调后退 0.1秒">-</button>
                <input type="text" class="time-input" :class="{'highlight': draftOut !== null}" :value="draftOut !== null ? formatTime(draftOut) : ''" placeholder="--:--.--" @change="updateDraftOut" @keyup.enter="($event.target as HTMLInputElement).blur()"/>
                <button class="nudge-btn" @click="nudgeOut(0.1)" title="微调前进 0.1秒">+</button>
              </div>
            </div>
          </div>

          <div class="draft-quick-actions" v-if="draftOut !== null && !showDraftModal">
            <button class="quick-btn ghost" @click="cancelDraft" title="退出或取消">✖ 取消</button>
            <button class="quick-btn outline" @click="showDraftModal = true" title="打开弹窗编辑文字描述">
              {{ selectedMarkerId ? '✏️ 编辑描述' : '📝 添加描述' }}
            </button>
            <button class="quick-btn primary" @click="saveMarker">
              {{ selectedMarkerId ? '💾 保存修改' : '✅ 确认打段' }}
            </button>
          </div>
        </div>

        <div class="list-footer" v-if="activeMarkers.length > 0">
          <button class="primary-btn export-btn" @click="exportMarkers" :disabled="isExporting">
            <span v-if="isExporting" class="spinner">⚙️</span>
            {{ isExporting ? '正在极速切片中...' : '🚀 一键导出全部片段' }}
          </button>
        </div>
      </div>
    </div>
  </div>

  <div class="view-container global-empty" v-else>
    <div class="empty-icon-large" style="font-size: 4rem;">🎬</div>
    <h2>欢迎使用智能打轴系统</h2>
    <p>请加载视频，体验全屏沉浸式标记与多色彩进度条。</p>
    <button class="primary-btn" style="margin-top: 1rem;" @click="selectVideo">📁 选择本地视频</button>
  </div>

  <transition name="toast"><div v-if="saveStatus" class="toast-message">{{ saveStatus }}</div></transition>

  <ExportResultModal
      v-model="showExportModal"
      :logs="exportLogs"
      :exportDir="finalExportDir"
      :costTime="exportCostTime"
  />

  <transition name="modal-fade">
    <div class="subtask-overlay" v-if="showSubTaskModal" @click.self="showSubTaskModal = false">
      <div class="subtask-modal-content">
        <div class="modal-header">
          <div class="header-left">
            <span class="success-icon">🔄</span>
            <div class="header-text">
              <h2>独立片段流复制提取</h2>
              <p class="subtitle">工作流跳转前置处理</p>
            </div>
          </div>
          <button class="icon-btn close-btn" @click="showSubTaskModal = false">✖</button>
        </div>
        <div class="modal-body">
          <p style="margin-top:0; color: #334155;">即将前往处理片段: <strong style="color:#2563eb">{{ pendingSubTaskMarker?.label }}</strong></p>
          <div class="info-box">
            <span style="color:#059669; font-weight:bold;">⚡ 智能探针与缓存判定已就绪</span><br/>
            系统将优先检测该片段是否已导出。若文件已存在，将秒速直达；若未提取，则自动调用并发引擎无损切片并跳转。
          </div>
        </div>
        <div class="modal-footer">
          <button class="secondary-btn" @click="showSubTaskModal = false">取消</button>
          <button class="primary-btn" @click="executeSubTask" :disabled="isExtracting">
            <span v-if="isExtracting" class="spinner" style="margin-right:6px;"></span>
            {{ isExtracting ? '无损提取中...' : '🚀 提取并跳转' }}
          </button>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { ref, shallowRef, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useAuthStore } from '../stores/auth';
import EasyCutPlayer from './EasyCutPlayer.vue';
import ExportResultModal from './ExportResultModal.vue';
import { useSettingsStore } from '../stores/settings';

interface Marker { id: string; startTime: number; endTime: number; label: string; is_deleted: boolean; }
interface ProjectMeta { project_id: string; project_name: string; created_at: number; linked_file_hash: string | null; linked_file_name: string | null; source_type: string; [key: string]: any; }
interface EasyCutProject { version: string; meta: ProjectMeta; markers: Marker[]; }

const settingsStore = useSettingsStore();
const authStore = useAuthStore();
const router = useRouter();
const isExporting = ref(false);
const route = useRoute();
const playerRef = ref<InstanceType<typeof EasyCutPlayer> | null>(null);
const isMounted = ref(false);

const videoPath = ref('');
const videoSrc = ref('');
const currentVideoTime = ref(0);
const videoDurationSec = ref(0);
const isPlayerFullscreen = ref(false);

const draftIn = ref<number | null>(null);
const draftOut = ref<number | null>(null);
const draftLabel = ref('');
const draftInputRef = ref<HTMLInputElement | null>(null);

const selectedMarkerId = ref<string | null>(null);
const markers = ref<Marker[]>([]);

// ==========================================
// 🌟 核心升级：线性游标撤销重做架构 (Undo / Redo Timeline)
// ==========================================
/*const historyTimeline = ref<Marker[][]>([]);
const historyCursor = ref(-1);*/
// 🌟 核心优化：使用 shallowRef 彻底斩断深层 Proxy 劫持，节省 90% 以上性能开销
const historyTimeline = shallowRef<Marker[][]>([]);
const historyCursor = ref(-1);
// 建立基准态
function initHistory() {
  historyTimeline.value = [JSON.parse(JSON.stringify(markers.value))];
  historyCursor.value = 0;
}

// 提交动作，保存快照
function commitHistory() {
  // 1. 如果游标不在末尾，则剪断未来的时间轴，开辟新分支
  historyTimeline.value = historyTimeline.value.slice(0, historyCursor.value + 1);
  // 2. 存入当前修改后的新状态
  historyTimeline.value.push(JSON.parse(JSON.stringify(markers.value)));
  // 3. 游标右移
  historyCursor.value++;
}

// 撤销操作 (Undo)
function undoHistory() {
  if (historyCursor.value > 0) {
    historyCursor.value--;
    markers.value = JSON.parse(JSON.stringify(historyTimeline.value[historyCursor.value]));
    resetDraft(); // 强行收起右下角的编辑面板，清除选中状态
    playerRef.value?.triggerOSD("↩️ 撤销: 返回上一步");
  } else {
    playerRef.value?.triggerOSD("🚫 无可撤销");
  }
}

// 重做操作 (Redo)
function redoHistory() {
  if (historyCursor.value < historyTimeline.value.length - 1) {
    historyCursor.value++;
    markers.value = JSON.parse(JSON.stringify(historyTimeline.value[historyCursor.value]));
    resetDraft();
    playerRef.value?.triggerOSD("↪️ 重做: 还原撤销操作");
  } else {
    playerRef.value?.triggerOSD("🚫 无可重做");
  }
}
// ==========================================

const showDraftModal = ref(false);

const showSubTaskModal = ref(false);
const isExtracting = ref(false);
const pendingSubTaskMarker = ref<Marker | null>(null);
const pendingSubTaskRoute = ref<string>('');

const modalOffsetX = ref(0);
const modalOffsetY = ref(0);
let dragStartX = 0;
let dragStartY = 0;
let initialOffsetX = 0;
let initialOffsetY = 0;
const isDraggingModal = ref(false);

function startDragModal(e: MouseEvent) {
  isDraggingModal.value = true; dragStartX = e.clientX; dragStartY = e.clientY;
  initialOffsetX = modalOffsetX.value; initialOffsetY = modalOffsetY.value;
  document.addEventListener('mousemove', onDragModal); document.addEventListener('mouseup', stopDragModal);
}
function onDragModal(e: MouseEvent) {
  if (!isDraggingModal.value) return;
  modalOffsetX.value = initialOffsetX + (e.clientX - dragStartX);
  modalOffsetY.value = initialOffsetY + (e.clientY - dragStartY);
}
function stopDragModal() {
  isDraggingModal.value = false; document.removeEventListener('mousemove', onDragModal); document.removeEventListener('mouseup', stopDragModal);
}

watch(draftOut, (newVal, oldVal) => {
  if (newVal !== null && oldVal === null && draftIn.value !== null) {
    modalOffsetX.value = 0; modalOffsetY.value = 0;
  }
});

const isClearing = ref(false);
const unsavedChanges = ref(false);
const saveStatus = ref('');
let toastTimer: number | null = null;
const isAutoSaving = ref(false);
let autoSaveTimer: number | null = null;

const currentProjectMeta = ref<ProjectMeta | null>(null);
const currentProjectVersion = ref('1.1.0');

const showExportModal = ref(false);
const exportLogs = ref('');
const finalExportDir = ref('');
const exportCostTime = ref('');

const videoFileName = computed(() => {
  if (!videoPath.value) return '';
  return videoPath.value.split(/[/\\]/).pop() || '';
});

const activeMarkers = computed(() => markers.value.filter(m => !m.is_deleted));

function formatTime(seconds: number): string {
  if (isNaN(seconds) || seconds === null) return '00:00.00';
  const m = Math.floor(seconds / 60).toString().padStart(2, '0');
  const s = Math.floor(seconds % 60).toString().padStart(2, '0');
  const ms = Math.floor((seconds % 1) * 100).toString().padStart(2, '0');
  return `${m}:${s}.${ms}`;
}

function parseTimeStr(timeStr: string): number | null {
  if (!timeStr) return null;
  const str = timeStr.trim();
  if (str === '') return null;

  if (!str.includes(':')) {
    const secs = parseFloat(str); return isNaN(secs) ? null : secs;
  }
  const parts = str.split(':');
  if (parts.length === 2) {
    const m = parseInt(parts[0], 10); const s = parseFloat(parts[1]);
    if (isNaN(m) || isNaN(s)) return null; return m * 60 + s;
  }
  if (parts.length === 3) {
    const h = parseInt(parts[0], 10); const m = parseInt(parts[1], 10); const s = parseFloat(parts[2]);
    if (isNaN(h) || isNaN(m) || isNaN(s)) return null; return h * 3600 + m * 60 + s;
  }
  return null;
}

const colorPalette = ['#34d399', '#60a5fa', '#f472b6', '#fbbf24', '#c084fc', '#f87171', '#2dd4bf', '#818cf8'];
function getMarkerColor(index: number) { return colorPalette[index % colorPalette.length]; }

function showToast(msg: string) {
  saveStatus.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => saveStatus.value = '', 3000);
}

function updateDraftIn(e: Event) {
  const input = e.target as HTMLInputElement; const parsed = parseTimeStr(input.value);
  if (parsed === null) { draftIn.value = null; return; }
  const safeTime = Math.max(0, Math.min(parsed, videoDurationSec.value || 999999));
  if (draftOut.value !== null && safeTime >= draftOut.value) { playerRef.value?.triggerOSD("⚠️ 入点不能大于等于出点"); input.value = draftIn.value !== null ? formatTime(draftIn.value) : ''; return; }
  draftIn.value = safeTime; playerRef.value?.seekTo(safeTime);
}

function updateDraftOut(e: Event) {
  const input = e.target as HTMLInputElement; const parsed = parseTimeStr(input.value);
  if (parsed === null) { draftOut.value = null; return; }
  const safeTime = Math.max(0, Math.min(parsed, videoDurationSec.value || 999999));
  if (draftIn.value !== null && safeTime <= draftIn.value) { playerRef.value?.triggerOSD("⚠️ 出点不能小于等于入点"); input.value = draftOut.value !== null ? formatTime(draftOut.value) : ''; return; }
  draftOut.value = safeTime; playerRef.value?.seekTo(safeTime);
}

function nudgeIn(delta: number) {
  const baseTime = draftIn.value !== null ? draftIn.value : currentVideoTime.value;
  const safeTime = Math.max(0, Math.min(baseTime + delta, videoDurationSec.value || 999999));
  if (draftOut.value !== null && safeTime >= draftOut.value) { playerRef.value?.triggerOSD("⚠️ 接近出点限制"); return; }
  draftIn.value = Number(safeTime.toFixed(2)); playerRef.value?.seekTo(draftIn.value);
}

function nudgeOut(delta: number) {
  const baseTime = draftOut.value !== null ? draftOut.value : currentVideoTime.value;
  const safeTime = Math.max(0, Math.min(baseTime + delta, videoDurationSec.value || 999999));
  if (draftIn.value !== null && safeTime <= draftIn.value) { playerRef.value?.triggerOSD("⚠️ 接近入点限制"); return; }
  draftOut.value = Number(safeTime.toFixed(2)); playerRef.value?.seekTo(draftOut.value);
}

watch(currentVideoTime, () => {
  if (playerRef.value) {
    // @ts-ignore
    if (playerRef.value.duration) videoDurationSec.value = playerRef.value.duration;
  }
});

function resetDraft() {
  draftIn.value = null; draftOut.value = null; draftLabel.value = '';
  selectedMarkerId.value = null; showDraftModal.value = false;
}

async function loadVideoProject(targetPath: string) {
  videoPath.value = targetPath; videoSrc.value = convertFileSrc(targetPath);
  resetDraft(); unsavedChanges.value = false; markers.value = [];
  try {
    const project = await invoke<EasyCutProject>('load_project', { videoPath: targetPath });
    currentProjectMeta.value = project.meta; currentProjectVersion.value = project.version;
    markers.value = project.markers;
    if (markers.value.length > 0) markers.value.sort((a, b) => a.startTime - b.startTime);
    initHistory(); // 🌟 加载完毕后，瞬间建立第 0 帧历史基准态！
    showToast(`✅ 项目加载成功`);
  } catch (e) {
    console.error(e);
    markers.value = [];
    initHistory();
  }
}

async function selectVideo() {
  const selected = await open({ multiple: false, filters: [{ name: 'Videos', extensions: ['mp4', 'mkv', 'mov', 'avi'] }] });
  if (selected && typeof selected === 'string') await loadVideoProject(selected);
}

function closeProject() {
  videoPath.value = ''; videoSrc.value = ''; markers.value = []; currentProjectMeta.value = null; resetDraft(); initHistory();
}

function selectMarker(m: Marker) {
  selectedMarkerId.value = m.id;
  draftIn.value = m.startTime;
  draftOut.value = m.endTime;
  draftLabel.value = m.label;
  showDraftModal.value = false;
  playerRef.value?.seekTo(m.startTime);
}

function playSegment(m: Marker) {
  selectMarker(m);
  playerRef.value?.play();
}

async function handleSubTask(m: Marker, targetRoute: string) {
  if (unsavedChanges.value) {
    await saveProjectCore();
    unsavedChanges.value = false;
  }
  pendingSubTaskMarker.value = m;
  pendingSubTaskRoute.value = targetRoute;
  showSubTaskModal.value = true;
}

async function executeSubTask() {
  if (!pendingSubTaskMarker.value || !videoPath.value) return;

  isExtracting.value = true;
  playerRef.value?.triggerOSD("🚀 智能探针启动...");

  try {
    const m = pendingSubTaskMarker.value;
    const videoStem = videoFileName.value.substring(0, videoFileName.value.lastIndexOf('.'));
    const ext = videoFileName.value.split('.').pop() || 'mp4';

    const safeLabel = m.label.replace(/[\\/:*?"<>|]/g, "_") || `片段_${m.id}`;
    const index = activeMarkers.value.findIndex(marker => marker.id === m.id);
    const expectedBatchFileName = `[${String(index + 1).padStart(2, '0')}]_${videoStem}_${safeLabel}.${ext}`;

    let expectedBatchPath = null;
    if (finalExportDir.value) {
      const separator = finalExportDir.value.includes('\\') ? '\\' : '/';
      expectedBatchPath = `${finalExportDir.value}${separator}${expectedBatchFileName}`;
    }

    const inputDir = videoPath.value.substring(0, videoPath.value.lastIndexOf('\\') > -1 ? videoPath.value.lastIndexOf('\\') : videoPath.value.lastIndexOf('/'));
    const fallbackFileName = `${videoStem}_独立提取_${safeLabel}.${ext}`;

    const outPath = await invoke<string>('extract_single_segment', {
      params: {
        video_path: videoPath.value,
        start_time: m.startTime,
        end_time: m.endTime,
        expected_batch_path: expectedBatchPath,
        fallback_output_dir: inputDir,
        fallback_file_name: fallbackFileName
      }
    });

    showSubTaskModal.value = false;
    router.push({ path: pendingSubTaskRoute.value, query: { loadVideo: outPath } });

  } catch (err) {
    console.error(err);
    alert(`🚨 提取失败！\n详细错误: ${err}`);
  } finally {
    isExtracting.value = false;
  }
}
function setInPoint() {
  draftIn.value = currentVideoTime.value;
  if (draftOut.value !== null && draftOut.value <= draftIn.value) draftOut.value = null;
  playerRef.value?.triggerOSD(`[ 入点 ] ${formatTime(draftIn.value)}`);
}

function setOutPoint() {
  if (draftIn.value === null) {
    const lastEnd = activeMarkers.value.length > 0 ? activeMarkers.value[activeMarkers.value.length - 1].endTime : 0;
    if (currentVideoTime.value <= lastEnd) { playerRef.value?.triggerOSD("⚠️ 无法打出点：时间有误"); return; }
    draftIn.value = lastEnd;
  }
  if (currentVideoTime.value <= draftIn.value) { playerRef.value?.triggerOSD("⚠️ 出点必须大于入点"); return; }
  draftOut.value = currentVideoTime.value;

  showDraftModal.value = true;
  if (playerRef.value && typeof playerRef.value.pause === 'function') playerRef.value.pause();
  nextTick(() => draftInputRef.value?.focus());
}

function cancelDraft() {
  if (showDraftModal.value) {
    showDraftModal.value = false;
    playerRef.value?.triggerOSD("弹窗已收起");
  } else if (draftOut.value !== null) {
    if (selectedMarkerId.value) {
      resetDraft();
      playerRef.value?.triggerOSD("已退出编辑");
    } else {
      draftOut.value = null;
      playerRef.value?.triggerOSD("已取消出点");
    }
  } else if (draftIn.value !== null) {
    draftIn.value = null;
    playerRef.value?.triggerOSD("已取消入点");
  }
  if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
}

function saveMarker() {
  if (draftIn.value !== null && draftOut.value !== null) {
    const safeLabel = draftLabel.value.trim() || `片段_${Math.floor(Math.random()*1000)}`;

    if (selectedMarkerId.value) {
      const target = markers.value.find(m => m.id === selectedMarkerId.value);
      if (target) {
        target.startTime = draftIn.value; target.endTime = draftOut.value; target.label = safeLabel; target.is_deleted = false;
      }
      playerRef.value?.triggerOSD("✅ 修改已保存");
    } else {
      markers.value.push({ id: Date.now().toString(), startTime: draftIn.value, endTime: draftOut.value, label: safeLabel, is_deleted: false });
      playerRef.value?.triggerOSD("✅ 已添加");
    }

    markers.value.sort((a, b) => a.startTime - b.startTime);
    commitHistory(); // 🌟 保存修改后，提交历史快照入栈
    resetDraft();
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
  }
}

function removeMarker(id: string) {
  const target = markers.value.find(m => m.id === id);
  if (target) {
    target.is_deleted = true;
    commitHistory(); // 🌟 标记删除后，提交历史快照入栈
    playerRef.value?.triggerOSD("🗑️ 已删除，按 Ctrl+Z 撤销");
    if (selectedMarkerId.value === id) resetDraft();
  }
}

async function clearAllMarkers() {
  const confirmed = await confirm("确定要清空吗？清空操作支持通过 Ctrl+Z 撤销。");
  if (!confirmed) return;
  isClearing.value = true;
  try {
    await invoke('move_marker_file_to_trash', { videoPath: videoPath.value });
    markers.value.forEach(m => m.is_deleted = true);
    commitHistory(); // 🌟 清空全部后，提交历史快照入栈
    resetDraft();
    playerRef.value?.triggerOSD("🗑️ 已清空");
  } catch (e) { showToast("清理失败"); }
  finally { setTimeout(() => isClearing.value = false, 1000); }
}

async function saveProjectCore() {
  if (!currentProjectMeta.value) return;
  await invoke('save_project', { project: { version: currentProjectVersion.value, meta: currentProjectMeta.value, markers: markers.value } });
}

async function saveMarkers() {
  try { await saveProjectCore(); unsavedChanges.value = false; showToast(`✅ JSON 已安全保存`); } catch (err) { showToast(`❌ 保存失败`); }
}

watch(markers, () => {
  if (isClearing.value) return;
  unsavedChanges.value = true;
  if (!settingsStore.autoSave || !videoPath.value) return;
  isAutoSaving.value = true;
  if (autoSaveTimer) window.clearTimeout(autoSaveTimer);
  autoSaveTimer = window.setTimeout(async () => {
    try { await saveProjectCore(); unsavedChanges.value = false; } catch (e) {} finally { isAutoSaving.value = false; }
  }, 800);
}, { deep: true });

async function exportMarkers() {
  if (activeMarkers.value.length === 0) { alert("没有可导出的有效标记！"); return; }
  const outDir = await open({ directory: true, multiple: false, title: '选择导出的文件夹' });
  if (!outDir || typeof outDir !== 'string') return;
  isExporting.value = true;
  playerRef.value?.triggerOSD("🚀 正在呼叫多线程引擎，全速无损导出中...");
  const startTime = performance.now();
  try {
    const result = await invoke<{ logs: string, target_dir: string }>('execute_marker_split_task', {
      params: { video_path: videoPath.value, output_dir: outDir, max_concurrent_tasks: settingsStore.maxConcurrentTasks, license_str: authStore.isPro ? authStore.licenseKey : null }
    });
    exportCostTime.value = ((performance.now() - startTime) / 1000).toFixed(1);
    exportLogs.value = result.logs; finalExportDir.value = result.target_dir; showExportModal.value = true;
  } catch (error) {
    alert(`🚨 导出失败: ${error}`); console.error("导出异常:", error);
  } finally { isExporting.value = false; }
}

function onInputEsc(e: KeyboardEvent) {
  if (isPlayerFullscreen.value) {
    (e.target as HTMLInputElement).blur();
    return;
  }
  cancelDraft();
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key.toLowerCase() === 'escape') {
    if (isPlayerFullscreen.value) return;
    if (draftIn.value !== null || selectedMarkerId.value !== null) {
      e.preventDefault(); e.stopPropagation(); cancelDraft(); return;
    }
  }

  const activeTag = document.activeElement?.tagName.toLowerCase();
  // 🌟 焦点防呆：如果在输入框内打字，不触发全局快捷键 (包括系统的 Ctrl+Z)
  if (activeTag === 'input' || activeTag === 'textarea') return;
  if (!videoSrc.value || showExportModal.value || showSubTaskModal.value) return;

  // 🌟 核心：拦截 Ctrl/Command 组合键，实现全局Undo/Redo
  const isCtrl = e.ctrlKey || e.metaKey;
  const isShift = e.shiftKey;
  const key = e.key.toLowerCase();

  // 处理 Ctrl + Z (撤销) 以及 Ctrl + Shift + Z (重做)
  if (isCtrl && key === 'z') {
    e.preventDefault();
    if (isShift) {
      redoHistory();
    } else {
      undoHistory();
    }
    return;
  }

  // 处理 Ctrl + Y (重做)
  if (isCtrl && key === 'y') {
    e.preventDefault();
    redoHistory();
    return;
  }

  switch(key) {
    case 'i': case '[': e.preventDefault(); setInPoint(); break;
    case 'o': case ']': e.preventDefault(); setOutPoint(); break;
    case ' ': e.preventDefault(); playerRef.value?.togglePlay(); break;
    case 'f': e.preventDefault(); playerRef.value?.toggleFullscreen(); break;
    case 'm': e.preventDefault(); playerRef.value?.toggleMute(); break;
    case 'l': e.preventDefault(); playerRef.value?.toggleDrawer(); break;
    case 'arrowup': e.preventDefault(); if (playerRef.value) { playerRef.value.setVolume(playerRef.value.getVolume() + 0.1); playerRef.value.triggerOSD(`🔊 音量: ${Math.round(playerRef.value.getVolume()*100)}%`);} break;
    case 'arrowdown': e.preventDefault(); if (playerRef.value) { playerRef.value.setVolume(playerRef.value.getVolume() - 0.1); playerRef.value.triggerOSD(`🔉 音量: ${Math.round(playerRef.value.getVolume()*100)}%`);} break;
    case 'arrowleft': e.preventDefault(); playerRef.value?.seekOffset(-(e.shiftKey ? 1 : 5)); break;
    case 'arrowright': e.preventDefault(); playerRef.value?.seekOffset(e.shiftKey ? 1 : 5); break;
    case 'z': e.preventDefault(); if(playerRef.value) playerRef.value.setSpeed(Math.max(0.1, playerRef.value.getPlaybackRate() - 0.1)); break;
    case 'x': e.preventDefault(); if(playerRef.value) playerRef.value.setSpeed(Math.min(3.0, playerRef.value.getPlaybackRate() + 0.1)); break;
    case 'c': e.preventDefault(); playerRef.value?.setSpeed(1.0); break;
    case 'backspace': if (activeMarkers.value.length > 0) removeMarker(activeMarkers.value[activeMarkers.value.length - 1].id); break;
  }
}

watch(() => route.query.loadVideo, (newPath) => {
  if (!route.path.includes('/marker')) return;
  if (newPath && typeof newPath === 'string' && videoPath.value !== newPath) {
    loadVideoProject(newPath);
  }
}, { immediate: true });

onMounted(() => { isMounted.value = true; window.addEventListener('keydown', handleKeyDown); });
// 修改 ManualMarker.vue 的 onUnmounted：
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);

  // 🌟 扫雷：清理幽灵定时器
  if (toastTimer) clearTimeout(toastTimer);
  if (autoSaveTimer) clearTimeout(autoSaveTimer);

  // 🌟 扫雷：强杀残留的全局拖拽事件
  document.removeEventListener('mousemove', onDragModal);
  document.removeEventListener('mouseup', stopDragModal);
});
</script>

<style scoped>
.manual-marker-wrapper { display: flex; flex-direction: column; height: 100%; animation: fadeIn 0.3s ease; overflow: hidden; }
.header-actions { width: 400px; }
.active-file-badge { display: flex; align-items: center; background: #e2e8f0; padding: 0.4rem 0.6rem 0.4rem 1rem; border-radius: 30px; gap: 0.75rem; border: 1px solid #cbd5e1; box-shadow: inset 0 1px 2px rgba(0,0,0,0.05); max-width: 450px; margin-left: auto;}
.file-icon { font-size: 1.2rem; }
.file-name { font-weight: 600; color: #334155; font-size: 0.9rem; flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: monospace;}
.badge-actions { display: flex; gap: 0.25rem; border-left: 1px solid #cbd5e1; padding-left: 0.5rem;}

.marker-container { display: flex; flex: 1; min-height: 0; gap: 1.5rem; transition: all 0.3s;}
.marker-container.is-fullscreen { gap: 0; }

.list-card { background: #ffffff; width: 360px; flex-shrink: 0; display: flex; flex-direction: column; overflow: hidden;}
.list-header {
  flex-shrink: 0; padding: 1rem 1.25rem; border-bottom: 1px solid #e5e7eb;
  display: flex; justify-content: space-between; align-items: center;
  background: #f9fafb; overflow: hidden;
}
.danger-icon:hover { background: #fee2e2; color: #ef4444; border-radius: 6px;}
.save-icon:hover { background: #dbeafe; color: #2563eb; border-radius: 6px;}
.save-icon.pulse { animation: subtle-pulse 2s infinite; }
@keyframes subtle-pulse { 0%, 100% { transform: scale(1); opacity: 1; } 50% { transform: scale(1.1); opacity: 0.7; } }
.list-title { margin: 0; font-size: 1.05rem; font-weight: 700; color: #111827; white-space: nowrap; }

.save-actions {
  display: flex; align-items: center; gap: 10px;
  flex-wrap: nowrap; white-space: nowrap; flex-shrink: 0;
}
.action-btn-group { display: flex; gap: 4px; align-items: center; }

.auto-save-indicator {
  font-size: 0.75rem; color: #10b981; background: #d1fae5;
  padding: 4px 8px; border-radius: 4px; font-weight: 600;
  display: inline-block; line-height: 1.5;
}

.auto-save-indicator.saving { color: #6b7280; background: #f3f4f6; }

.list-body { flex: 1; overflow-y: auto; padding: 1rem; }
.empty-state { text-align: center; color: #6b7280; margin-top: 2rem;}
.marker-list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.75rem; }
.marker-item { display: flex; justify-content: space-between; padding: 0.85rem; background: #ffffff; border: 1px solid #e5e7eb; border-radius: 8px; border-left-width: 4px; cursor: pointer; transition: all 0.2s;}

.marker-item:hover { background: #f8fafc; border-color: #cbd5e1; }
.marker-item.is-selected { background: #eff6ff; border-color: #3b82f6; box-shadow: 0 2px 4px rgba(59,130,246,0.1); transform: scale(1.02); z-index: 2;}

.marker-info { display: flex; flex-direction: column; gap: 6px; overflow: hidden;}
.marker-header { display: flex; align-items: center; gap: 6px;}
.marker-index { color: white; font-size: 0.7rem; padding: 2px 6px; border-radius: 4px; font-weight: bold;}
.marker-label { font-weight: 600; color: #1f2937; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;}
.marker-time { font-family: monospace; font-size: 0.85rem; color: #6b7280; }

/* 🌟 操作按钮样式优化 */
.marker-actions { display: flex; gap: 4px; align-items: center; opacity: 0; transition: opacity 0.2s;}
.marker-item:hover .marker-actions { opacity: 1; }
.marker-item.is-selected .marker-actions { opacity: 1; }

.icon-btn { width: 28px; height: 28px; border-radius: 6px; border: 1px solid #cbd5e1; background: white; cursor: pointer; display: flex; align-items: center; justify-content: center; color: #64748b; font-size: 0.85rem; transition: 0.2s;}
.icon-btn:hover { border-color: #3b82f6; color: #2563eb; }

.play-btn { color: #10b981; }
.play-btn:hover { border-color: #10b981; color: #059669; background: #ecfdf5; }
.split-btn:hover { border-color: #8b5cf6; color: #7c3aed; background: #f5f3ff; }
.marker-btn:hover { border-color: #f59e0b; color: #d97706; background: #fffbeb; }
.delete-btn { color: #ef4444; }
.delete-btn:hover { border-color: #fca5a5; background: #fee2e2; color: #b91c1c; }

.fullscreen-draft-modal {
  position: fixed; bottom: 8%; left: 50%;
  width: auto; min-width: 480px;
  background: rgba(15,23,42,0.8); border: 1px solid rgba(56,189,248, 0.4);
  padding: 1.2rem 1.5rem; border-radius: 12px;
  backdrop-filter: blur(12px); box-shadow: 0 15px 30px rgba(0,0,0,0.5); text-align: center; z-index: 9999;
}
.modal-drag-handle {
  cursor: grab; display: flex; justify-content: space-between; align-items: center;
  border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 0.6rem; margin-bottom: 1rem;
}
.modal-drag-handle:active { cursor: grabbing; }
.modal-title { color: white; margin: 0; font-size: 1.1rem; pointer-events: none;}
.drag-hint { font-size: 0.75rem; color: #94a3b8; font-weight: normal; margin-left: 6px;}

.modal-close-btn {
  background: transparent; border: none; color: #94a3b8; font-size: 1.2rem; cursor: pointer; transition: color 0.2s;
  padding: 0 4px; pointer-events: auto;
}
.modal-close-btn:hover { color: #ef4444; }

.modal-time-visual {
  display: flex; align-items: center; justify-content: center; gap: 1rem; margin-bottom: 1.2rem;
}
.modal-time-visual .label { font-size: 0.8rem; color: #94a3b8; font-weight: 600; margin-bottom: 6px; display: block; pointer-events: none;}
.modal-time-visual .time-editor-wrapper {
  background: rgba(30,41,59,0.8); border: 1px solid #475569; border-radius: 8px;
}
.modal-time-visual .nudge-btn { background: transparent; color: #cbd5e1; border-color: #475569; }
.modal-time-visual .nudge-btn:hover { background: rgba(255,255,255,0.1); color: white; }
.modal-time-visual .time-input { color: #e2e8f0; border-color: #475569; }
.modal-time-visual .time-input:focus { color: white; border-color: #38bdf8; }
.modal-time-visual .arrow { color: #475569; padding-top: 18px; pointer-events: none;}

.modal-input-group { display: flex; gap: 0.75rem; width: 100%;}
.modal-input-group input { flex: 1; min-width: 0; padding: 0.7rem 1rem; border-radius: 8px; border: 1px solid #475569; background: rgba(30,41,59,0.8); color: white; font-size: 0.95rem; outline: none; transition: border 0.2s;}
.modal-input-group input:focus { border-color: #38bdf8; }
.modal-input-group button { flex-shrink: 0; padding: 0 1.5rem; background: #38bdf8; border: none; border-radius: 8px; font-weight: bold; font-size: 0.95rem; cursor: pointer; color: #0f172a; white-space: nowrap; transition: 0.2s;}
.modal-input-group button:disabled { background: #475569; color: #94a3b8; cursor: not-allowed; }
.modal-input-group button:hover:not(:disabled) { background: #7dd3fc; }
.modal-input-group button.is-editing { background: #f59e0b; color: white;}
.modal-input-group button.is-editing:hover { background: #fbbf24;}

.slide-up-fade-enter-active, .slide-up-fade-leave-active { transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1); }
.slide-up-fade-enter-from { opacity: 0; transform: translate(calc(-50% + 0px), 20px); }
.slide-up-fade-leave-to { opacity: 0; transform: translate(calc(-50% + 0px), 20px); }

/* 底部编辑器 UI */
.draft-section { flex-shrink: 0; padding: 0.8rem; background: #f8fafc; border-top: 1px solid #e5e7eb; display: flex; flex-direction: column; align-items: center; gap: 12px; }
.draft-time-visual { display: flex; align-items: center; gap: 0.8rem; }

.point-box { display: flex; flex-direction: column; align-items: center; gap: 4px; }
.point-box .label { font-size: 0.75rem; color: #64748b; font-weight: 600; }

.time-editor-wrapper {
  display: flex; align-items: center; background: #ffffff; border: 1px solid #cbd5e1;
  border-radius: 6px; overflow: hidden; height: 34px; transition: 0.2s;
  width: max-content;
}
.time-editor-wrapper:focus-within { border-color: #3b82f6; box-shadow: 0 0 0 2px rgba(59,130,246,0.1); }

.nudge-btn {
  background: #f8fafc; border: none; color: #64748b;
  width: 28px; height: 100%; flex-shrink: 0;
  cursor: pointer; font-weight: bold; font-size: 1.1rem; display: flex;
  justify-content: center; align-items: center; transition: 0.2s;
}
.nudge-btn:hover { background: #e2e8f0; color: #2563eb; }
.nudge-btn:active { background: #cbd5e1; }

.time-input {
  width: 75px; flex-shrink: 0;
  height: 100%; border: none; border-left: 1px solid #e2e8f0; border-right: 1px solid #e2e8f0;
  text-align: center; font-family: 'Consolas', monospace; font-size: 0.85rem; color: #475569;
  outline: none; background: transparent; transition: 0.2s;
}
.time-input.highlight { color: #2563eb; font-weight: 700; background: #eff6ff;}

.arrow { color: #94a3b8; font-size: 1.2rem; padding-top: 18px; }

/* 底部操作区 */
.draft-quick-actions {
  display: flex; gap: 8px; justify-content: center; width: 100%;
}
.quick-btn {
  padding: 6px 14px; border-radius: 6px; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: 0.2s; border: 1px solid transparent;
}
.quick-btn.primary { background: #2563eb; color: white; flex: 1; display: flex; align-items: center; justify-content: center; gap: 4px; box-shadow: 0 1px 2px rgba(37,99,235,0.2);}
.quick-btn.primary:hover { background: #1d4ed8; }
.quick-btn.outline { background: white; border-color: #cbd5e1; color: #475569; display: flex; align-items: center; gap: 4px; box-shadow: 0 1px 2px rgba(0,0,0,0.02);}
.quick-btn.outline:hover { background: #f8fafc; border-color: #94a3b8; color: #1e293b;}
.quick-btn.ghost { background: transparent; color: #64748b; display: flex; align-items: center; gap: 4px;}
.quick-btn.ghost:hover { background: #f1f5f9; color: #334155; }

.list-footer { padding: 1rem 1.5rem; border-top: 1px solid #e5e7eb; background: #f9fafb; flex-shrink: 0;}
.export-btn { width: 100%; padding: 0.85rem; font-size: 1.05rem; background: linear-gradient(135deg, #2563eb, #1d4ed8); box-shadow: 0 4px 6px -1px rgba(37, 99, 235, 0.2); transition: all 0.3s ease; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer;}
.export-btn:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 6px 8px -1px rgba(37, 99, 235, 0.3); }

.global-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px dashed #d1d5db; color: #6b7280; border-radius: 12px; height: 100%;}

/* 🌟 子任务弹窗 (SubTask Modal) */
.subtask-overlay { position: fixed; inset: 0; background: rgba(15, 23, 42, 0.6); backdrop-filter: blur(3px); z-index: 10000; display: flex; justify-content: center; align-items: center; }
.subtask-modal-content { background: #ffffff; width: 450px; border-radius: 12px; box-shadow: 0 20px 40px -10px rgba(0, 0, 0, 0.2); overflow: hidden; display: flex; flex-direction: column; }
.subtask-modal-content .modal-header { padding: 1.2rem 1.5rem; border-bottom: 1px solid #e5e7eb; display: flex; justify-content: space-between; align-items: flex-start; }
.subtask-modal-content .header-left { display: flex; align-items: center; gap: 12px; }
.subtask-modal-content .success-icon { font-size: 1.5rem; background: #eff6ff; border: 1px solid #bfdbfe; border-radius: 10px; width: 40px; height: 40px; display: flex; align-items: center; justify-content: center; }
.subtask-modal-content .header-text h2 { margin: 0 0 4px 0; font-size: 1.1rem; color: #111827; font-weight: 800; }
.subtask-modal-content .subtitle { margin: 0; font-size: 0.8rem; color: #64748b; }
.subtask-modal-content .modal-body { padding: 1.5rem; }
.subtask-modal-content .info-box { font-size: 0.85rem; color: #64748b; line-height: 1.5; background: #f8fafc; padding: 12px; border-radius: 8px; border: 1px solid #e2e8f0; margin-top: 1rem;}
.subtask-modal-content .modal-footer { padding: 1rem 1.5rem; background: #f8fafc; border-top: 1px solid #e5e7eb; display: flex; justify-content: flex-end; gap: 10px; }

.modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 0.25s ease; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }
.modal-fade-enter-active .subtask-modal-content { animation: modalPopIn 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275); }
</style>