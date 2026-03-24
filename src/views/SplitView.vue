<template>
  <div class="view-container">

    <header class="view-header">
      <h1 class="view-title">基础无损分割</h1>
      <p class="view-subtitle">极速流复制技术，无需重新编码，瞬间完成视频切片。</p>
    </header>

    <div class="tabs-header">
      <div class="tabs-nav">
        <button class="tab-btn active">
          <span class="icon">⏱️</span> 固定时长分割
        </button>
        <button class="tab-btn disabled" title="Pro 版本功能，即将推出">
          <span class="icon">🔢</span> 固定数量分割 <span class="pro-tag">PRO</span>
        </button>
      </div>
    </div>

    <div class="workspace-grid">
      <div class="card form-card">
        <div class="card-body">
          <div class="section-title">1. 选择视频源</div>
          <div class="form-group">
            <div class="input-with-btn">
              <input v-model="batchParams.inputPath" type="text" readonly placeholder="请点击右侧按钮选择需要分割的视频..." class="file-input" />
              <button class="secondary-btn" @click="selectInputFile">📁 浏览</button>
            </div>

            <transition name="fade">
              <div class="duration-badge" v-if="videoDurationSec > 0">
                <span class="icon">⏱️</span> 总时长:
                <span class="time-val">
                  {{ showDurationInMinutes ? (videoDurationSec / 60).toFixed(2) : videoDurationSec }}
                </span>
                <button class="text-link" @click="showDurationInMinutes = !showDurationInMinutes">
                  切换为{{ showDurationInMinutes ? '秒' : '分钟' }}
                </button>
              </div>
            </transition>
          </div>

          <hr class="divider" />

          <div class="section-title">2. 输出设置</div>
          <div class="form-group">
            <label>📁 目标保存目录</label>
            <div class="input-with-btn">
              <input v-model="batchParams.outputDir" type="text" readonly placeholder="默认保存在原视频同级目录..." class="file-input" />
              <button class="secondary-btn" @click="selectOutputDir">更改</button>
            </div>
            <div class="path-preview" v-if="batchParams.videoName && batchParams.outputDir">
              预期输出: {{ batchParams.outputDir }}\<strong>{{ batchParams.videoName }}</strong>\...
            </div>
          </div>

          <div class="form-group">
            <label>⏳ 每个片段时长 (秒)</label>
            <div class="duration-input-wrapper">
              <input v-model.number="batchParams.segmentDuration" type="number" min="1" class="number-input" />
              <span class="duration-hint">≈ {{ (batchParams.segmentDuration / 60).toFixed(1) }} 分钟/段</span>
            </div>
          </div>

          <button class="primary-btn mt-4" @click="runBatchSplit" :disabled="!isReadyToProcess || isProcessing">
            <span v-if="isProcessing" class="spinner">⚙️</span>
            {{ isProcessing ? '全速处理中...' : '🚀 开始批量分割' }}
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
              <span v-if="isProcessing">⏳ 正在切割中...</span>
              <span v-else-if="isError">❌ 处理遇到错误</span>
              <span v-else>🎉 任务完成 (共 {{ resultLogs.length }} 段)</span>
            </div>

            <button v-if="!isProcessing && !isError && resultLogs.length > 0"
                    class="secondary-btn small"
                    @click="openTargetFolder">
              📂 打开输出目录
            </button>
          </div>

          <ul class="log-list" v-if="!isError">
            <li v-for="(log, index) in resultLogs" :key="index">
              <span class="check-icon">✓</span> {{ log.replace('✅ 生成: ', '') }}
            </li>
          </ul>
          <div class="error-msg" v-if="isError">{{ resultLogs[0] }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

const isProcessing = ref(false);
const resultLogs = ref<string[]>([]);
const isError = ref(false);

const videoDurationSec = ref(0);
const showDurationInMinutes = ref(true);

const batchParams = reactive({
  inputPath: '', outputDir: '', videoName: '', segmentDuration: 300
});

const isReadyToProcess = computed(() => batchParams.inputPath && batchParams.outputDir && batchParams.segmentDuration > 0);

async function selectInputFile() {
  const selected = await open({ multiple: false, filters: [{ name: 'Videos', extensions: ['mp4', 'mkv', 'mov', 'avi'] }] });
  if (selected && typeof selected === 'string') {
    batchParams.inputPath = selected;
    const pathParts = selected.replace(/\\/g, '/').split('/');
    const fileNameWithExt = pathParts.pop() || '';
    const dotIndex = fileNameWithExt.lastIndexOf('.');
    batchParams.videoName = dotIndex > -1 ? fileNameWithExt.substring(0, dotIndex) : fileNameWithExt;
    batchParams.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

    // 获取视频时长
    try {
      videoDurationSec.value = await invoke<number>('get_video_duration_cmd', { videoPath: selected });
    } catch (e) {
      console.error("无法获取时长", e);
    }
    resultLogs.value = [];
  }
}

async function selectOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected && typeof selected === 'string') batchParams.outputDir = selected;
}

async function openTargetFolder() {
  const targetPath = `${batchParams.outputDir}\\${batchParams.videoName}`;
  try {
    await invoke('open_folder', { path: targetPath });
  } catch (e) {
    console.error(e);
  }
}

