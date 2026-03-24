<template>
  <div class="app-container">
    <header class="header">
      <div class="logo">✂️</div>
      <div class="title-area">
        <h1>易剪 (EasyCut)</h1>
        <p class="subtitle">极速无损视频处理引擎 v1.0</p>
      </div>
    </header>

    <main class="main-content">
      <div class="card">
        <div class="card-header">
          <h2>⏱️ 固定时长批量分割</h2>
          <p>将长视频按设定的时间（如每 5 分钟）切分成多个无损片段。</p>
        </div>

        <div class="card-body">
          <div class="form-group">
            <label>🎬 输入视频文件</label>
            <div class="input-with-btn">
              <input v-model="batchParams.inputPath" type="text" readonly placeholder="请选择需要分割的视频..." />
              <button class="secondary-btn" @click="selectInputFile">浏览文件</button>
            </div>
          </div>

          <div class="form-group">
            <label>📁 输出保存目录</label>
            <div class="input-with-btn">
              <input v-model="batchParams.outputDir" type="text" readonly placeholder="请选择保存目录..." />
              <button class="secondary-btn" @click="selectOutputDir">更改目录</button>
            </div>
            <span class="hint" v-if="batchParams.videoName && batchParams.outputDir">
              预期生成: {{ batchParams.outputDir }}\<b>{{ batchParams.videoName }}</b>\{{ batchParams.videoName }}_part1.mp4
            </span>
          </div>

          <div class="form-group">
            <label>⏳ 每个片段时长 (秒)</label>
            <input
                v-model.number="batchParams.segmentDuration"
                type="number"
                min="1"
                placeholder="例如: 300"
            />
            <span class="hint">当前设置: {{ (batchParams.segmentDuration / 60).toFixed(1) }} 分钟 / 段</span>
          </div>

          <button class="primary-btn" @click="runBatchSplit" :disabled="isProcessing">
            <span v-if="isProcessing" class="spinner">⚙️</span>
            {{ isProcessing ? '引擎全速处理中...' : '🚀 开始批量分割' }}
          </button>

          <div v-if="resultLogs.length > 0" :class="['result-box', isError ? 'error' : 'success']">
            <div class="result-header">
              <span>🎉 批量分割任务完成！(共 {{ resultLogs.length }} 段)</span>
              <button class="text-btn" @click="isExpanded = !isExpanded" v-if="resultLogs.length > 5">
                {{ isExpanded ? '收起列表' : '展开全部' }}
              </button>
            </div>
            <ul class="log-list">
              <li v-for="(log, index) in visibleLogs" :key="index">{{ log }}</li>
              <li v-if="!isExpanded && resultLogs.length > 5" class="ellipsis">... 还有 {{ resultLogs.length - 5 }} 个片段被隐藏 ...</li>
            </ul>
          </div>
        </div>
      </div>
      <div class="card" style="margin-top: 2rem;">
        <div class="card-header">
          <h2>🏷️ 智能视频打轴与标记</h2>
          <p>为长视频标记精彩片段，生成专属 JSON 配置，随时可根据标记无损提取。</p>
        </div>

        <div class="card-body">
          <div class="form-group">
            <label>🎬 选择要在本软件中预览打轴的视频</label>
            <div class="input-with-btn">
              <input v-model="markerParams.videoPath" type="text" readonly placeholder="请选择视频文件..." />
              <button class="secondary-btn" @click="selectVideoForMarker">加载视频</button>
            </div>
          </div>

          <div v-if="markerParams.videoSrc" class="player-section">
            <video
                ref="videoPlayerRef"
                controls
                class="video-player"
                :src="markerParams.videoSrc"
                @timeupdate="onTimeUpdate"
            ></video>

            <div class="controls-bar">
              <div class="time-display">
                当前时间: <span>{{ currentTime.toFixed(2) }} s</span>
              </div>
              <div class="action-buttons">
                <button class="mark-btn in" @click="setInPoint">设置入点 [</button>
                <button class="mark-btn out" @click="setOutPoint">设置出点 ]</button>
              </div>
            </div>

            <div class="draft-marker" v-if="draftMarker.startTime !== null">
          <span>待添加片段: {{ draftMarker.startTime.toFixed(2) }}s ~
            {{ draftMarker.endTime !== null ? draftMarker.endTime.toFixed(2) + 's' : '等待打出点...' }}
          </span>
              <input v-model="draftMarker.label" type="text" placeholder="输入片段描述 (如: 精彩打斗)" />
              <button class="primary-btn small" @click="addMarkerToList" :disabled="draftMarker.endTime === null || !draftMarker.label">
                添加至列表
              </button>
            </div>
          </div>

          <div class="marker-list-section" v-if="markers.length > 0">
            <h3>已标记片段 ({{ markers.length }})</h3>
            <ul class="marker-list">
              <li v-for="(m, index) in markers" :key="m.id">
                <span class="tag">{{ m.label }}</span>
                <span class="time">{{ m.startTime.toFixed(1) }}s - {{ m.endTime.toFixed(1) }}s</span>
                <button class="text-btn danger" @click="removeMarker(index)">删除</button>
              </li>
            </ul>
            <button class="primary-btn" @click="saveMarkersToJSON">💾 保存配置文件 (JSON)</button>
            <p v-if="saveStatus" class="status-msg">{{ saveStatus }}</p>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import {ref, reactive, computed} from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog'; // <--- 引入 dialog 插件
