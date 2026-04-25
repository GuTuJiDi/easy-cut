<template>
  <div class="view-container">

    <header class="view-header">
      <h1 class="view-title">基础无损分割</h1>
      <p class="view-subtitle">极速流复制技术，无需重新编码，瞬间完成视频切片。</p>
    </header>

    <div class="tabs-header">
      <div class="tabs-nav">
        <button
            class="tab-btn"
            :class="{ active: splitMode === 'duration' }"
            @click="splitMode = 'duration'"
        >
          <span class="icon">⏱️</span> 固定时长分割
        </button>

        <button
            class="tab-btn"
            :class="{ active: splitMode === 'count', disabled: !authStore.isPro }"
            @click="handleCountTabClick"
            :title="authStore.isPro ? '切换到固定数量分割' : 'PRO 旗舰版专属功能'"
        >
          <span class="icon">🔢</span> 固定数量分割 <span class="pro-tag">PRO</span>
        </button>
      </div>
    </div>

    <div class="workspace-grid">
      <div class="card form-card">

        <div class="form-scroller hide-scrollbar">
          <div class="card-body">
            <div class="section-title">1. 选择视频源</div>

            <div class="form-group source-group">

              <button v-if="!batchParams.inputPath" class="upload-dropzone-btn" @click="selectInputFile">
                <span class="upload-icon">📁</span>
                <span class="upload-text">点击选择待分割视频</span>
              </button>

              <div v-else class="active-file-wrapper">
                <div class="active-file-badge">
                  <span class="file-icon">🎬</span>
                  <span class="file-name" :title="batchParams.inputPath">{{ videoFileName }}</span>
                  <div class="badge-actions">
                    <button class="icon-btn" @click="selectInputFile" title="更换视频">🔄</button>
                    <button class="icon-btn" @click="clearInputFile" title="移除视频">✖</button>
                  </div>
                </div>

                <transition name="fade">
                  <div class="duration-badge" v-if="videoDurationSec > 0">
                    <span class="icon">⏱️</span> 视频总长:
                    <span class="time-val">{{ showDurationInMinutes ? (videoDurationSec / 60).toFixed(2) : videoDurationSec }}</span>
                    <button class="text-link" @click="showDurationInMinutes = !showDurationInMinutes">
                      切换为{{ showDurationInMinutes ? '秒' : '分钟' }}
                    </button>
                  </div>
                </transition>
              </div>
            </div>

            <hr class="divider" />

            <div class="section-title">2. 输出设置</div>
            <div class="form-group">
              <label>📁 目标保存目录</label>
              <div class="input-with-btn">
                <input v-model="batchParams.outputDir" type="text" readonly placeholder="默认保存在原视频同级目录..." class="file-input dir-input" />
                <button class="secondary-btn" @click="selectOutputDir">更改</button>
              </div>

              <div class="path-preview-card" v-if="batchParams.videoName && batchParams.outputDir">
                <div class="preview-header">📂 预期生成示例</div>
                <div class="preview-path" :title="`${batchParams.outputDir}\\${batchParams.videoName}\\[01]_${batchParams.videoName}.mp4`">
                  {{ batchParams.outputDir }}\<strong class="highlight-dir">{{ batchParams.videoName }}</strong>\[01]_{{ batchParams.videoName }}.mp4
                </div>
              </div>
            </div>

            <transition name="fade-slide" mode="out-in">

              <div v-if="splitMode === 'duration'" key="mode-duration" class="form-group strategy-block">
                <label>⏳ 每个片段时长</label>
                <div class="duration-input-wrapper">
                  <div class="input-with-addon">
                    <input v-model.number="displayDuration" type="number" step="0.1" min="0.1" class="number-input" />
                    <button class="addon-btn" @click="toggleDurationUnit">
                      {{ isDurationInMinutes ? '分钟' : '秒' }} 🔃
                    </button>
                  </div>
                  <span class="duration-hint">
                    {{ isDurationInMinutes ? `≈ ${displayDuration * 60} 秒/段` : `≈ ${(displayDuration / 60).toFixed(2)} 分钟/段` }}
                  </span>
                </div>
              </div>

              <div v-else-if="splitMode === 'count'" key="mode-count" class="form-group strategy-block pro-strategy-block">
                <div class="pro-label-group">
                  <label>🔢 等分切割数量</label>
                  <span class="pro-badge-mini">PRO 专属算法</span>
                </div>

                <div class="stepper-wrapper">
                  <button class="stepper-btn" @click="batchParams.segmentCount = Math.max(2, batchParams.segmentCount - 1)">-</button>
                  <input v-model.number="batchParams.segmentCount" type="number" min="2" max="999" class="stepper-input" />
                  <button class="stepper-btn" @click="batchParams.segmentCount++">+</button>
                  <span class="stepper-unit">段</span>
                </div>

                <div class="estimation-box" :class="{ 'ready': videoDurationSec > 0 }">
                  <span class="icon">💡</span>
                  <span v-if="videoDurationSec > 0">
                    智能预估：每段将被切割为约 <strong>{{ estimatedDurationPerSegment }}</strong>
                  </span>
                  <span v-else class="hint-text">请先选择视频源，激活预估。</span>
                </div>
              </div>

            </transition>

          </div>
        </div>

        <div class="form-footer">
          <button class="primary-btn execute-btn" @click="runBatchSplit" :disabled="!isReadyToProcess || isProcessing">
            <span v-if="isProcessing" class="spinner">⚙️</span>
            {{ isProcessing ? '全速引擎切割中...' : '🚀 开始极速分割' }}
          </button>
        </div>
      </div>

      <div class="card result-card">
        <div v-if="resultLogs.length === 0 && !isProcessing" class="empty-state">
          <span class="empty-icon">✂️</span>
          <p>分割任务列表</p>
          <span class="hint">左侧点击执行后，此处将展示生成结果</span>
        </div>

        <div v-else class="result-content" :class="{ 'has-error': isError }">

          <div class="result-header">
            <div class="status-title">
              <span v-if="isProcessing">⏳ 全速切割中...</span>
              <span v-else-if="isError">❌ 处理遇到错误</span>
              <div v-else class="success-header-group">
                <span>🎉 任务完成 (共 {{ parsedLogs.length }} 段)</span>
                <transition name="pop-in">
                  <span v-if="splitCostTime" class="time-badge">
                    ⏱️ 耗时: <strong>{{ splitCostTime }}</strong> 秒
                  </span>
                </transition>
              </div>
            </div>

            <button v-if="!isProcessing && !isError && parsedLogs.length > 0" class="secondary-btn small" @click="openTargetFolder">
              📂 打开输出目录
            </button>
          </div>

          <div class="error-msg hide-scrollbar" v-if="isError">{{ resultLogs[0] }}</div>

          <div class="list-container" v-else>
            <ul class="parsed-log-list hide-scrollbar">
              <li v-for="(item, index) in visibleLogs" :key="index" class="log-item">
                <div class="log-info">
                  <span class="check-icon">✓</span>
                  <span class="log-index">{{ String(item.id).padStart(2, '0') }}</span>
                  <span class="log-filename" :title="item.path">{{ item.filename }}</span>
                  <span class="log-duration">{{ item.durationStr }}</span>
                </div>

                <div class="log-actions">
                  <button class="action-btn split-btn" @click="continueSplit(item.path)" title="将此片段作为源视频，继续细分">
                    <span class="icon">✂️</span> 继续分割
                  </button>
                  <button class="action-btn marker-btn" @click="goToMarker(item.path)" title="前往打轴工作流">
                    <span class="icon">🏷️</span> 打轴标记
                  </button>
                  <button class="action-icon play-btn" @click="playVideoWithSystem(item.path)" title="调用系统播放器预览">
                    ▶
                  </button>
                </div>
              </li>
            </ul>

            <div class="list-toggle" v-if="parsedLogs.length > 6">
              <button class="toggle-btn" @click="isListExpanded = !isListExpanded">
                {{ isListExpanded ? '▲ 收起列表' : `▼ 展开其余 ${parsedLogs.length - 6} 个片段` }}
              </button>
            </div>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useRouter,useRoute } from 'vue-router';
