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
      <!-- 播放器子组件 (完全解耦) -->
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
        <!-- 打轴输入框插槽：保证全屏时不被遮挡 -->
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
              <li v-for="(m, index) in activeMarkers" :key="m.id" class="marker-item" :style="{ borderLeftColor: getMarkerColor(index) }">
                <div class="marker-info">
                  <div class="marker-header">
                    <span class="marker-index" :style="{ backgroundColor: getMarkerColor(index), color: '#fff' }">#{{ index + 1 }}</span>
                    <span class="marker-label" :title="m.label">{{ m.label }}</span>
                  </div>
                  <span class="marker-time">{{ formatTime(m.startTime) }} ➔ {{ formatTime(m.endTime) }}</span>
                </div>
                <div class="marker-actions">
                  <button class="icon-btn play-btn" @click="playerRef?.seekTo(m.startTime)" title="播放">▶</button>
                  <button class="icon-btn delete-btn" @click="removeMarker(m.id)" title="删除">✖</button>
                </div>
              </li>
            </transition-group>
          </ul>
        </div>

        <div class="draft-section" :class="{ 'active': draftIn !== null }">
          <div class="draft-time-visual">
            <div class="point-box"><span class="label">入点 (IN)</span><span class="value" :class="{'highlight': draftIn !== null}">{{ draftIn !== null ? formatTime(draftIn) : '--:--.--' }}</span></div>
            <div class="arrow">➔</div>
            <div class="point-box"><span class="label">出点 (OUT)</span><span class="value" :class="{'highlight': draftOut !== null}">{{ draftOut !== null ? formatTime(draftOut) : '等待打点...' }}</span></div>
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

  <!-- 轻提示 -->
  <transition name="toast"><div v-if="saveStatus" class="toast-message">{{ saveStatus }}</div></transition>

  <!-- 🌟 核心修复：解除注释，重新激活带有精美计时日志的导出弹窗 -->
  <ExportResultModal
      v-model="showExportModal"
      :logs="exportLogs"
      :exportDir="finalExportDir"
      :costTime="exportCostTime"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useRoute } from 'vue-router'; // 🌟 新增：引入当前路由对象
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

// 引入解耦的组件
import EasyCutPlayer from './EasyCutPlayer.vue';
import ExportResultModal from './ExportResultModal.vue';
import { useSettingsStore } from '../stores/settings';

// ==========================================
// 1. 数据契约与状态
// ==========================================
interface Marker { id: string; startTime: number; endTime: number; label: string; is_deleted: boolean; }
interface ProjectMeta { project_id: string; project_name: string; created_at: number; linked_file_hash: string | null; linked_file_name: string | null; source_type: string; [key: string]: any; }
interface EasyCutProject { version: string; meta: ProjectMeta; markers: Marker[]; }

const settingsStore = useSettingsStore();
const route = useRoute(); // 🌟 新增：获取路由参数
// 引用子组件实例
const playerRef = ref<InstanceType<typeof EasyCutPlayer> | null>(null);

// 业务状态
const videoPath = ref('');
const videoSrc = ref('');
const currentVideoTime = ref(0);
const isPlayerFullscreen = ref(false);

const draftIn = ref<number | null>(null);
const draftOut = ref<number | null>(null);
const draftLabel = ref('');
const draftInputRef = ref<HTMLInputElement | null>(null);

const markers = ref<Marker[]>([]);
const deletedHistory = ref<Marker[]>([]);
const lastBatchSnapshot = ref<Marker[] | null>(null);

const isClearing = ref(false);
const unsavedChanges = ref(false);
const saveStatus = ref('');
let toastTimer: number | null = null;
const isAutoSaving = ref(false);
let autoSaveTimer: number | null = null;

const currentProjectMeta = ref<ProjectMeta | null>(null);
const currentProjectVersion = ref('1.1.0');

const isExporting = ref(false);
const showExportModal = ref(false);
const exportLogs = ref('');
const finalExportDir = ref('');


const exportCostTime = ref(''); // 🌟 新增：单独存储耗时


// ==========================================
// 2. 计算属性
// ==========================================
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
const colorPalette = ['#34d399', '#60a5fa', '#f472b6', '#fbbf24', '#c084fc', '#f87171', '#2dd4bf', '#818cf8'];
function getMarkerColor(index: number) { return colorPalette[index % colorPalette.length]; }

function showToast(msg: string) {
  saveStatus.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => saveStatus.value = '', 3000);
}

// ==========================================
// 3. 业务逻辑
// ==========================================
// ==========================================
// 3. 视频与项目管理
// ==========================================