import { convertFileSrc } from '@tauri-apps/api/core'; // Tauri V2 用来加载本地文件的 API
// 状态管理
const isProcessing = ref(false);
const resultLogs = ref<string[]>([]); // 改为数组，方便渲染折叠列表
const isError = ref(false);
const isExpanded = ref(false); // 控制是否展开长列表
// 表单数据绑定
const batchParams = reactive({
  inputPath: '',
  outputDir: '',
  videoName: '', // 纯视频名称（如：20231024.先导片）
  segmentDuration: 300
});
const visibleLogs = computed(() => {
  return isExpanded.value ? resultLogs.value : resultLogs.value.slice(0, 5);
});

// --- 状态与引用 ---
const videoPlayerRef = ref<HTMLVideoElement | null>(null);
const currentTime = ref(0);
const saveStatus = ref('');
async function selectInputFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Video Files', extensions: ['mp4', 'mkv', 'mov'] }]
  });

  if (selected && typeof selected === 'string') {
    batchParams.inputPath = selected;

    // 智能解析路径 (兼容 Windows \ 和 Mac/Linux /)
    const normalizedPath = selected.replace(/\\/g, '/');
    const pathParts = normalizedPath.split('/');
    const fileNameWithExt = pathParts.pop() || '';

    // 提取纯名称 (去掉 .mp4)
    const dotIndex = fileNameWithExt.lastIndexOf('.');
    batchParams.videoName = dotIndex > -1 ? fileNameWithExt.substring(0, dotIndex) : fileNameWithExt;

    // 默认输出目录设为原视频所在目录
    batchParams.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');
  }
}

async function selectOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected && typeof selected === 'string') {
    batchParams.outputDir = selected;
  }
}
// 触发后端切割任务
async function runBatchSplit() {
  if (!batchParams.inputPath || !batchParams.outputDir) return;
  isProcessing.value = true;
  resultLogs.value = [];
  isError.value = false;
  isExpanded.value = false; // 每次执行重置折叠状态

  try {
    const result = await invoke<string>('batch_split_by_duration', {
      inputPath: batchParams.inputPath,
      outputDir: batchParams.outputDir,
      videoName: batchParams.videoName,
      segmentDuration: batchParams.segmentDuration
    });
    // 按行拆分后端返回的日志
    resultLogs.value = result.split('\n');
  } catch (error) {
    resultLogs.value = [`❌ 发生错误: ${error}`];
    isError.value = true;
  } finally {
    isProcessing.value = false;
  }
}
const markerParams = reactive({
  videoPath: '',
  videoSrc: '' // 用于 <video> 标签的协议转换路径
});

// 标记列表契约
interface Marker { id: string; startTime: number; endTime: number; label: string; }
const markers = ref<Marker[]>([]);

// 当前正在打轴的草稿
const draftMarker = reactive<{ startTime: number | null, endTime: number | null, label: string }>({
  startTime: null,
  endTime: null,
  label: ''
});

// --- 核心方法 ---
async function selectVideoForMarker() {
  const selected = await open({ multiple: false, filters: [{ name: 'Videos', extensions: ['mp4'] }] });
  if (selected && typeof selected === 'string') {
    markerParams.videoPath = selected;
    // Tauri 出于安全限制，Webview 不能直接读 C:\ 盘。必须用 convertFileSrc 转成 asset:// 协议
    markerParams.videoSrc = convertFileSrc(selected);
    markers.value = []; // 清空旧列表
    saveStatus.value = '';
  }
}

// 监听视频播放时间变化
function onTimeUpdate() {
  if (videoPlayerRef.value) {
    currentTime.value = videoPlayerRef.value.currentTime;
  }
}

function setInPoint() {
  draftMarker.startTime = currentTime.value;
  draftMarker.endTime = null; // 重置出点
}

function setOutPoint() {
  if (draftMarker.startTime !== null && currentTime.value > draftMarker.startTime) {
    draftMarker.endTime = currentTime.value;
  } else {
    alert("出点必须大于入点！");
  }
}

function addMarkerToList() {
  if (draftMarker.startTime !== null && draftMarker.endTime !== null) {
    markers.value.push({
      id: Date.now().toString(),
      startTime: draftMarker.startTime,
      endTime: draftMarker.endTime,
      label: draftMarker.label
    });
    // 重置草稿
    draftMarker.startTime = null; draftMarker.endTime = null; draftMarker.label = '';
  }
}

