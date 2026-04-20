<template>
  <div class="view-container">
    <header class="view-header">
      <h1 class="view-title">视频轨道检查 <span class="pro-badge-title">PRO</span></h1>
      <p class="view-subtitle">深度嗅探媒体流元数据，支持一键无损分离所有音轨或剥离纯净画面。</p>
    </header>

    <div class="main-layout hide-scrollbar">
      <div class="card form-card">
        <div class="card-body">
          <div class="section-title">1. 选择探测源</div>

          <div class="form-group source-group">
            <button
                v-if="!mediaInfo && !isLoading"
                class="upload-dropzone-btn pro-theme"
                @click="triggerFileSelect"
                @dragover.prevent="isDragging = true"
                @dragleave.prevent="isDragging = false"
                @drop.prevent="handleDrop"
                :class="{ 'is-active': isDragging }"
            >
              <span class="upload-icon">📂</span>
              <span class="upload-text">点击或拖拽视频文件至此进行深度嗅探</span>
            </button>

            <div v-if="isLoading" class="loader-box">
              <div class="spinner"></div>
              <p class="loading-text">正在解析底层流信息...</p>
            </div>

            <div v-if="mediaInfo && !isLoading" class="active-file-wrapper">
              <div class="active-file-badge">
                <span class="file-icon">🎬</span>
                <span class="file-name" :title="filePath">{{ fileName }}</span>
                <div class="file-meta-tag">{{ mediaInfo.format_name.toUpperCase() }}</div>
                <div class="badge-actions">
                  <button class="icon-btn" @click="resetInspector" title="移除并重新选择">✖</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <transition name="fade-up">
        <div class="dashboard-grid" v-if="mediaInfo && !isLoading">

          <div class="info-card video-card">
            <div class="card-header">
              <div class="header-title">
                <span class="type-icon">🎥</span> 视频流 ({{ mediaInfo.video_streams.length }})
              </div>
            </div>
            <div class="stream-list hide-scrollbar">
              <div v-if="mediaInfo.video_streams.length === 0" class="empty-hint">未检测到有效视频流</div>
              <div v-for="v in mediaInfo.video_streams" :key="v.index" class="stream-item">
                <div class="stream-main">
                  <span class="badge codec-badge">{{ v.codec.toUpperCase() }}</span>
                  <span class="resolution">{{ v.width }} × {{ v.height }}</span>
                </div>
                <div class="stream-sub">{{ v.fps.toFixed(2) }} FPS</div>
              </div>
            </div>
          </div>

          <div class="info-card audio-card">
            <div class="card-header">
              <div class="header-title">
                <span class="type-icon">🎵</span> 音频流 ({{ mediaInfo.audio_streams.length }})
              </div>
            </div>
            <div class="stream-list hide-scrollbar">
              <div v-if="mediaInfo.audio_streams.length === 0" class="empty-hint">无音轨</div>
              <div v-for="a in mediaInfo.audio_streams" :key="a.index" class="stream-item">
                <div class="stream-main">
                  <span class="badge codec-badge">{{ a.codec.toUpperCase() }}</span>
                  <span class="lang-text">{{ getLanguageName(a.language) }}</span>
                </div>
                <div class="stream-sub">轨道 #{{ a.index }}</div>
              </div>
            </div>
          </div>

          <div class="info-card sub-card">
            <div class="card-header">
              <div class="header-title">
                <span class="type-icon">📝</span> 字幕流 ({{ mediaInfo.subtitle_streams.length }})
              </div>
            </div>
            <div class="stream-list hide-scrollbar">
              <div v-if="mediaInfo.subtitle_streams.length === 0" class="empty-hint">未检测到软字幕</div>
              <div v-for="s in mediaInfo.subtitle_streams" :key="s.index" class="stream-item">
                <div class="stream-main">
                  <span class="badge codec-badge">{{ s.codec.toUpperCase() }}</span>
                  <span class="lang-text">{{ getLanguageName(s.language) }}</span>
                </div>
                <div class="stream-sub">轨道 #{{ s.index }}</div>
              </div>
            </div>
          </div>

        </div>
      </transition>

      <transition name="fade-up">
        <div class="card action-center-card" v-if="mediaInfo && !isLoading">
          <div class="card-body">
            <div class="section-title">2. 智能快速操作</div>

            <div class="form-group">
              <label>📁 保存目录</label>
              <div class="input-with-btn">
                <input v-model="outputDir" type="text" readonly class="file-input dir-input" placeholder="请选择保存目录" />
                <button class="secondary-btn" @click="changeOutputDir" :disabled="isExecuting">更改</button>
              </div>
            </div>

            <div class="smart-actions">
              <button
                  class="primary-btn action-btn pro-gradient"
                  :disabled="mediaInfo.audio_streams.length === 0 || isExecuting"
                  @click="extractAllAudio"
              >
                <span class="btn-icon">🎵</span>
                一键分离所有音轨 ({{ mediaInfo.audio_streams.length }})
              </button>

              <button
                  class="primary-btn action-btn pure-video-gradient"
                  :disabled="mediaInfo.video_streams.length === 0 || isExecuting"
                  @click="exportPureVideo"
              >
                <span class="btn-icon">🎥</span>
                提取纯净画面 (去音/软字幕)
              </button>
            </div>

            <div class="feedback-area" v-if="isExecuting || executionLog">
              <div class="result-header">
                <div class="status-title">
                  <span v-if="isExecuting">⏳ 正在底层无损执行中...</span>
                  <span v-else-if="isError" class="error-title">❌ 任务失败</span>
                  <span v-else class="success-title">🎉 执行完毕</span>
                </div>
                <button v-if="!isExecuting && !isError && executionLog" class="secondary-btn small" @click="openTargetFolder">
                  📂 打开输出目录
                </button>
              </div>
              <div class="execution-log hide-scrollbar" :class="{ 'is-error': isError }">
                {{ executionLog }}
              </div>
            </div>

          </div>
        </div>
      </transition>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

