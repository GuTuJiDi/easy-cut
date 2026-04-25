<template>
  <div class="view-container">
    <header class="view-header">
      <div class="header-content">
        <h1 class="view-title">视频轨道检查 <span class="pro-badge-title">PRO</span></h1>
        <p class="view-subtitle">深度嗅探媒体流元数据，支持轨道级精确制导与独立剥离。</p>
      </div>
    </header>

    <div class="main-layout">
      <div v-if="!authStore.isPro" class="pro-lock-overlay">
        <div class="lock-content">
          <span class="lock-icon">🔒</span>
          <h2>专属功能已锁定</h2>
          <p>“高级轨道探测与分离”为 PRO 旗舰版专属权益。<br>请前往授权中心输入激活码解锁该功能。</p>
          <button class="primary-btn" @click="$router.push('/auth')">前往授权中心</button>
        </div>
      </div>
      <div v-else class="split-layout">

        <aside class="left-panel custom-scrollbar">

          <div class="panel-card source-card">
            <h3 class="panel-title">1. 选择探测源</h3>

            <div v-if="authStore.isInitialized &&!mediaInfo && !isLoading"
                 class="upload-dropzone"
                 :class="{ 'is-dragover': isDragging }"
                 @click="triggerFileSelect"
                 @dragover.prevent="isDragging = true"
                 @dragleave.prevent="isDragging = false"
                 @drop.prevent="handleDrop">
              <span class="dropzone-icon">✨</span>
              <p class="dropzone-text">点击或拖拽选择视频</p>
            </div>

            <div v-if="!authStore.isInitialized" class="loading-state">
              <div class="spinner-small blue"></div>
              <span>正在嗅探底层流...</span>
            </div>

            <div v-if="mediaInfo && !isLoading" class="active-file-compact">
              <div class="file-header-row">
                <span class="file-icon-box">🎬</span>
                <span class="file-name-min" :title="filePath">{{ fileName }}</span>
                <button class="icon-btn-min danger" @click="resetInspector" title="移除视频">✖</button>
              </div>
              <div class="file-tags-row">
                <span class="tag format-tag">{{ mediaInfo.format_name.toUpperCase() }}</span>
                <span class="tag">⏱️ {{ formatDuration(mediaInfo.duration_sec) }}</span>
                <span class="tag">💾 {{ formatBytes(mediaInfo.size_bytes) }}</span>
              </div>
            </div>

            <button v-if="mediaInfo && !isLoading" class="replace-btn" @click="triggerFileSelect">
              🔄 更换探测视频
            </button>
          </div>

          <transition name="fade">
            <div class="panel-card action-card" v-if="mediaInfo && !isLoading">
              <h3 class="panel-title">2. 快速分离策略</h3>

              <div class="form-group">
                <label class="mini-label">📁 保存目录</label>
                <div class="input-with-btn">
                  <input v-model="outputDir" type="text" readonly class="dir-input-mini" />
                  <button class="btn-outline-mini" @click="changeOutputDir" :disabled="executingTask !== null">更改</button>
                </div>
              </div>

              <div class="vertical-actions">
                <button class="btn-block extract-audio"
                        :disabled="mediaInfo.audio_streams.length === 0 || executingTask !== null"
                        @click="extractAllAudio">
                  <span v-if="executingTask === 'all_audio'" class="spinner-small white"></span>
                  <span v-else class="btn-icon">🎵</span>
                  <span class="btn-text">{{ executingTask === 'all_audio' ? '分离中...' : `一键分离所有音轨 (${mediaInfo.audio_streams.length})` }}</span>
                </button>

                <button class="btn-block extract-video"
                        :disabled="mediaInfo.video_streams.length === 0 || executingTask !== null"
                        @click="exportPureVideo">
                  <span v-if="executingTask === 'pure_video'" class="spinner-small white"></span>
                  <span v-else class="btn-icon">🎥</span>
                  <span class="btn-text">{{ executingTask === 'pure_video' ? '剥离中...' : '提取纯净画面 (去音/去字)' }}</span>
                </button>
              </div>
            </div>
          </transition>
        </aside>

        <main class="right-panel">

          <div v-if="!mediaInfo && !isLoading" class="empty-placeholder">
            <div class="placeholder-icon">📡</div>
            <p>请在左侧导入视频文件，探测结果将在此展示</p>
          </div>

          <transition name="fade">
            <div class="right-content-wrapper" v-if="mediaInfo && !isLoading">

              <div class="tracks-container">
                <h3 class="panel-title">轨道拓扑视图</h3>
                <div class="track-grid">

                  <div class="track-column">
                    <div class="column-header blue"><span class="icon">🎥</span> 视频流</div>
                    <div class="column-body custom-scrollbar">
                      <div v-if="mediaInfo.video_streams.length === 0" class="empty-track">无视频流</div>
                      <div v-for="v in mediaInfo.video_streams" :key="v.index" class="track-node">
                        <div class="node-info">
                          <div class="node-top">
                            <span class="codec blue">{{ v.codec.toUpperCase() }}</span>
                            <span class="index">#{{ v.index }}</span>
                          </div>
                          <div class="node-main">{{ v.width }} × {{ v.height }}</div>
                          <div class="node-sub">{{ v.fps.toFixed(2) }} FPS</div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="track-column">
                    <div class="column-header green"><span class="icon">🎵</span> 音频流</div>
                    <div class="column-body custom-scrollbar">
                      <div v-if="mediaInfo.audio_streams.length === 0" class="empty-track">无独立音轨</div>
                      <div v-for="a in mediaInfo.audio_streams" :key="a.index" class="track-node hoverable">
                        <div class="node-info">
                          <div class="node-top">
                            <span class="codec green">{{ a.codec.toUpperCase() }}</span>
                            <span class="index">#{{ a.index }}</span>
                          </div>
                          <div class="node-main" :title="a.title || getLanguageName(a.language)">
                            {{ a.title || getLanguageName(a.language) }}
                          </div>
                        </div>
                        <button class="node-action-btn" @click="extractSingleTrack(a, 'audio')" :disabled="executingTask !== null" title="提取此条音轨">
                          <span v-if="executingTask === 'audio_' + a.index" class="spinner-small"></span>
                          <span v-else>提取</span>
                        </button>
                      </div>
                    </div>
                  </div>

                  <div class="track-column">
                    <div class="column-header orange"><span class="icon">📝</span> 软字幕流</div>
                    <div class="column-body custom-scrollbar">
                      <div v-if="mediaInfo.subtitle_streams.length === 0" class="empty-track">无内挂软字幕</div>
                      <div v-for="s in mediaInfo.subtitle_streams" :key="s.index" class="track-node hoverable">
                        <div class="node-info">
                          <div class="node-top">
                            <span class="codec orange">{{ s.codec.toUpperCase() }}</span>
                            <span class="index">#{{ s.index }}</span>
                          </div>
                          <div class="node-main" :title="s.title || getLanguageName(s.language)">
                            {{ s.title || getLanguageName(s.language) }}
                          </div>
                        </div>
                        <button class="node-action-btn" @click="extractSingleTrack(s, 'subtitle')" :disabled="executingTask !== null" title="提取此字幕文件">
                          <span v-if="executingTask === 'subtitle_' + s.index" class="spinner-small"></span>
                          <span v-else>提取</span>
                        </button>
                      </div>
                    </div>
                  </div>

                </div>
              </div>

              <div class="terminal-container" v-if="executingTask !== null || executionLog">
                <div class="terminal-header">
                  <div class="term-left">
                    <div class="status-dot" :class="{'is-pulsing': executingTask !== null, 'is-error': isError, 'is-success': !isError && executingTask === null}"></div>
                    <span class="term-title">
                      {{ executingTask !== null ? '任务执行中' : (isError ? '任务中断' : '任务完成') }}
                    </span>
                    <span class="term-timer" v-if="taskCostTime !== '0.0' || executingTask !== null">
                      ⏱️ 耗时: {{ taskCostTime }}s
                    </span>
                  </div>
                  <button v-if="executingTask === null && !isError && executionLog" class="term-open-btn" @click="openTargetFolder">
                    📂 打开输出目录
                  </button>
                </div>
                <div class="terminal-body custom-scrollbar" :class="{ 'error-text': isError }">
                  <pre>{{ executionLog }}</pre>
                </div>
              </div>

            </div>
          </transition>
        </main>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useAuthStore } from '../stores/auth'; // 引入 Store