import { handleSecurityBreach } from '../utils/securityGuard';
import { useSettingsStore } from '../stores/settings';
import { useAuthStore } from '../stores/auth';
import { showToast } from '../utils/toast';
const settingsStore = useSettingsStore();
const authStore = useAuthStore();
const router = useRouter();
const route = useRoute();
// 🌟 核心修复：监听从其他页面（如打轴页面）传来的跨界跳转加载请求
watch(
    () => route.query.loadVideo,
    async (newPath) => {
      // 领地意识：只在基础分割页面才响应
      // 假设基础分割的路由是 '/' 或 '/split'
      if (!route.path.includes('/split') && route.path !== '/') return;

      if (newPath && typeof newPath === 'string' && batchParams.value.inputPath !== newPath) {
        batchParams.value.inputPath = newPath;
        const pathParts = newPath.replace(/\\/g, '/').split('/');
        const fileNameWithExt = pathParts.pop() || '';
        const dotIndex = fileNameWithExt.lastIndexOf('.');

        batchParams.value.videoName = (dotIndex > -1 ? fileNameWithExt.substring(0, dotIndex) : fileNameWithExt).trim();
        batchParams.value.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

        try {
          videoDurationSec.value = await invoke<number>('get_video_duration_cmd', { videoPath: newPath });
        } catch (e) {
          // 1. 优先执行安全熔断检查
          if (await handleSecurityBreach(e)) return;

          // 2. 如果不是安全问题，走正常业务报错弹窗
          showToast(`任务失败: ${e}`, 'error');
          console.error("无法获取新片段时长", e);
        }

        resultLogs.value = [];
        splitCostTime.value = '';
      }
    },
    { immediate: true }
);

