<template>
  <div class="view-container">
    <header class="view-header">
      <h1 class="view-title">音频极速分离 <span class="pro-badge-title">PRO</span></h1>
      <p class="view-subtitle">基于 FFprobe 探针技术，智能嗅探并瞬间剥离原始音轨 (0二次编码损失)。</p>
    </header>

    <div class="workspace-grid">
      <div class="card form-card">
        <div class="form-scroller hide-scrollbar">
          <div class="card-body">
            <div class="section-title">1. 选择视频源</div>
            <div class="form-group source-group">
              <button v-if="!params.inputPath" class="upload-dropzone-btn pro-theme" @click="selectInputFile">
                <span class="upload-icon">🎵</span>
                <span class="upload-text">点击选择待提取视频</span>
              </button>

              <div v-else class="active-file-wrapper">
                <div class="active-file-badge">
                  <span class="file-icon">🎬</span>
                  <span class="file-name" :title="params.inputPath">{{ videoFileName }}</span>
                  <div class="badge-actions">
                    <button class="icon-btn" @click="selectInputFile" title="更换视频">🔄</button>
                    <button class="icon-btn" @click="clearInputFile" title="移除视频">✖</button>
                  </div>
                </div>
                <transition name="fade">
                  <div class="duration-badge" v-if="videoDurationSec > 0">
                    <span class="icon">⏱️</span> 视频总长: <span class="time-val">{{ formatDuration(videoDurationSec) }}</span>
                  </div>
                </transition>
              </div>
            </div>

            <hr class="divider" />

            <div class="section-title">2. 提取设置</div>
            <div class="form-group">
              <label>📁 保存目录</label>
              <div class="input-with-btn">
                <input v-model="params.outputDir" type="text" readonly placeholder="默认保存在原视频目录..." class="file-input dir-input" />
                <button class="secondary-btn" @click="selectOutputDir">更改</button>
              </div>
            </div>

            <div class="form-group">
              <label>🧬 目标封装格式</label>
              <div class="probe-badge">
                <span class="probe-icon">⚡</span>
                <span class="probe-text">自动探测原轨格式 (如 .m4a / .ac3)，确保绝对无损。</span>
              </div>
            </div>

            <div class="form-group strategy-block pro-strategy-block">
              <div class="pro-label-group">
                <label>⏱️ 截取范围 (默认全片)</label>
              </div>
              <div class="time-range-row">
                <div class="input-box">
                  <span class="prefix">起点(s)</span>
                  <input v-model.number="params.startTime" type="number" min="0" class="number-input" />
                </div>
                <span class="range-arrow">➔</span>
                <div class="input-box">
                  <span class="prefix">时长(s)</span>
                  <input v-model.number="params.duration" type="number" min="0.1" class="number-input" />
                </div>
              </div>
            </div>

          </div>
        </div>

        <div class="form-footer action-footer">
          <button
              v-if="!isProcessing"
              class="primary-btn execute-btn pro-execute"
              @click="runExtract"  :disabled="!isReady"
          >
            🚀 开始物理级分离 </button>

          <button
              v-else
              class="execute-btn danger-execute"
              @click="cancelTask"
          >
            <span class="spinner">⚙️</span>
            🛑 停止当前任务
          </button>
        </div>
      </div>

      <div class="card result-card">
        <div v-if="!resultLog && !isProcessing" class="empty-state">
          <span class="empty-icon">🎧</span>
          <p>提取结果将在此展示</p>
        </div>

        <div v-else class="result-content" :class="{ 'has-error': isError }">
          <div class="result-header">
            <div class="status-title">
              <span v-if="isProcessing">⏳ 无损抽离中 (通常 1~3 秒)...</span>
              <span v-else-if="isError">❌ 提取失败</span>
              <span v-else>🎉 提取成功</span>
            </div>
            <button v-if="!isProcessing && !isError && resultLog" class="secondary-btn small" @click="openTargetFolder">
              📂 打开目录
            </button>
          </div>

          <div class="error-msg hide-scrollbar" v-if="isError">{{ resultLog }}</div>

          <div class="success-box" v-else-if="!isProcessing && resultLog">
            <div class="log-info">
              <span class="check-icon">✓</span>
              <span class="log-filename" :title="resultLog">{{ extractedFileName }}</span>
            </div>
            <div class="quick-actions" style="margin-top: 15px;">
              <button class="action-btn play-btn" @click="playResult" title="调用系统播放器预览">
                ▶ 播放试听
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
import { useRoute } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const authStore = useAuthStore();
const route = useRoute();

const isProcessing = ref(false);
const isError = ref(false);
const resultLog = ref('');
const videoDurationSec = ref(0);

// 🌟 净化：去掉了 bitrate 和 outputName
const params = ref({
  inputPath: '',
  outputDir: '',
  startTime: 0,
  duration: 0
});