// 🌟 引入 Tauri 官方路径处理 API，替代脆弱的字符串操作
import { dirname } from '@tauri-apps/api/path';
const authStore = useAuthStore();
// 状态管理
const isDragging = ref(false);
const isLoading = ref(false);
const executingTask = ref<string | null>(null);
const isError = ref(false);

const mediaInfo = ref<any>(null);
const filePath = ref('');
const outputDir = ref('');
const executionLog = ref('');
// 🌟 安全优化：合法的视频后缀白名单
const VALID_EXTENSIONS = ['.mp4', '.mkv', '.mov', '.avi', '.flv', '.ts', '.webm'];
// 任务耗时追踪逻辑
const taskCostTime = ref('0.0');
let timerInterval: number | null = null;

watch(executingTask, (newVal) => {
  if (newVal !== null) {
    const start = performance.now();
    taskCostTime.value = '0.0';
    if (timerInterval) clearInterval(timerInterval);
    timerInterval = window.setInterval(() => {
      taskCostTime.value = ((performance.now() - start) / 1000).toFixed(1);
    }, 100);
  } else {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }
});

const fileName = computed(() => filePath.value.split(/[/\\]/).pop());

function formatDuration(seconds: number): string {
  if (!seconds || isNaN(seconds)) return '00:00:00';
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return [h, m, s].map(v => v.toString().padStart(2, '0')).join(':');
}