async function runBatchSplit() {
  if (!isReadyToProcess.value) return;
  isProcessing.value = true; resultLogs.value = []; isError.value = false;
  try {
    const result = await invoke<string>('batch_split_by_duration', batchParams);
    resultLogs.value = result.split('\n').filter(line => line.trim() !== '');
  } catch (error) {
    resultLogs.value = [`${error}`]; isError.value = true;
  } finally {
    isProcessing.value = false;
  }
}
</script>

<style scoped>
/* *** 修复核心：视图级容器，应用统一的 padding 设计规范 *** */
.view-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 2rem; /* <--- 復用 MarkerWrapperView 的 2rem 间距规范 *** */
  box-sizing: border-box;
  animation: fadeIn 0.3s ease-in-out;
}

/* Page Header (移除原先割裂的 margin-left) */
.view-header { margin-bottom: 1rem; flex-shrink: 0;}
.view-title { font-size: 1.75rem; font-weight: 700; margin: 0 0 0.25rem 0; color: #111827;}
.view-subtitle { color: #6b7280; margin: 0; font-size: 0.95rem; }

/* Tabs Header (復用上一版本中高质量的 Tabs 样式实现完美统一) */
.tabs-header { margin-bottom: 1.5rem; flex-shrink: 0; }
.tabs-nav { display: inline-flex; background: #e5e7eb; padding: 4px; border-radius: 10px; gap: 4px;}
.tab-btn { background: transparent; border: none; padding: 0.6rem 1.2rem; border-radius: 8px; font-weight: 600; color: #4b5563; cursor: pointer; transition: all 0.2s; font-size: 0.95rem;}
.tab-btn.active { background: white; color: #2563eb; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
.tab-btn.disabled { opacity: 0.6; cursor: not-allowed; }
.icon { font-size: 1.1rem; margin-right: 6px; }
.pro-tag { font-size: 0.6rem; background: #f59e0b; color: white; padding: 1px 4px; border-radius: 4px; margin-left: 4px; vertical-align: top;}

/* 双栏网格布局 (復用 Turn 6 的高质量布局) */
.workspace-grid { display: grid; grid-template-columns: 450px 1fr; gap: 1.5rem; flex: 1; min-height: 0; }
.card { background: #fff; border-radius: 12px; border: 1px solid #e5e7eb; overflow: hidden; display: flex; flex-direction: column;}
.form-card .card-body { padding: 1.5rem; overflow-y: auto;}

.section-title { font-size: 1.05rem; font-weight: 600; margin-bottom: 1rem; color: #111827;}
.form-group { margin-bottom: 1.25rem; display: flex; flex-direction: column; }
.form-group label { font-size: 0.85rem; color: #4b5563; margin-bottom: 0.4rem; }
.input-with-btn { display: flex; gap: 0.5rem; }
.file-input { flex: 1; padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 6px; font-size: 0.85rem; background: #f9fafb; }
.secondary-btn { padding: 0.5rem 1rem; background: #fff; border: 1px solid #d1d5db; border-radius: 6px; cursor: pointer; white-space: nowrap; font-weight: 500;}
.secondary-btn.small { padding: 0.3rem 0.8rem; font-size: 0.85rem; }
.secondary-btn:hover { background: #f3f4f6; }
.primary-btn { width: 100%; padding: 0.75rem; background: #2563eb; color: #fff; border: none; border-radius: 6px; font-weight: 600; cursor: pointer; font-size: 1rem; }
.primary-btn:hover { background: #1d4ed8; }
.primary-btn:disabled { background: #9ca3af; }
.mt-4 { margin-top: 1.5rem; }

.duration-badge { margin-top: 0.5rem; font-size: 0.85rem; color: #059669; background: #d1fae5; padding: 0.4rem 0.8rem; border-radius: 6px; display: inline-flex; align-items: center; gap: 0.5rem; width: fit-content;}
.time-val { font-weight: bold; font-family: monospace; font-size: 1rem;}
.text-link { background: none; border: none; color: #2563eb; cursor: pointer; text-decoration: underline; padding: 0;}

.path-preview { margin-top: 0.5rem; font-size: 0.8rem; color: #6b7280; background: #f3f4f6; padding: 0.5rem; border-radius: 6px; word-break: break-all;}
.number-input { width: 120px; padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 6px; }
.duration-input-wrapper { display: flex; align-items: center; gap: 0.5rem; }
.duration-hint { font-size: 0.85rem; color: #6b7280; }
.divider { border: 0; border-top: 1px solid #e5e7eb; margin: 1.5rem 0; }

/* 右侧结果卡片 */
.result-card { background: #f9fafb; }
.empty-state { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; color: #9ca3af; }
.empty-icon { font-size: 2.5rem; margin-bottom: 0.5rem; }
.result-content { display: flex; flex-direction: column; height: 100%; padding: 1.5rem; }
.result-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.75rem;}
.status-title { font-weight: 600; color: #166534; font-size: 1.1rem;}
.has-error .status-title { color: #dc2626; }
.log-list { flex: 1; overflow-y: auto; list-style: none; padding: 0; margin: 0; font-family: monospace; font-size: 0.85rem; color: #374151; }
.log-list li { padding: 0.4rem 0; border-bottom: 1px dashed #e5e7eb; }
.check-icon { color: #10b981; font-weight: bold; margin-right: 4px;}
.error-msg { color: #dc2626; font-family: monospace; white-space: pre-wrap; }
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
</style>