// 🌟 修复：抽离出独立的核心加载逻辑，供手动选择和路由跳转复用
async function loadVideoProject(targetPath: string) {
  videoPath.value = targetPath;
  videoSrc.value = convertFileSrc(targetPath);
  draftIn.value = null; draftOut.value = null; draftLabel.value = '';
  unsavedChanges.value = false; markers.value = []; deletedHistory.value = []; lastBatchSnapshot.value = null;

  try {
    const project = await invoke<EasyCutProject>('load_project', { videoPath: targetPath });
    currentProjectMeta.value = project.meta;
    currentProjectVersion.value = project.version;
    markers.value = project.markers;
    if (markers.value.length > 0) markers.value.sort((a, b) => a.startTime - b.startTime);
    showToast(`✅ 项目加载成功`);
  } catch (e) {
    console.error(e);
    markers.value = [];
  }
}

// UI 触发的手动选择文件
async function selectVideo() {
  const selected = await open({ multiple: false, filters: [{ name: 'Videos', extensions: ['mp4', 'mkv', 'mov', 'avi'] }] });
  if (selected && typeof selected === 'string') {
    await loadVideoProject(selected);
  }
}

function closeProject() {
  videoPath.value = ''; videoSrc.value = ''; markers.value = []; currentProjectMeta.value = null; draftIn.value = null; draftOut.value = null; deletedHistory.value = [];
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
  playerRef.value?.togglePlay();
  nextTick(() => draftInputRef.value?.focus());
}

function addMarker() {
  if (draftIn.value !== null && draftOut.value !== null && draftLabel.value.trim() !== '') {
    markers.value.push({ id: Date.now().toString(), startTime: draftIn.value, endTime: draftOut.value, label: draftLabel.value.trim(), is_deleted: false });
    markers.value.sort((a, b) => a.startTime - b.startTime);
    draftIn.value = null; draftOut.value = null; draftLabel.value = '';
    playerRef.value?.triggerOSD("✅ 已添加");
    playerRef.value?.togglePlay();
  }
}

function removeMarker(id: string) {
  const target = markers.value.find(m => m.id === id);
  if (target) {
    target.is_deleted = true; deletedHistory.value.push(target);
    playerRef.value?.triggerOSD("🗑️ 已删除，按 Ctrl+Z 撤销");
  }
}

async function clearAllMarkers() {
  const confirmed = await confirm("确定要清空吗？原记录将备份至回收站。");
  if (!confirmed) return;
  isClearing.value = true;
  try {
    await invoke('move_marker_file_to_trash', { videoPath: videoPath.value });
    lastBatchSnapshot.value = JSON.parse(JSON.stringify(markers.value));
    markers.value.forEach(m => m.is_deleted = true);
    playerRef.value?.triggerOSD("🗑️ 已清空");
  } catch (e) { showToast("备份失败"); }
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
  const outDir = await open({ directory: true, multiple: false });
  if (!outDir || typeof outDir !== 'string') return;

  isExporting.value = true;
  playerRef.value?.triggerOSD("🚀 FFmpeg 极速引擎启动...");
  if (autoSaveTimer) window.clearTimeout(autoSaveTimer);
  const startTime = performance.now();
  try {
    const result = await invoke<{logs: string, target_dir: string}>('execute_marker_split_task', { videoPath: videoPath.value, outputDir: outDir });
    // 🌟 核心修改：分离日志与时间
    exportCostTime.value = ((performance.now() - startTime) / 1000).toFixed(1);
    exportLogs.value = result.logs; // 不再拼接时间文本
    finalExportDir.value = result.target_dir;

    // 🌟 核心触发：打开含有计时的弹窗！
    showExportModal.value = true;
  } catch (err) {
    alert(`❌ 导出失败:\n${err}`);
  } finally {
    isExporting.value = false;
  }
}