function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function resetInspector() {
  mediaInfo.value = null; filePath.value = ''; outputDir.value = ''; executionLog.value = ''; isError.value = false;
  taskCostTime.value = '0.0';
}

function getLanguageName(langCode: string): string {
  if (!langCode || langCode === 'und') return '未知 (und)';
  if (langCode === 'chi' || langCode === 'zho') return '中文';
  if (langCode === 'eng') return '英文';
  if (langCode === 'jpn') return '日文';
  return langCode;
}

async function handleDrop(e: DragEvent) {
  isDragging.value = false;
  const file = e.dataTransfer?.files[0];
  if (file) {
    const realPath = (file as any).path as string;
    if (!realPath) {
      alert("无法获取本地绝对路径，请点击按钮选择文件。");
      return;
    }
    // 🌟 安全防御：校验拖拽文件的后缀，防止恶意文件注入底层 FFmpeg
    const isVideo = VALID_EXTENSIONS.some(ext => realPath.toLowerCase().endsWith(ext));
    if (!isVideo) {
      alert("不支持的文件格式，请拖入有效的视频文件！");
      return;
    }
    startProbing(realPath);
  }
}

async function triggerFileSelect() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Video', extensions: ['mp4', 'mkv', 'mov', 'avi', 'flv', 'ts', 'webm'] }]
  });
  if (selected && typeof selected === 'string') startProbing(selected);
}

async function changeOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected && typeof selected === 'string') outputDir.value = selected;
}

async function openTargetFolder() {
  try { await invoke('open_folder', { path: outputDir.value }); } catch (e) { console.error(e); }
}