function removeMarker(index: number) {
  markers.value.splice(index, 1);
}

// 调用后端接口保存 JSON
async function saveMarkersToJSON() {
  try {
    const jsonPath = await invoke<string>('save_markers', {
      videoPath: markerParams.videoPath,
      markers: markers.value
    });
    saveStatus.value = `✅ 标记已成功保存至: ${jsonPath}`;
  } catch (err) {
    saveStatus.value = `❌ 保存失败: ${err}`;
  }
}
</script>

<style scoped>
/* ===== 现代化 UI 样式定义 ===== */
.app-container {
  min-height: 100vh;
  background-color: #f3f4f6;
  font-family: 'Segoe UI', system-ui, sans-serif;
  color: #1f2937;
  padding: 2rem;
}

.header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 2px solid #e5e7eb;
}

.logo {
  font-size: 2.5rem;
}

.title-area h1 {
  margin: 0;
  font-size: 1.8rem;
  color: #111827;
}

.subtitle {
  margin: 0;
  color: #6b7280;
  font-size: 0.9rem;
}

.main-content {
  max-width: 700px;
  margin: 0 auto;
}

.card {
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  overflow: hidden;
}

.card-header {
  background-color: #f9fafb;
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
}

.card-header h2 {
  margin: 0 0 0.5rem 0;
  font-size: 1.25rem;
  color: #374151;
}

.card-header p {
  margin: 0;
  color: #6b7280;
  font-size: 0.9rem;
}

.card-body {
  padding: 1.5rem;
}

.form-group {
  margin-bottom: 1.5rem;
  display: flex;
  flex-direction: column;
}

.form-group label {
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #374151;
  font-size: 0.95rem;
}

.form-group input {
  padding: 0.75rem 1rem;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 1rem;
  transition: all 0.2s;
}

.form-group input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

.hint {
  font-size: 0.8rem;
  color: #9ca3af;
  margin-top: 0.4rem;
}

.primary-btn {
  width: 100%;
  padding: 0.875rem;
  background-color: #2563eb;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 1.1rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 0.5rem;
}

.primary-btn:hover:not(:disabled) {
  background-color: #1d4ed8;
}

.primary-btn:disabled {
  background-color: #9ca3af;
  cursor: not-allowed;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  100% { transform: rotate(360deg); }
}

.result-box {
  margin-top: 1.5rem;
  padding: 1rem;
  border-radius: 8px;
  font-family: monospace;
  font-size: 0.9rem;
  overflow-x: auto;
}

.result-box pre {
  margin: 0;
  white-space: pre-wrap;
}

.success {
  background-color: #f0fdf4;
  border: 1px solid #bbf7d0;
  color: #166534;
}

.error {
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  color: #991b1b;
}

/* 在之前的样式底部追加 */
.input-with-btn {
  display: flex;
  gap: 10px;
}
.input-with-btn input {
  flex: 1;
  background-color: #f9fafb; /* 只读状态的底色 */
  cursor: not-allowed;
}
.secondary-btn {
  padding: 0 1rem;
  background-color: #e5e7eb;
  color: #374151;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
  white-space: nowrap;
}
.secondary-btn:hover {
  background-color: #d1d5db;
}

/* 补充折叠列表的样式 */
.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
  font-weight: bold;
}
.text-btn {
  background: none;
  border: none;
  color: #2563eb;
  cursor: pointer;
  font-size: 0.85rem;
  padding: 0;
}
.text-btn:hover { text-decoration: underline; }
.log-list {
  list-style: none;
  padding: 0;
  margin: 0;
  font-family: monospace;
  font-size: 0.85rem;
  line-height: 1.5;
}
.ellipsis {
  color: #6b7280;
  font-style: italic;
  margin-top: 0.5rem;
}

/* 简化部分样式展示 */
.player-section { margin-top: 1rem; background: #000; border-radius: 8px; overflow: hidden; }
.video-player { width: 100%; max-height: 400px; display: block; }
.controls-bar { background: #1f2937; padding: 10px; display: flex; justify-content: space-between; align-items: center; color: white; }
.mark-btn { background: #4b5563; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; margin-left: 8px; }
.mark-btn:hover { background: #6b7280; }
.draft-marker { padding: 10px; background: #f3f4f6; display: flex; gap: 10px; align-items: center; }
.marker-list-section { margin-top: 1.5rem; }
.marker-list { list-style: none; padding: 0; }
.marker-list li { display: flex; justify-content: space-between; padding: 8px; border-bottom: 1px solid #e5e7eb; align-items: center; }
.tag { font-weight: bold; color: #2563eb; }
.time { color: #6b7280; font-family: monospace; }
.text-btn.danger { color: #dc2626; }
.status-msg { margin-top: 10px; font-weight: bold; color: #166534; }
</style>