// ==========================================
// 6. 全局快捷键路由
// ==========================================
function handleKeyDown(e: KeyboardEvent) {
  const activeTag = document.activeElement?.tagName.toLowerCase();
  if (activeTag === 'input' || activeTag === 'textarea') return;

  // 🌟 修复关键：如果弹窗开启，阻止一切快捷键，避免误触
  if (!videoSrc.value || showExportModal.value) return;

  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') {
    e.preventDefault();
    if (lastBatchSnapshot.value) { markers.value = [...lastBatchSnapshot.value]; lastBatchSnapshot.value = null; playerRef.value?.triggerOSD("↩️ 恢复全部"); return; }
    if (deletedHistory.value.length > 0) { const last = deletedHistory.value.pop()!; last.is_deleted = false; playerRef.value?.triggerOSD("↩️ 撤销删除"); return; }
    return;
  }

  // 播放器操作移交子组件
  switch(e.key.toLowerCase()) {
    case 'i': case '[': e.preventDefault(); setInPoint(); break;
    case 'o': case ']': e.preventDefault(); setOutPoint(); break;
    case ' ': e.preventDefault(); playerRef.value?.togglePlay(); break;
    case 'f': e.preventDefault(); playerRef.value?.toggleFullscreen(); break;
    case 'm': e.preventDefault(); playerRef.value?.toggleMute(); break;
      // 在 handleKeyDown 的 switch 语句中：
    case 'l': e.preventDefault(); playerRef.value?.toggleDrawer(); break; // 🌟 修复：恢复 L 键呼出侧边栏
    case 'arrowup': e.preventDefault(); if (playerRef.value) { playerRef.value.setVolume(playerRef.value.getVolume() + 0.1); playerRef.value.triggerOSD(`🔊 音量: ${Math.round(playerRef.value.getVolume()*100)}%`);} break;
    case 'arrowdown': e.preventDefault(); if (playerRef.value) { playerRef.value.setVolume(playerRef.value.getVolume() - 0.1); playerRef.value.triggerOSD(`🔉 音量: ${Math.round(playerRef.value.getVolume()*100)}%`);} break;
    case 'arrowleft': e.preventDefault(); playerRef.value?.seekTo(currentVideoTime.value - (e.shiftKey ? 1 : 5)); break;
    case 'arrowright': e.preventDefault(); playerRef.value?.seekTo(currentVideoTime.value + (e.shiftKey ? 1 : 5)); break;
    case 'z': e.preventDefault(); if(playerRef.value) playerRef.value.setSpeed(Math.max(0.1, playerRef.value.getPlaybackRate() - 0.1)); break;
    case 'x': e.preventDefault(); if(playerRef.value) playerRef.value.setSpeed(Math.min(3.0, playerRef.value.getPlaybackRate() + 0.1)); break;
    case 'c': e.preventDefault(); playerRef.value?.setSpeed(1.0); break;
    case 'backspace': if (activeMarkers.value.length > 0) removeMarker(activeMarkers.value[activeMarkers.value.length - 1].id); break;
  }
}