async function startProbing(path: string) {
  isLoading.value = true; executionLog.value = ''; isError.value = false; filePath.value = path; taskCostTime.value = '0.0';

  try {
    // 🌟 健壮性优化：使用官方跨平台 API 获取安全的父级目录
    outputDir.value = await dirname(path);

    mediaInfo.value = await invoke('probe_media_info_cmd', {
      videoPath: path,
      sessionToken: authStore.sessionToken // 这个你之前加了，保持即可
    });
  } catch (err) {
    alert(`探测失败: ${err}`); resetInspector();
  } finally {
    isLoading.value = false;
  }
}
// 🌟 2. 修改单轨提取
// 🌟 铁穹接入：修复所有的 invoke 调用，补齐 sessionToken
async function extractSingleTrack(track: any, type: 'audio' | 'subtitle') {
  executingTask.value = `${type}_${track.index}`;
  isError.value = false;
  const targetName = track.title || getLanguageName(track.language);
  executionLog.value = `> 正在挂载单轨提取任务...\n> 目标轨道: #${track.index} [${targetName}]\n> 底层流复制进行中...`;

  try {
    const videoStem = fileName.value?.substring(0, fileName.value.lastIndexOf('.')) || 'video';
    let ext = 'mkv';
    if (type === 'audio') {
      const c = track.codec.toLowerCase();
      if (c.includes('aac')) ext = 'm4a';
      else if (c.includes('eac3')) ext = 'eac3';
      else if (c.includes('ac3')) ext = 'ac3';
      else if (c.includes('mp3')) ext = 'mp3';
      else if (c.includes('wav') || c.includes('pcm')) ext = 'wav';
      else if (c.includes('flac')) ext = 'flac';
      else ext = 'mka';
    } else {
      ext = track.codec.toLowerCase().includes('ass') ? 'ass' : 'srt';
    }
    const outName = `${videoStem}_轨道${track.index}_${targetName}.${ext}`;

    const res = await invoke<string>('extract_single_segment', {
      params: {
        video_path: filePath.value, start_time: 0, end_time: mediaInfo.value.duration_sec,
        expected_batch_path: null, fallback_output_dir: outputDir.value, fallback_file_name: outName,
        export_type: type === 'audio' ? 'audio_only' : 'subs_only', track_index: track.index
      },
      sessionToken: authStore.sessionToken // 🛡️ 补齐令牌
    });
    executionLog.value += `\n> [OK] 提取成功: ${res}`;
  } catch (err) {
    isError.value = true; executionLog.value += `\n> [FATAL] 异常中断:\n${String(err)}`;
  } finally { executingTask.value = null; }
}
// 🌟 3. 修改一键分离所有音轨
async function extractAllAudio() {
  if (!mediaInfo.value?.audio_streams.length) return;
  executingTask.value = 'all_audio'; isError.value = false;
  executionLog.value = `> 初始化批量音频分离引擎...\n> 检索到 ${mediaInfo.value.audio_streams.length} 条音轨\n> 并发流拷贝进行中...`;

  try {
    const res = await invoke('run_extract_all_audio_cmd', {
      params: { input_path: filePath.value, output_dir: outputDir.value, audio_streams: mediaInfo.value.audio_streams },
      sessionToken: authStore.sessionToken // 🛡️ 补齐令牌
    });
    executionLog.value += `\n> [OK] 批量分离完成:\n${res}`;
  } catch (err) {
    isError.value = true; executionLog.value += `\n> [FATAL] 引擎异常:\n${String(err)}`;
  } finally { executingTask.value = null; }
}
// 🌟 4. 修改提取纯净画面
async function exportPureVideo() {
  if (!mediaInfo.value?.video_streams.length) return;
  executingTask.value = 'pure_video'; isError.value = false;
  executionLog.value = `> 挂载纯净画面封装器...\n> 指令集: 阻断音频流(-an) | 阻断软字幕流(-sn)\n> 提示: 烙印在画面像素上的硬字幕(Hard Subs)无法被流过滤。\n> 处理中...`;

  try {
    const res = await invoke('run_export_pure_video_cmd', {
      params: { input_path: filePath.value, output_dir: outputDir.value },
      sessionToken: authStore.sessionToken // 🛡️ 补齐令牌
    });
    executionLog.value += `\n> [OK] 封装完成:\n${res}`;
  } catch (err) {
    isError.value = true; executionLog.value += `\n> [FATAL] 处理失败:\n${String(err)}`;
  } finally { executingTask.value = null; }
}
</script>

<style scoped>
@import '../assets/global.css';