// 状态管理
const isDragging = ref(false);
const isLoading = ref(false);
const isExecuting = ref(false);
const isError = ref(false);

const mediaInfo = ref<any>(null);
const filePath = ref('');
const outputDir = ref('');
const executionLog = ref('');

const fileName = computed(() => filePath.value.split(/[/\\]/).pop());

// 重置页面状态
function resetInspector() {
  mediaInfo.value = null;
  filePath.value = '';
  outputDir.value = '';
  executionLog.value = '';
  isError.value = false;
}

// 获取语言的全称显示
function getLanguageName(langCode: string): string {
  if (!langCode || langCode === 'und') return '未指定 (und)';
  if (langCode === 'chi' || langCode === 'zho') return '中文';
  if (langCode === 'eng') return '英文';
  if (langCode === 'jpn') return '日文';
  return langCode;
}

// 拖拽处理
async function handleDrop(e: DragEvent) {
  isDragging.value = false;
  const file = e.dataTransfer?.files[0];
  if (file) {
    const realPath = (file as any).path;
    if (realPath) {
      startProbing(realPath);
    } else {
      alert("无法获取本地绝对路径，请点击按钮选择文件。");
    }
  }
}

// 文件选择
async function triggerFileSelect() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Video', extensions: ['mp4', 'mkv', 'mov', 'avi', 'flv'] }]
  });
  if (selected && typeof selected === 'string') {
    startProbing(selected);
  }
}

// 更改输出目录
async function changeOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected && typeof selected === 'string') {
    outputDir.value = selected;
  }
}

// 打开结果目录
async function openTargetFolder() {
  try { await invoke('open_folder', { path: outputDir.value }); } catch (e) { console.error(e); }
}

// 执行底层探测
async function startProbing(path: string) {
  isLoading.value = true;
  executionLog.value = '';
  isError.value = false;
  filePath.value = path;

  // 默认将输出目录设为输入文件所在目录
  const pathParts = path.replace(/\\/g, '/').split('/');
  pathParts.pop();
  outputDir.value = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

  try {
    mediaInfo.value = await invoke('probe_media_info_cmd', { videoPath: path });
  } catch (err) {
    alert(`探测失败: ${err}`);
    resetInspector();
  } finally {
    isLoading.value = false;
  }
}