// 🌟 核心修复：使用 watch 监听路由参数变化（无视 keep-alive 缓存）
watch(
    () => route.query.loadVideo,
    (newPath) => {
      if (newPath && typeof newPath === 'string') {
        // 避免重复加载当前正在处理的同一个视频
        if (videoPath.value !== newPath) {
          loadVideoProject(newPath);
        }
      }
    },
    { immediate: true } // immediate: true 保证了不管是第一次进页面还是后续跳转，都能立刻触发
);

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<style scoped>
/* 包含父组件特有的布局和列表样式 */
.manual-marker-wrapper { display: flex; flex-direction: column; height: 100%; animation: fadeIn 0.3s ease; overflow: hidden; }
.header-actions { width: 400px; }
.active-file-badge { display: flex; align-items: center; background: #e2e8f0; padding: 0.4rem 0.6rem 0.4rem 1rem; border-radius: 30px; gap: 0.75rem; border: 1px solid #cbd5e1; box-shadow: inset 0 1px 2px rgba(0,0,0,0.05); max-width: 450px; margin-left: auto;}
.file-icon { font-size: 1.2rem; }
.file-name { font-weight: 600; color: #334155; font-size: 0.9rem; flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: monospace;}
.badge-actions { display: flex; gap: 0.25rem; border-left: 1px solid #cbd5e1; padding-left: 0.5rem;}

.marker-container { display: flex; flex: 1; min-height: 0; gap: 1.5rem; transition: all 0.3s;}
.marker-container.is-fullscreen { gap: 0; }

.list-card { background: #ffffff; width: 320px; flex-shrink: 0; display: flex; flex-direction: column; overflow: hidden;}
.list-header {
  flex-shrink: 0; padding: 1rem 1.25rem; border-bottom: 1px solid #e5e7eb;
  display: flex; justify-content: space-between; align-items: center;
  background: #f9fafb; overflow: hidden; /* 防止溢出 */
}
/*.action-btn-group { display: flex; gap: 4px; }*/
.danger-icon:hover { background: #fee2e2; color: #ef4444; border-radius: 6px;}
.save-icon:hover { background: #dbeafe; color: #2563eb; border-radius: 6px;}
.save-icon.pulse { animation: subtle-pulse 2s infinite; }
@keyframes subtle-pulse { 0%, 100% { transform: scale(1); opacity: 1; } 50% { transform: scale(1.1); opacity: 0.7; } }
.list-title { margin: 0; font-size: 1.05rem; font-weight: 700; color: #111827; white-space: nowrap; }



/* 🌟 核心修复：强制不换行，并确保子元素横向居中对齐 */
.save-actions {
  display: flex; align-items: center; gap: 10px;
  flex-wrap: nowrap; white-space: nowrap; flex-shrink: 0;
}
.action-btn-group { display: flex; gap: 4px; align-items: center; }

/* 🌟 优化状态标签，使其高度和图标按钮对齐 */
.auto-save-indicator {
  font-size: 0.75rem; color: #10b981; background: #d1fae5;
  padding: 4px 8px; border-radius: 4px; font-weight: 600;
  display: inline-block; line-height: 1.5;
}

.auto-save-indicator.saving { color: #6b7280; background: #f3f4f6; }



.list-body { flex: 1; overflow-y: auto; padding: 1rem; }
.empty-state { text-align: center; color: #6b7280; margin-top: 2rem;}
.marker-list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.75rem; }
.marker-item { display: flex; justify-content: space-between; padding: 0.85rem; background: #ffffff; border: 1px solid #e5e7eb; border-radius: 8px; border-left-width: 4px; cursor: pointer;}
.marker-item:hover { background: #f8fafc; border-color: #cbd5e1; }
.marker-info { display: flex; flex-direction: column; gap: 6px; overflow: hidden;}
.marker-header { display: flex; align-items: center; gap: 6px;}
.marker-index { color: white; font-size: 0.7rem; padding: 2px 6px; border-radius: 4px; font-weight: bold;}
.marker-label { font-weight: 600; color: #1f2937; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;}
.marker-time { font-family: monospace; font-size: 0.85rem; color: #6b7280; }
.marker-actions { display: flex; gap: 0.25rem; align-items: center; opacity: 0; transition: opacity 0.2s;}
.marker-item:hover .marker-actions { opacity: 1; }

.play-btn { color: #10b981; }
.play-btn:hover { background: #d1fae5; color: #059669; }
.delete-btn { color: #ef4444; }
.delete-btn:hover { background: #fee2e2; color: #b91c1c; }

/* 全屏打轴框样式 */
.fullscreen-draft-modal {
  position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
  width: 90%; max-width: 450px;
  background: rgba(15,23,42,0.85); border: 1px solid #38bdf8; padding: 2rem; border-radius: 16px;
  backdrop-filter: blur(12px); box-shadow: 0 25px 50px -12px rgba(0,0,0,0.6); text-align: center; z-index: 50;
}
.fullscreen-draft-modal h3 { color: white; margin: 0 0 0.5rem 0; font-size: 1.5rem;}
.modal-time { color: #94a3b8; font-family: monospace; font-size: 1.3rem; margin-bottom: 1.5rem; }
.modal-input-group { display: flex; gap: 0.75rem; width: 100%;}
.modal-input-group input { flex: 1; min-width: 0; padding: 0.8rem 1rem; border-radius: 8px; border: 1px solid #475569; background: #1e293b; color: white; font-size: 1rem; outline: none;}
.modal-input-group input:focus { border-color: #38bdf8; }
.modal-input-group button { flex-shrink: 0; padding: 0 1.5rem; background: #38bdf8; border: none; border-radius: 8px; font-weight: bold; font-size: 1rem; cursor: pointer; color: #0f172a; white-space: nowrap;}

.draft-section { flex-shrink: 0; padding: 1rem; background: #f8fafc; border-top: 1px solid #e5e7eb; display: flex; justify-content: center; height: 60px;}
.draft-time-visual { display: flex; align-items: center; gap: 1rem; }
.point-box { background: transparent; border-radius: 8px; text-align: center; }
.point-box .label { font-size: 0.75rem; color: #64748b; margin-right: 8px;}
.point-box .value { font-family: monospace; font-size: 1.1rem; color: #475569; }
.point-box .value.highlight { color: #2563eb; font-weight: bold; }
.arrow { color: #475569; font-size: 1rem; }

.list-footer { padding: 1rem 1.5rem; border-top: 1px solid #e5e7eb; background: #f9fafb; flex-shrink: 0;}
.export-btn { width: 100%; padding: 0.85rem; font-size: 1.05rem; background: linear-gradient(135deg, #2563eb, #1d4ed8); box-shadow: 0 4px 6px -1px rgba(37, 99, 235, 0.2); transition: all 0.3s ease;}
.export-btn:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 6px 8px -1px rgba(37, 99, 235, 0.3); }

.global-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px dashed #d1d5db; color: #6b7280; border-radius: 12px; height: 100%;}
.zoom-in-enter-active, .zoom-in-leave-active { transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1); }
.zoom-in-enter-from, .zoom-in-leave-to { opacity: 0; transform: translate(-50%, -50%) scale(0.9); }
</style>