watch(
    () => route.query.loadVideo,
    async (newPath) => {
      if (!route.path.includes('/audio')) return;

      if (newPath && typeof newPath === 'string' && params.value.inputPath !== newPath) {
        params.value.inputPath = newPath;
        const pathParts = newPath.replace(/\\/g, '/').split('/');
        pathParts.pop();
        params.value.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

        try {
          videoDurationSec.value = await invoke<number>('get_video_duration_cmd', { videoPath: newPath });
          params.value.startTime = 0;
          params.value.duration = Math.ceil(videoDurationSec.value);
        } catch (e) {
          console.error("无法获取时长", e);
        }
        resultLog.value = '';
      }
    },
    { immediate: true }
);

const videoFileName = computed(() => params.value.inputPath.split(/[/\\]/).pop() || '');
const extractedFileName = computed(() => resultLog.value.replace('✅ 生成: ', '').split(/[/\\]/).pop() || '');
const isReady = computed(() => params.value.inputPath && params.value.outputDir && params.value.duration > 0);

function formatDuration(seconds: number): string {
  if (!seconds || isNaN(seconds)) return '00:00';
  const m = Math.floor(seconds / 60).toString().padStart(2, '0');
  const s = Math.floor(seconds % 60).toString().padStart(2, '0');
  return `${m}:${s}`;
}

async function cancelTask() {
  try {
    await invoke('cancel_active_tasks');
  } catch (e) {
    console.error("发送取消指令失败:", e);
  }
}

async function selectInputFile() {
  const selected = await open({ multiple: false, filters: [{ name: 'Videos', extensions: ['mp4', 'mkv', 'mov', 'avi'] }] });
  if (selected && typeof selected === 'string') {
    params.value.inputPath = selected;
    const pathParts = selected.replace(/\\/g, '/').split('/');
    pathParts.pop();
    params.value.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

    try {
      videoDurationSec.value = await invoke<number>('get_video_duration_cmd', { videoPath: selected });
      params.value.startTime = 0;
      params.value.duration = Math.ceil(videoDurationSec.value);
    } catch (e) { console.error("无法获取时长", e); }
    resultLog.value = '';
  }
}

function clearInputFile() {
  params.value.inputPath = ''; params.value.outputDir = '';
  videoDurationSec.value = 0; params.value.duration = 0; resultLog.value = '';
}

async function selectOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected && typeof selected === 'string') params.value.outputDir = selected;
}

async function runExtract() {
  if (!authStore.isPro) {
    alert("🔒 该功能为 PRO 专业版独占，请先在全局设置中激活！");
    return;
  }

  isProcessing.value = true;
  isError.value = false;
  resultLog.value = '';

  try {
    // 🌟 API 载荷对齐：只发送必要的参数
    const res = await invoke<string>('execute_audio_extract', {
      params: {
        input_path: params.value.inputPath,
        output_dir: params.value.outputDir,
        start_time: params.value.startTime,
        duration: params.value.duration,
        // license_str: authStore.licenseKey
      },
      sessionToken: authStore.sessionToken,
    });
    resultLog.value = res;
  }catch (error) {
    const errMsg = String(error);
    if (errMsg.includes('主动取消')) {
      resultLog.value = '🛑 任务已取消';
      isError.value = false;
    } else {
      resultLog.value = errMsg;
      isError.value = true;
      if (errMsg.includes("拦截")) alert(errMsg);
    }
  } finally {
    isProcessing.value = false;
  }
}

async function openTargetFolder() {
  try { await invoke('open_folder', { path: params.value.outputDir }); } catch (e) { console.error(e); }
}
async function playResult() {
  const target = resultLog.value.replace('✅ 生成: ', '').trim();
  try { await invoke('open_file', { path: target }); } catch (e) { console.error(e); }
}
</script>

<style scoped>
@import '../assets/global.css';