const isProcessing = ref(false);
const processLogs = ref('');
const resultLogs = ref<string[]>([]);
const isError = ref(false);

const videoDurationSec = ref(0);
const showDurationInMinutes = ref(true);

const splitMode = ref<'duration' | 'count'>('duration');

const batchParams = ref({
  inputPath: '',
  outputDir: '',
  videoName: '输出视频',
  segmentDuration: 30,
  segmentCount: 5
});

const displayDuration = ref(5);
const isDurationInMinutes = ref(true);
const splitCostTime = ref('');
const isListExpanded = ref(false);

const parsedLogs = computed(() => {
  if (resultLogs.value.length === 0 || isError.value) return [];
  return resultLogs.value.map((log, index) => {
    const cleanPath = log.replace('✅ 生成: ', '').trim();
    const filename = cleanPath.split(/[/\\]/).pop() || cleanPath;

    let segDur = 0;

    if (splitMode.value === 'duration') {
      segDur = batchParams.value.segmentDuration;
      if (index === resultLogs.value.length - 1 && videoDurationSec.value > 0) {
        const remainder = videoDurationSec.value % batchParams.value.segmentDuration;
        if (remainder > 0) segDur = remainder;
      }
    } else if (splitMode.value === 'count') {
      segDur = videoDurationSec.value > 0 ? (videoDurationSec.value / batchParams.value.segmentCount) : 0;
    }

    const durationStr = formatDuration(segDur);
    return { id: index + 1, filename, path: cleanPath, durationStr };
  });
});

const visibleLogs = computed(() => {
  if (isListExpanded.value) return parsedLogs.value;
  return parsedLogs.value.slice(0, 6);
});

watch([displayDuration, isDurationInMinutes], () => {
  if (displayDuration.value > 0) {
    batchParams.value.segmentDuration = isDurationInMinutes.value ? displayDuration.value * 60 : displayDuration.value;
  }
}, { immediate: true });