/* ================= 基础与排版 (严格的视图控制) ================= */
.view-container { display: flex; flex-direction: column; height: 100vh; overflow: hidden; background: #f8fafc; }
.view-header { background: white; border-bottom: 1px solid #e2e8f0; padding: 1rem 1.5rem; flex-shrink: 0; z-index: 10;}
.header-content { width: 100%; display: flex; flex-direction: column; gap: 4px;}
.view-title { margin: 0; display: flex; align-items: center; gap: 10px; font-size: 1.3rem; color: #0f172a;}
.view-subtitle { margin: 0; color: #64748b; font-size: 0.85rem; }
.pro-badge-title { background: linear-gradient(135deg, #3b82f6, #1d4ed8); color: white; font-size: 0.7rem; padding: 3px 8px; border-radius: 6px; font-weight: 900; letter-spacing: 0.5px;}

/* 主工作区：让出滚动权给内部组件 */
.main-layout { flex: 1; min-height: 0; padding: 1.5rem; display: flex; flex-direction: column; overflow: hidden; }

/* 🌟 精美的纤细半透明滚动条 */
.custom-scrollbar::-webkit-scrollbar { width: 6px; height: 6px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(148, 163, 184, 0.3); border-radius: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb:hover { background: rgba(148, 163, 184, 0.6); }

/* ================= 🌟 全新流式两栏布局 ================= */
.split-layout {
  display: flex; gap: 1.5rem; width: 100%; height: 100%; min-height: 0;
}

/* 面板通用样式 */
.panel-card { background: white; border-radius: 12px; border: 1px solid #e2e8f0; padding: 1.25rem; box-shadow: 0 2px 4px rgba(0,0,0,0.02); display: flex; flex-direction: column; gap: 1rem;}
.panel-title { margin: 0; font-size: 1rem; font-weight: 700; color: #1e293b; padding-bottom: 0.75rem; border-bottom: 1px dashed #e2e8f0;}

/* ================= 左侧：控制台 (紧凑排版) ================= */
.left-panel { width: 340px; flex-shrink: 0; display: flex; flex-direction: column; gap: 1.2rem; overflow-y: auto; padding-right: 4px;}

.upload-dropzone {
  border: 1.5px dashed #cbd5e1; border-radius: 8px; background: #f8fafc; padding: 2rem 1rem;
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  cursor: pointer; transition: all 0.2s ease; text-align: center;
}
.upload-dropzone:hover, .upload-dropzone.is-dragover { border-color: #f59e0b; background: #fffbeb; }
.dropzone-icon { font-size: 2rem; margin-bottom: 0.5rem; }
.dropzone-text { margin: 0; font-size: 0.85rem; color: #64748b; font-weight: 500;}

.loading-state { display: flex; flex-direction: column; align-items: center; gap: 10px; padding: 2rem 0; color: #64748b; font-size: 0.85rem;}

.active-file-compact { display: flex; flex-direction: column; gap: 10px; background: #f1f5f9; padding: 12px; border-radius: 8px; border: 1px solid #cbd5e1;}
.file-header-row { display: flex; align-items: center; gap: 8px; }
.file-icon-box { font-size: 1.2rem; }
.file-name-min { font-size: 0.9rem; font-weight: 700; color: #1e293b; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: monospace; flex: 1;}
.icon-btn-min { background: transparent; border: none; color: #94a3b8; font-size: 1rem; cursor: pointer; padding: 2px; transition: 0.2s; flex-shrink: 0;}
.icon-btn-min.danger:hover { color: #ef4444; }

.file-tags-row { display: flex; gap: 6px; flex-wrap: wrap;}
.tag { font-size: 0.7rem; background: #e2e8f0; color: #475569; padding: 3px 8px; border-radius: 4px; font-weight: bold; letter-spacing: 0.5px;}
.format-tag { background: #e0f2fe; color: #0369a1; }

.replace-btn { background: white; border: 1px dashed #cbd5e1; color: #64748b; padding: 8px; border-radius: 8px; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: 0.2s;}
.replace-btn:hover { background: #f8fafc; border-color: #94a3b8; color: #334155;}

.form-group { display: flex; flex-direction: column; gap: 6px; }
.mini-label { font-size: 0.8rem; color: #64748b; font-weight: 600; }
.input-with-btn { display: flex; gap: 6px; }
.dir-input-mini { flex: 1; min-width: 0; padding: 8px 10px; border: 1px solid #cbd5e1; border-radius: 6px; background: #f8fafc; font-size: 0.8rem; color: #475569; font-family: monospace; outline: none;}
.btn-outline-mini { padding: 0 12px; background: white; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 0.8rem; font-weight: 600; color: #475569; cursor: pointer; transition: 0.2s;}
.btn-outline-mini:hover:not(:disabled) { background: #f1f5f9; border-color: #94a3b8; }

.vertical-actions { display: flex; flex-direction: column; gap: 10px; margin-top: 4px;}
.btn-block { width: 100%; padding: 12px; border-radius: 8px; border: none; font-size: 0.9rem; font-weight: 700; color: white; display: flex; align-items: center; justify-content: center; gap: 8px; cursor: pointer; transition: all 0.2s ease;}
.btn-block:hover:not(:disabled) { transform: translateY(-1px); filter: brightness(1.05); }
.btn-block:active:not(:disabled) { transform: translateY(0); filter: brightness(0.95); }
.btn-block:disabled { opacity: 0.6; filter: grayscale(0.8); cursor: not-allowed; }

.extract-audio { background: #10b981; box-shadow: 0 2px 4px rgba(16,185,129,0.2);}
.extract-video { background: #8b5cf6; box-shadow: 0 2px 4px rgba(139,92,246,0.2);}

/* ================= 右侧：大屏展示 (限制高度，内部滚动) ================= */
.right-panel { flex: 1; min-width: 0; display: flex; flex-direction: column; height: 100%; min-height: 0;}

.empty-placeholder { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; background: white; border-radius: 12px; border: 1px dashed #cbd5e1; color: #94a3b8;}
.placeholder-icon { font-size: 3.5rem; margin-bottom: 1rem; opacity: 0.5;}

.right-content-wrapper { display: flex; flex-direction: column; gap: 1.5rem; height: 100%; min-height: 0;}

/* 轨道视图区域 */
.tracks-container { flex: 1; min-height: 0; background: white; border-radius: 12px; border: 1px solid #e2e8f0; padding: 1.25rem; box-shadow: 0 2px 4px rgba(0,0,0,0.02); display: flex; flex-direction: column;}
.track-grid { flex: 1; min-height: 0; display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; margin-top: 1rem;}
.track-column { background: #f8fafc; border-radius: 8px; border: 1px solid #e2e8f0; display: flex; flex-direction: column; min-height: 0;}

.column-header { padding: 10px 12px; font-size: 0.9rem; font-weight: 700; display: flex; align-items: center; gap: 6px; background: white; border-bottom: 1px solid #e2e8f0;}
.column-header.blue { color: #0284c7; border-top: 3px solid #3b82f6;}
.column-header.green { color: #059669; border-top: 3px solid #10b981;}
.column-header.orange { color: #d97706; border-top: 3px solid #f59e0b;}

.column-body { flex: 1; overflow-y: auto; padding: 8px;}
.empty-track { text-align: center; color: #94a3b8; font-size: 0.8rem; font-style: italic; padding-top: 2rem;}

/* 🌟 微交互轨道卡片：左右排列节省纵向空间 */
.track-node { background: white; border: 1px solid #e2e8f0; border-radius: 6px; padding: 10px; margin-bottom: 8px; position: relative; transition: all 0.2s; display: flex; align-items: center; justify-content: space-between; gap: 8px;}
.track-node.hoverable:hover { border-color: #cbd5e1; box-shadow: 0 2px 4px rgba(0,0,0,0.04);}

.node-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
.node-top { display: flex; align-items: center; justify-content: space-between; }
.codec { font-size: 0.65rem; font-weight: 800; padding: 2px 6px; border-radius: 4px; font-family: monospace;}
.codec.blue { background: #e0f2fe; color: #0284c7; }
.codec.green { background: #d1fae5; color: #059669; }
.codec.orange { background: #fef3c7; color: #d97706; }
.index { font-size: 0.7rem; color: #94a3b8; font-family: monospace; font-weight: bold;}

.node-main { font-size: 0.85rem; font-weight: 600; color: #1e293b; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;}
.node-sub { font-size: 0.75rem; color: #64748b; font-family: monospace; margin-top: 2px;}

/* 纤薄的提取按钮 */
.node-action-btn {
  flex-shrink: 0; width: auto; padding: 6px 12px; height: 30px;
  background: #f8fafc; border: 1px solid #cbd5e1; border-radius: 4px;
  font-size: 0.75rem; font-weight: 600; color: #475569; cursor: pointer; transition: 0.2s;
  display: flex; align-items: center; justify-content: center;
}
.track-node.hoverable:hover .node-action-btn:not(:disabled) { background: white; border-color: #94a3b8; color: #0f172a;}
.node-action-btn:disabled { opacity: 0.7; cursor: not-allowed; }

/* ================= 控制台 Terminal ================= */
.terminal-container { flex-shrink: 0; height: 260px; background: #0f172a; border-radius: 12px; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.1);}
.terminal-header { display: flex; justify-content: space-between; align-items: center; padding: 8px 16px; background: #1e293b; border-bottom: 1px solid #334155;}
.term-left { display: flex; align-items: center; gap: 10px; }
.status-dot { width: 8px; height: 8px; border-radius: 50%; background: #64748b;}
.status-dot.is-pulsing { background: #eab308; box-shadow: 0 0 8px #fcd34d; animation: pulse 1.5s infinite;}
.status-dot.is-error { background: #ef4444; box-shadow: 0 0 8px #fca5a5;}
.status-dot.is-success { background: #10b981; box-shadow: 0 0 8px #6ee7b7;}
.term-title { color: #e2e8f0; font-size: 0.85rem; font-weight: 600;}
.term-timer { color: #38bdf8; font-size: 0.8rem; font-family: monospace; font-weight: bold; background: rgba(56,189,248,0.1); padding: 2px 8px; border-radius: 4px; margin-left: 8px;}

.term-open-btn { background: transparent; border: 1px solid #475569; color: #cbd5e1; padding: 4px 10px; border-radius: 6px; font-size: 0.75rem; cursor: pointer; transition: 0.2s;}
.term-open-btn:hover { background: #334155; color: white;}

.terminal-body { padding: 1rem 1.5rem; overflow-y: auto; flex: 1;}
.terminal-body pre { margin: 0; font-family: 'Consolas', monospace; font-size: 0.85rem; color: #a7f3d0; line-height: 1.6; white-space: pre-wrap; word-break: break-all;}
.terminal-body.error-text pre { color: #fca5a5; }

/* ================= 通用动画 ================= */
.spinner-small { width: 14px; height: 14px; border: 2px solid rgba(0,0,0,0.2); border-top-color: currentColor; border-radius: 50%; animation: spin 1s linear infinite; display: inline-block;}
.spinner-small.blue { border-color: #bfdbfe; border-top-color: #3b82f6;}
.spinner-small.white { border-color: rgba(255,255,255,0.3); border-top-color: white;}

@keyframes spin { to { transform: rotate(360deg); } }
@keyframes pulse { 0% { opacity: 0.5; } 50% { opacity: 1; } 100% { opacity: 0.5; } }

.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

/* ================= PRO 锁定遮罩 ================= */
.pro-lock-overlay {
  flex: 1; display: flex; align-items: center; justify-content: center;
  background: rgba(248, 250, 252, 0.8); border-radius: 12px; backdrop-filter: blur(4px);
}
.lock-content {
  text-align: center; background: white; padding: 3rem; border-radius: 16px;
  box-shadow: 0 10px 25px rgba(0,0,0,0.05); border: 1px solid #e2e8f0;
}
.lock-icon { font-size: 4rem; display: block; margin-bottom: 1rem; }
.lock-content h2 { color: #0f172a; margin-bottom: 0.5rem; }
.lock-content p { color: #64748b; font-size: 0.95rem; line-height: 1.6; margin-bottom: 2rem; }
</style>