/* ... (保留原有的通用样式) ... */
.hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
.hide-scrollbar::-webkit-scrollbar { display: none; }
.view-container { display: flex; flex-direction: column; height: 100vh; overflow: hidden; padding: 1.2rem; box-sizing: border-box; background: #f8fafc;}
.view-header { margin-bottom: 0.8rem; }
.view-title { margin: 0 0 0.4rem 0; display: flex; align-items: center; gap: 10px;}
.view-subtitle { margin: 0; color: #6b7280; font-size: 0.9rem; }
.pro-badge-title { background: linear-gradient(135deg, #a855f7, #7e22ce); color: white; font-size: 0.8rem; padding: 4px 8px; border-radius: 6px; font-weight: 900;}

.workspace-grid { display: grid; grid-template-columns: minmax(320px, 450px) 1fr; gap: 1.2rem; flex: 1; min-height: 0; }
.card { display: flex; flex-direction: column; background: white; border-radius: 12px; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05); border: 1px solid #e5e7eb; overflow: hidden; }
.form-scroller { flex: 1; min-height: 0; overflow-y: auto; }
.card-body { padding: 1.2rem 1.2rem 0.5rem 1.2rem; }
.form-footer { padding: 0.8rem 1.2rem; background: white; border-top: 1px solid #e5e7eb; flex-shrink: 0; }

.section-title { font-size: 1.05rem; font-weight: 600; margin-bottom: 0.8rem; color: #111827;}
.form-group { margin-bottom: 1rem; display: flex; flex-direction: column; }
.form-group label { font-size: 0.85rem; color: #4b5563; margin-bottom: 0.4rem; font-weight: 600;}

.upload-dropzone-btn { width: 100%; min-height: 116px; background: #f8fafc; border: 2px dashed #cbd5e1; border-radius: 8px; cursor: pointer; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; transition: all 0.2s ease;}
.upload-dropzone-btn.pro-theme:hover { background: #faf5ff; border-color: #a855f7; }
.upload-dropzone-btn.pro-theme .upload-text { color: #9333ea; font-weight: 700;}

.active-file-wrapper { display: flex; flex-direction: column; min-height: 116px;}
.active-file-badge { display: flex; align-items: center; background: #f8fafc; padding: 0.5rem 0.75rem; border-radius: 8px; gap: 0.75rem; border: 1px solid #cbd5e1; margin-bottom: 8px;}
.file-icon { font-size: 1.4rem; }
.file-name { font-weight: 600; color: #334155; font-size: 0.9rem; flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: monospace;}
.badge-actions { display: flex; gap: 0.25rem; border-left: 1px solid #cbd5e1; padding-left: 0.5rem;}
.icon-btn { background: transparent; border: none; color: #64748b; cursor: pointer; transition: 0.2s;}

.input-with-btn { display: flex; gap: 8px; }
.file-input { flex: 1; padding: 0.6rem 1rem; border: 1px solid #d1d5db; border-radius: 8px; background: #f9fafb; color: #4b5563; outline: none; font-family: monospace;}

/* 🌟 新增：智能探针 UI 样式 */
.probe-badge { display: flex; align-items: center; gap: 10px; background: #ecfeff; border: 1px solid #a5f3fc; padding: 10px 12px; border-radius: 8px; }
.probe-icon { font-size: 1.2rem; }
.probe-text { font-size: 0.85rem; color: #0e7490; font-weight: 600; }

.pro-strategy-block { background: linear-gradient(to right, #faf5ff, #ffffff); padding: 1rem 1.2rem; border-radius: 10px; border: 1px dashed #d8b4fe; }
.time-range-row { display: flex; align-items: center; gap: 10px;}
.input-box { display: flex; border: 1px solid #cbd5e1; border-radius: 6px; overflow: hidden; background: white;}
.input-box .prefix { background: #f1f5f9; padding: 6px 10px; font-size: 0.8rem; color: #475569; font-weight: 600; border-right: 1px solid #cbd5e1;}
.input-box input { width: 70px; border: none; outline: none; text-align: center; font-family: monospace; font-weight: bold;}
.range-arrow { color: #94a3b8;}

.execute-btn { width: 100%; padding: 0.8rem; font-size: 1.05rem; justify-content: center; border: none; border-radius: 8px; color: white; font-weight: bold; cursor: pointer; transition: 0.2s;}
.pro-execute { background: linear-gradient(135deg, #9333ea, #7e22ce); }
.pro-execute:hover:not(:disabled) { box-shadow: 0 4px 12px rgba(147, 51, 234, 0.3); transform: translateY(-1px);}
.pro-execute:disabled { background: #cbd5e1; cursor: not-allowed;}

.result-card { display: flex; flex-direction: column;}
.empty-state { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: #94a3b8; }
.empty-icon { font-size: 3rem; margin-bottom: 0.5rem; opacity: 0.5;}
.result-content { padding: 1.5rem; display: flex; flex-direction: column;}
.result-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.75rem;}
.status-title { font-weight: 700; color: #166534; font-size: 1.1rem;}
.has-error .status-title { color: #dc2626; }
.error-msg { color: #dc2626; font-family: monospace; white-space: pre-wrap; background: #fef2f2; padding: 1rem; border-radius: 8px; border: 1px solid #fca5a5;}

.success-box { background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 15px;}
.log-info { display: flex; align-items: center; gap: 8px;}
.check-icon { color: #10b981; font-weight: bold;}
.log-filename { font-family: monospace; font-size: 0.9rem; color: #334155; font-weight: 600;}
.action-btn { background: white; border: 1px solid #cbd5e1; padding: 6px 12px; border-radius: 6px; font-size: 0.85rem; font-weight: bold; cursor: pointer; color: #10b981; display: flex; align-items: center; gap: 4px; transition: 0.2s;}
.action-btn:hover { border-color: #10b981; background: #ecfdf5; }
.secondary-btn { padding: 0.5rem 1rem; background: white; border: 1px solid #d1d5db; border-radius: 6px; cursor: pointer; font-weight: 600;}
.secondary-btn:hover { background: #f8fafc; }

.action-footer { display: flex; gap: 10px; }
.danger-execute {
  background: linear-gradient(135deg, #ef4444, #dc2626);
  display: flex;
  align-items: center;
  gap: 8px;
}
.danger-execute:hover {
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
  transform: translateY(-1px);
}
.spinner { animation: spin 2s linear infinite; }
@keyframes spin { 100% { transform: rotate(360deg); } }
</style>