function toggleDurationUnit() {
  isDurationInMinutes.value = !isDurationInMinutes.value;
  if (isDurationInMinutes.value) {
    displayDuration.value = Number((displayDuration.value / 60).toFixed(2));
  } else {
    displayDuration.value = Math.round(displayDuration.value * 60);
  }
}

const estimatedDurationPerSegment = computed(() => {
  if (videoDurationSec.value <= 0 || batchParams.value.segmentCount <= 0) return '00:00';
  const avgSec = videoDurationSec.value / batchParams.value.segmentCount;
  return formatDuration(avgSec);
});

const videoFileName = computed(() => {
  if (!batchParams.value.inputPath) return '';
  return batchParams.value.inputPath.split(/[/\\]/).pop() || '';
});

const isReadyToProcess = computed(() =>
    batchParams.value.inputPath &&
    batchParams.value.outputDir &&
    batchParams.value.segmentDuration > 0
);

async function selectInputFile() {
  const selected = await open({ multiple: false, filters: [{ name: 'Videos', extensions: ['mp4', 'mkv', 'mov', 'avi'] }] });
  if (selected && typeof selected === 'string') {
    batchParams.value.inputPath = selected;
    const pathParts = selected.replace(/\\/g, '/').split('/');
    const fileNameWithExt = pathParts.pop() || '';
    const dotIndex = fileNameWithExt.lastIndexOf('.');

    batchParams.value.videoName = (dotIndex > -1 ? fileNameWithExt.substring(0, dotIndex) : fileNameWithExt).trim();
    batchParams.value.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

    try {
      videoDurationSec.value = await invoke<number>('get_video_duration_cmd', { videoPath: selected });
    } catch (e) { console.error("无法获取时长", e); }
    resultLogs.value = [];
  }
}

function clearInputFile() {
  batchParams.value.inputPath = '';
  batchParams.value.outputDir = '';
  batchParams.value.videoName = '';
  videoDurationSec.value = 0;
  resultLogs.value = [];
}

async function selectOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected && typeof selected === 'string') batchParams.value.outputDir = selected;
}

async function openTargetFolder() {
  const targetPath = `${batchParams.value.outputDir}\\${batchParams.value.videoName}`;
  try { await invoke('open_folder', { path: targetPath }); } catch (e) { console.error(e); }
}

async function runBatchSplit() {
  if (!batchParams.value.inputPath || !batchParams.value.outputDir) {
    alert("请完整填写输入路径和输出目录！");
    return;
  }

  isProcessing.value = true;
  isError.value = false;
  splitCostTime.value = '';
  isListExpanded.value = false;
  processLogs.value = "🚀 正在启动多媒体并发引擎，榨干系统性能中...\n";
  const startTime = performance.now();

  try {
    let logsString = "";

    if (splitMode.value === 'duration') {
      logsString = await invoke<string>('batch_split_by_duration', {
        params: {
          input_path: batchParams.value.inputPath,
          output_dir: batchParams.value.outputDir,
          video_name: batchParams.value.videoName,
          segment_duration: Number(batchParams.value.segmentDuration),
          max_concurrent_tasks: settingsStore.maxConcurrentTasks
        }
      });
    }
    else if (splitMode.value === 'count') {
      logsString = await invoke<string>('batch_split_by_count', {
        params: {
          input_path: batchParams.value.inputPath,
          output_dir: batchParams.value.outputDir,
          video_name: batchParams.value.videoName,
          segment_count: Number(batchParams.value.segmentCount),
          max_concurrent_tasks: settingsStore.maxConcurrentTasks,
          // license_str: authStore.licenseKey
        },
        sessionToken: authStore.sessionToken
      });
    }

    const timeSpent = ((performance.now() - startTime) / 1000).toFixed(2);
    splitCostTime.value = timeSpent;
    resultLogs.value = logsString.split('\n').filter(line => line.trim() !== '');

  } catch (error) {
    resultLogs.value = [`${error}`];
    isError.value = true;
    console.error("切割失败:", error);
    // 🌟 加上弹窗，让 Rust 的报错直接显示在屏幕上！
    alert(`任务失败被拦截: ${error}`);
    if (String(error).includes("核心授权拦截")) {
      alert("该功能为 PRO 旗舰版专属，请前往全局设置绑定设备授权！");
    }
  } finally {
    isProcessing.value = false;
  }
}