// 动作 1：提取所有音轨
async function extractAllAudio() {
  if (!mediaInfo.value?.audio_streams.length) return;
  isExecuting.value = true;
  isError.value = false;
  executionLog.value = "🚀 正在并发提取所有音轨 (物理拷贝)...";

  try {
    const res = await invoke('run_extract_all_audio_cmd', {
      params: {
        input_path: filePath.value,
        output_dir: outputDir.value,
        audio_streams: mediaInfo.value.audio_streams
      }
    });
    executionLog.value = res as string;
  } catch (err) {
    isError.value = true;
    executionLog.value = String(err);
  } finally {
    isExecuting.value = false;
  }
}

// 动作 2：提取纯视频
async function exportPureVideo() {
  if (!mediaInfo.value?.video_streams.length) return;
  isExecuting.value = true;
  isError.value = false;
  executionLog.value = "🚀 正在无损剥离音轨与字幕...";

  try {
    const res = await invoke('run_export_pure_video_cmd', {
      params: {
        input_path: filePath.value,
        output_dir: outputDir.value,
      }
    });
    executionLog.value = res as string;
  } catch (err) {
    isError.value = true;
    executionLog.value = String(err);
  } finally {
    isExecuting.value = false;
  }
}
</script>

<style scoped>
@import '../assets/global.css';

.hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
.hide-scrollbar::-webkit-scrollbar { display: none; }