async function playVideoWithSystem(filePath: string) {
  try { await invoke('open_file', { path: filePath }); } catch (e) { console.error("播放失败", e); }
}

function formatDuration(seconds: number): string {
  if (!seconds || isNaN(seconds)) return '00:00';
  const m = Math.floor(seconds / 60).toString().padStart(2, '0');
  const s = Math.floor(seconds % 60).toString().padStart(2, '0');
  return `${m}:${s}`;
}

async function continueSplit(targetPath: string) {
  batchParams.value.inputPath = targetPath;
  const pathParts = targetPath.replace(/\\/g, '/').split('/');
  const fileNameWithExt = pathParts.pop() || '';
  const dotIndex = fileNameWithExt.lastIndexOf('.');

  batchParams.value.videoName = dotIndex > -1 ? fileNameWithExt.substring(0, dotIndex) : fileNameWithExt;
  batchParams.value.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

  try {
    videoDurationSec.value = await invoke<number>('get_video_duration_cmd', { videoPath: targetPath });
  } catch (e) { console.error("无法获取时长", e); }

  resultLogs.value = [];
  isListExpanded.value = false;
  splitCostTime.value = '';
}

function goToMarker(targetPath: string) {
  router.push({ path: '/marker', query: { loadVideo: targetPath } });
}

function handleCountTabClick() {
  if (authStore.isPro) {
    splitMode.value = 'count';
  } else {
    alert('此功能为 PRO 旗舰版独占，请先前往【全局偏好设置】激活！');
  }
}
</script>

<style scoped>
/* ========================================= */
/* 🌟 核心：隐藏原生外观，保留逻辑滚动能力 */
/* ========================================= */
.hide-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
.hide-scrollbar::-webkit-scrollbar {
  display: none;
}

.view-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  padding: 1.2rem;
  box-sizing: border-box;
}

.view-header {
  margin-bottom: 0.8rem;
}
.view-title {
  margin: 0 0 0.4rem 0;
}
.view-subtitle {
  margin: 0;
  color: #6b7280;
  font-size: 0.9rem;
}

.tabs-header {
  margin-bottom: 0.8rem;
  flex-shrink: 0;
}

.tabs-nav {
  display: flex;
  gap: 10px;
}

.tab-btn {
  padding: 0.5rem 1rem;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  background: white;
  color: #374151;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s;
}

.tab-btn.active {
  background: #f1f5f9;
  border-color: #3b82f6;
  color: #2563eb;
  box-shadow: 0 1px 3px rgba(0,0,0,0.05);
}

.tab-btn.disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.pro-tag {
  font-size: 0.7rem;
  background: #9333ea;
  color: white;
  padding: 2px 6px;
  border-radius: 4px;
}

/* ========================================= */
/* 🌟 网格布局与高度限制 */
/* ========================================= */
.workspace-grid {
  display: grid;
  grid-template-columns: minmax(320px, 450px) 1fr;
  gap: 1.2rem;
  flex: 1;
  min-height: 0;
}

.form-card {
  display: flex;
  flex-direction: column;
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
  border: 1px solid #e5e7eb;
  overflow: hidden;
}

/* 🌟 核心修复 2：允许自动滚动。避免在极端缩小窗口时下方内容被物理隐形 */
.form-scroller {
  flex: 1;
  min-height: 0;
  overflow-y: auto; /* 改为 auto，配合上面的 hide-scrollbar 类，实现可滚动但看不见滚动条 */
}

.card-body { padding: 1.2rem 1.2rem 0.5rem 1.2rem; }

.form-footer {
  padding: 0.8rem 1.2rem;
  background: white;
  border-top: 1px solid #e5e7eb;
  flex-shrink: 0;
}

.execute-btn {
  width: 100%;
  padding: 0.8rem;
  font-size: 1.05rem;
  justify-content: center;
}

.section-title { font-size: 1.05rem; font-weight: 600; margin-bottom: 0.8rem; color: #111827;}
.form-group { margin-bottom: 0.8rem; display: flex; flex-direction: column; }
.form-group label { font-size: 0.85rem; color: #4b5563; margin-bottom: 0.4rem; font-weight: 500;}

/* 🌟 核心修复 1：改用 min-height，防跳动且允许自适应缩放撑开 */
.source-group {
  min-height: 116px;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  margin-bottom: 0.5rem;
}

.upload-dropzone-btn {
  width: 100%;
  flex: 1; /* 配合父级 min-height 自然撑开 */
  min-height: 100px;
  background: #f8fafc;
  border: 2px dashed #cbd5e1;
  border-radius: 8px;
  color: #64748b;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: all 0.2s ease;
}
.upload-dropzone-btn:hover {
  background: #eff6ff;
  border-color: #3b82f6;
}
.upload-dropzone-btn .upload-icon {
  font-size: 2rem;
}
.upload-dropzone-btn .upload-text {
  font-size: 0.95rem;
  font-weight: 600;
  color: #3b82f6;
}

.active-file-wrapper {
  display: flex;
  flex-direction: column;
}

/* ========================================= */
/* 🌟 其他优美样式维持原样 */
/* ========================================= */
.success-header-group { display: flex; align-items: center; gap: 0.75rem; }
.time-badge { background: #ecfdf5; color: #059669; padding: 2px 10px; border-radius: 20px; font-size: 0.8rem; font-weight: 500; border: 1px solid #a7f3d0; display: inline-flex; align-items: center; gap: 4px; }
.time-badge strong { font-size: 0.95rem; font-family: 'Consolas', monospace; font-weight: 800; }

.list-container { display: flex; flex-direction: column; flex: 1; overflow: hidden; }
.parsed-log-list { flex: 1; overflow-y: auto; list-style: none; padding: 0; margin: 0; padding-right: 5px;}
.log-item { display: flex; justify-content: space-between; align-items: center; padding: 0.6rem 0.5rem; border-bottom: 1px solid #f1f5f9; transition: all 0.2s; border-radius: 6px;}
.log-item:hover { background: #f8fafc; border-color: transparent; box-shadow: 0 1px 3px rgba(0,0,0,0.05); }

.log-info { display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0; }

.log-index {
  font-family: 'Consolas', monospace;
  font-size: 0.85rem;
  color: #94a3b8;
  font-weight: 600;
  margin-right: 2px;
}

.log-filename { font-family: 'Consolas', monospace; font-size: 0.85rem; color: #334155; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 600; cursor: default;}

.log-actions { display: flex; gap: 4px; opacity: 0; transition: opacity 0.2s; }
.log-item:hover .log-actions { opacity: 1; }

.action-icon { background: white; border: 1px solid #e2e8f0; color: #64748b; font-size: 0.9rem; padding: 4px 8px; border-radius: 6px; cursor: pointer; transition: all 0.2s; display: flex; align-items: center; justify-content: center; }
.action-icon:hover { border-color: #3b82f6; color: #2563eb; background: #eff6ff; }
.play-btn { color: #10b981; }
.play-btn:hover { border-color: #10b981; color: #059669; background: #ecfdf5; }

.list-toggle { margin-top: 10px; text-align: center; padding-top: 10px; border-top: 1px dashed #e2e8f0; }
.toggle-btn { background: transparent; border: none; color: #64748b; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: color 0.2s; padding: 5px 15px; border-radius: 20px;}
.toggle-btn:hover { color: #2563eb; background: #f1f5f9; }

.pop-in-enter-active { animation: popIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275); }
@keyframes popIn { 0% { transform: scale(0.8); opacity: 0; } 100% { transform: scale(1); opacity: 1; } }

.active-file-badge { display: flex; align-items: center; background: #f8fafc; padding: 0.5rem 0.75rem; border-radius: 8px; gap: 0.75rem; border: 1px solid #cbd5e1; }
.file-icon { font-size: 1.4rem; }
.file-name { font-weight: 600; color: #334155; font-size: 0.95rem; flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: 'Consolas', monospace;}
.badge-actions { display: flex; gap: 0.25rem; border-left: 1px solid #cbd5e1; padding-left: 0.5rem;}
.dir-input { font-family: 'Consolas', monospace; color: #475569; }

.path-preview-card { margin-top: 0.75rem; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; overflow: hidden; }
.preview-header { font-size: 0.75rem; background: #e2e8f0; color: #475569; padding: 4px 8px; font-weight: bold; }
.preview-path { padding: 8px; font-family: 'Consolas', monospace; font-size: 0.8rem; color: #64748b; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; direction: rtl; text-align: left; }
.highlight-dir { color: #2563eb; font-weight: bold; }

.duration-input-wrapper { display: flex; align-items: center; gap: 0.75rem; }
.input-with-addon { display: flex; border: 1px solid #d1d5db; border-radius: 6px; overflow: hidden; width: 160px; transition: border-color 0.2s;}
.input-with-addon:focus-within { border-color: #3b82f6; box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1); }
.input-with-addon .number-input { border: none; padding: 0.5rem; flex: 1; min-width: 0; outline: none; font-weight: bold; color: #1e293b;}
.addon-btn { background: #f1f5f9; border: none; border-left: 1px solid #d1d5db; padding: 0 10px; color: #475569; font-weight: bold; cursor: pointer; transition: background 0.2s; white-space: nowrap; font-size: 0.85rem;}
.addon-btn:hover { background: #e2e8f0; color: #2563eb; }

.duration-hint { font-size: 0.85rem; color: #64748b; }
.duration-badge { margin-top: 0.5rem; font-size: 0.85rem; color: #059669; background: #d1fae5; padding: 0.4rem 0.8rem; border-radius: 6px; display: inline-flex; align-items: center; gap: 0.5rem; width: fit-content;}
.time-val { font-weight: bold; font-family: monospace; font-size: 1rem;}
.text-link { background: none; border: none; color: #2563eb; cursor: pointer; text-decoration: underline; padding: 0;}
.divider { border: 0; border-top: 1px dashed #e2e8f0; margin: 0.8rem 0; }

.result-card { background: #f9fafb; border-radius: 12px; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05); border: 1px solid #e5e7eb; display: flex; flex-direction: column; overflow: hidden;}
.empty-state { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; color: #94a3b8; }
.empty-icon { font-size: 3rem; margin-bottom: 0.5rem; opacity: 0.5;}
.result-content { display: flex; flex-direction: column; height: 100%; padding: 1.5rem; }
.result-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.75rem;}
.status-title { font-weight: 700; color: #166534; font-size: 1.1rem;}
.has-error .status-title { color: #dc2626; }
.log-list { flex: 1; overflow-y: auto; list-style: none; padding: 0; margin: 0; font-family: 'Consolas', monospace; font-size: 0.85rem; color: #334155; }
.log-list li { padding: 0.5rem 0; border-bottom: 1px dashed #e2e8f0; }
.check-icon { color: #10b981; font-weight: bold; margin-right: 6px;}
.error-msg { color: #dc2626; font-family: monospace; white-space: pre-wrap; }

.primary-btn { padding: 0.6rem 1.5rem; background: #2563eb; color: white; border: none; border-radius: 8px; font-weight: 600; cursor: pointer; transition: all 0.2s;}
.primary-btn:hover:not(:disabled) { background: #1d4ed8; }
.primary-btn:disabled { background: #9ca3af; cursor: not-allowed; }
.secondary-btn { padding: 0.6rem 1rem; background: white; border: 1px solid #d1d5db; border-radius: 8px; color: #374151; font-weight: 600; cursor: pointer; transition: all 0.2s; }
.secondary-btn:hover { background: #f3f4f6; }
.secondary-btn.small { padding: 0.4rem 0.8rem; font-size: 0.85rem;}

.input-with-btn { display: flex; gap: 8px; }
.file-input { flex: 1; padding: 0.6rem 1rem; border: 1px solid #d1d5db; border-radius: 8px; background: #f9fafb; color: #4b5563; outline: none; }

@media (max-width: 950px) {
  .workspace-grid { grid-template-columns: 1fr; min-height: auto; overflow-y: visible; }
  .form-card, .result-card { min-height: auto; flex-shrink: 0; }
}

.log-duration { font-family: 'Consolas', monospace; font-size: 0.75rem; color: #64748b; background: #f1f5f9; padding: 2px 6px; border-radius: 4px; margin-left: 6px; font-weight: 500; border: 1px solid #e2e8f0;}
.action-btn { display: flex; align-items: center; gap: 4px; background: white; border: 1px solid #e2e8f0; color: #64748b; font-size: 0.8rem; font-weight: 600; padding: 4px 10px; border-radius: 6px; cursor: pointer; transition: all 0.2s;}
.split-btn:hover { border-color: #8b5cf6; color: #7c3aed; background: #f5f3ff;}
.marker-btn:hover { border-color: #f59e0b; color: #d97706; background: #fffbeb;}

.fade-slide-enter-active, .fade-slide-leave-active { transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1); }
.fade-slide-enter-from { opacity: 0; transform: translateY(-10px); }
.fade-slide-leave-to { opacity: 0; transform: translateY(10px); }
.strategy-block { padding: 0.2rem 0; }

.pro-strategy-block { background: linear-gradient(to right, #faf5ff, #ffffff); padding: 1rem 1.2rem; border-radius: 10px; border: 1px dashed #d8b4fe; box-shadow: inset 0 2px 4px rgba(0,0,0,0.02);}
.pro-label-group { display: flex; align-items: center; gap: 8px; margin-bottom: 0.8rem; }
.pro-label-group label { margin: 0; color: #581c87; font-weight: 700; }
.pro-badge-mini { background: #9333ea; color: white; font-size: 0.65rem; padding: 2px 6px; border-radius: 4px; font-weight: bold; letter-spacing: 0.5px;}

.stepper-wrapper { display: inline-flex; align-items: center; background: white; border: 1px solid #cbd5e1; border-radius: 8px; overflow: hidden; height: 38px;}
.stepper-btn { background: #f8fafc; border: none; width: 38px; height: 100%; font-size: 1.2rem; color: #475569; cursor: pointer; transition: all 0.2s;}
.stepper-btn:hover { background: #e2e8f0; color: #2563eb; }
.stepper-input { width: 60px; height: 100%; border: none; border-left: 1px solid #e2e8f0; border-right: 1px solid #e2e8f0; text-align: center; font-size: 1.05rem; font-weight: bold; color: #1e293b; outline: none; -moz-appearance: textfield;}
.stepper-input::-webkit-outer-spin-button, .stepper-input::-webkit-inner-spin-button { -webkit-appearance: none; margin: 0; }
.stepper-unit { padding: 0 12px; font-size: 0.9rem; color: #64748b; font-weight: 500; background: #f8fafc;}

.estimation-box { margin-top: 1rem; font-size: 0.85rem; color: #94a3b8; display: flex; align-items: center; gap: 6px; padding-left: 2px;}
.estimation-box.ready { color: #7e22ce; }
.estimation-box strong { font-family: 'Consolas', monospace; font-size: 1rem; background: #f3e8ff; padding: 2px 6px; border-radius: 4px; border: 1px solid #e9d5ff; color: #9333ea;}
</style>