.view-container { display: flex; flex-direction: column; height: 100vh; overflow: hidden; padding: 1.2rem; box-sizing: border-box; background: #f8fafc;}
.view-header { margin-bottom: 0.8rem; flex-shrink: 0; }
.view-title { margin: 0 0 0.4rem 0; display: flex; align-items: center; gap: 10px;}
.view-subtitle { margin: 0; color: #6b7280; font-size: 0.9rem; }
.pro-badge-title { background: linear-gradient(135deg, #3b82f6, #1d4ed8); color: white; font-size: 0.8rem; padding: 4px 8px; border-radius: 6px; font-weight: 900;}

.main-layout { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 1.2rem; padding-bottom: 2rem; }

/* 基础卡片与通用样式 (沿用你已有的风格) */
.card { background: white; border-radius: 12px; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05); border: 1px solid #e5e7eb; overflow: hidden; }
.card-body { padding: 1.2rem; }
.section-title { font-size: 1.05rem; font-weight: 600; margin-bottom: 0.8rem; color: #111827;}
.form-group { margin-bottom: 1rem; display: flex; flex-direction: column; }
.form-group label { font-size: 0.85rem; color: #4b5563; margin-bottom: 0.4rem; font-weight: 600;}

/* 交互按钮 */
.upload-dropzone-btn { width: 100%; min-height: 116px; background: #f8fafc; border: 2px dashed #cbd5e1; border-radius: 8px; cursor: pointer; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; transition: all 0.2s ease;}
.upload-dropzone-btn.pro-theme:hover, .upload-dropzone-btn.is-active { background: #eff6ff; border-color: #3b82f6; }
.upload-dropzone-btn.pro-theme .upload-text { color: #1d4ed8; font-weight: 700;}
.upload-icon { font-size: 2rem; }

.input-with-btn { display: flex; gap: 8px; }
.file-input { flex: 1; padding: 0.6rem 1rem; border: 1px solid #d1d5db; border-radius: 8px; background: #f9fafb; color: #4b5563; outline: none; font-family: monospace;}
.secondary-btn { padding: 0.5rem 1rem; background: white; border: 1px solid #d1d5db; border-radius: 6px; cursor: pointer; font-weight: 600; transition: 0.2s; white-space: nowrap;}
.secondary-btn:hover:not(:disabled) { background: #f8fafc; }
.secondary-btn:disabled { opacity: 0.5; cursor: not-allowed; }

/* 已加载状态条 */
.active-file-wrapper { display: flex; flex-direction: column; }
.active-file-badge { display: flex; align-items: center; background: #f8fafc; padding: 0.75rem 1rem; border-radius: 8px; gap: 0.75rem; border: 1px solid #cbd5e1;}
.file-icon { font-size: 1.4rem; }
.file-name { font-weight: 600; color: #334155; font-size: 0.9rem; flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: monospace;}
.file-meta-tag { background: #e2e8f0; color: #475569; font-size: 0.75rem; padding: 3px 8px; border-radius: 4px; font-weight: bold; }
.badge-actions { border-left: 1px solid #cbd5e1; padding-left: 0.5rem;}
.icon-btn { background: transparent; border: none; color: #64748b; cursor: pointer; font-size: 1rem; transition: 0.2s;}
.icon-btn:hover { color: #ef4444; }

/* 加载动画 */
.loader-box { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 116px; background: #f8fafc; border-radius: 8px; border: 1px solid #e2e8f0;}
.spinner { width: 28px; height: 28px; border: 3px solid #cbd5e1; border-top-color: #3b82f6; border-radius: 50%; animation: spin 1s linear infinite; margin-bottom: 8px;}
.loading-text { color: #64748b; font-size: 0.85rem; font-weight: 600;}
@keyframes spin { to { transform: rotate(360deg); } }

/* 仪表盘卡片组 */
.dashboard-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem; }
.info-card { background: white; border-radius: 10px; border: 1px solid #e5e7eb; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.02);}
.card-header { padding: 0.8rem 1rem; background: #f8fafc; border-bottom: 1px solid #e5e7eb; }
.header-title { font-weight: 700; color: #1e293b; font-size: 0.95rem; display: flex; align-items: center; gap: 8px;}
.video-card .card-header { border-top: 3px solid #3b82f6; }
.audio-card .card-header { border-top: 3px solid #10b981; }
.sub-card .card-header { border-top: 3px solid #f59e0b; }

.stream-list { padding: 0.5rem; max-height: 200px; overflow-y: auto;}
.stream-item { display: flex; align-items: center; justify-content: space-between; padding: 0.6rem 0.8rem; border-radius: 6px; margin-bottom: 4px; background: #f8fafc; border: 1px solid transparent; transition: 0.2s;}
.stream-item:hover { border-color: #cbd5e1; background: white;}
.stream-main { display: flex; align-items: center; gap: 10px; }
.stream-sub { font-size: 0.8rem; color: #64748b; font-family: monospace; font-weight: 600;}
.badge { padding: 3px 6px; border-radius: 4px; font-size: 0.75rem; font-weight: bold; font-family: monospace;}
.codec-badge { background: #e0f2fe; color: #1d4ed8; border: 1px solid #bfdbfe;}
.resolution { font-size: 0.85rem; font-weight: 700; color: #334155; }
.lang-text { font-size: 0.85rem; color: #475569; font-weight: 600;}
.empty-hint { text-align: center; color: #94a3b8; font-size: 0.85rem; padding: 1.5rem 0; font-style: italic;}

/* 底部操作区 */
.action-center-card { background: linear-gradient(to bottom, white, #f8fafc); }
.smart-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 1.5rem; }
.action-btn { padding: 1rem; font-size: 1rem; font-weight: bold; border-radius: 8px; border: none; cursor: pointer; color: white; display: flex; align-items: center; justify-content: center; gap: 8px; transition: 0.2s;}
.action-btn:disabled { opacity: 0.5; cursor: not-allowed; filter: grayscale(1); }
.pro-gradient { background: linear-gradient(135deg, #10b981, #059669); }
.pro-gradient:hover:not(:disabled) { box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3); transform: translateY(-1px);}
.pure-video-gradient { background: linear-gradient(135deg, #8b5cf6, #4338ca); }
.pure-video-gradient:hover:not(:disabled) { box-shadow: 0 4px 12px rgba(139, 92, 246, 0.3); transform: translateY(-1px);}

/* 结果反馈区 */
.feedback-area { margin-top: 1.5rem; background: white; border: 1px solid #e2e8f0; border-radius: 8px; overflow: hidden;}
.result-header { display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1rem; background: #f8fafc; border-bottom: 1px solid #e2e8f0;}
.status-title { font-weight: 700; color: #475569; font-size: 0.95rem;}
.success-title { color: #166534; }
.error-title { color: #dc2626; }
.execution-log { padding: 1rem; font-family: 'Consolas', monospace; font-size: 0.85rem; color: #334155; white-space: pre-wrap; max-height: 150px; overflow-y: auto; line-height: 1.5;}
.execution-log.is-error { background: #fef2f2; color: #b91c1c; }

/* 动画 */
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.3s ease-out; }
.fade-up-enter-from { opacity: 0; transform: translateY(15px); }
</style>