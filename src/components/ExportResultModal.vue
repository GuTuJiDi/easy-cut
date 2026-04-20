<template>
  <transition name="modal-fade">
    <div class="export-result-overlay" v-if="modelValue" @click.self="close">
      <div class="export-modal-content">

        <div class="modal-header">
          <div class="header-left">
            <div class="success-icon">🎉</div>
            <div class="header-text">
              <h2>导出圆满完成</h2>
              <p class="subtitle">共生成 <strong>{{ parsedLogs.length }}</strong> 个片段</p>
            </div>
          </div>

          <div class="header-right">
            <span v-if="costTime" class="time-badge">
              ⏱️ 极速耗时: <strong>{{ costTime }}</strong> 秒
            </span>
            <button class="icon-btn close-btn" @click="close">✖</button>
          </div>
        </div>

        <div class="modal-body custom-scrollbar">
          <div class="list-container">
            <div class="list-row" v-for="(item, index) in parsedLogs" :key="index">
              <div class="row-left">
                <span class="check-mark">✓</span>
                <span class="row-idx">{{ (index + 1).toString().padStart(2, '0') }}</span>
                <span class="row-filename" :title="item.fullPath">{{ item.filename }}</span>
              </div>

              <div class="row-right">
                <button class="action-btn split-btn" @click="goToWorkflow(item.fullPath, '/split')" title="将此片段导入基础分割">
                  ✂️ 分割
                </button>
                <button class="action-btn marker-btn" @click="goToWorkflow(item.fullPath, '/marker')" title="将此片段导入打轴标记">
                  🏷️ 打轴
                </button>
                <button class="action-btn play-btn" @click="playSegment(item.fullPath)" title="调用系统播放器预览">
                  ▶ 预览
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <div class="footer-path" :title="exportDir">
            📂 保存于: {{ exportDir }}
          </div>
          <div class="footer-actions">
            <button class="secondary-btn" @click="close">关闭</button>
            <button class="primary-btn" @click="openFolder">打开所在文件夹</button>
          </div>
        </div>

      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useRouter } from 'vue-router'; // 🌟 引入路由实例

const props = defineProps<{
  modelValue: boolean;
  logs: string;
  exportDir: string;
  costTime?: string;
}>();

const emit = defineEmits(['update:modelValue']);
const router = useRouter(); // 🌟 初始化路由

function close() {
  emit('update:modelValue', false);
}

// 将原本生硬的字符串日志，解析为结构化对象数组
const parsedLogs = computed(() => {
  if (!props.logs) return [];
  return props.logs
      .split('\n')
      .filter(line => line.trim().startsWith('✅ 生成:'))
      .map(line => {
        const fullPath = line.replace('✅ 生成:', '').trim();
        const filename = fullPath.split(/[/\\]/).pop() || fullPath;
        return { fullPath, filename };
      });
});

// 🌟 核心逻辑：工作流无缝跳转
function goToWorkflow(targetPath: string, routePath: string) {
  close(); // 先优雅地关闭当前成功弹窗
  // 携带绝对路径参数，跳转到对应的流水线工位
  router.push({ path: routePath, query: { loadVideo: targetPath } });
}

async function openFolder() {
  if (props.exportDir) {
    try {
      await invoke('open_folder', { path: props.exportDir });
      close();
    } catch (e) {
      alert("无法打开目录");
    }
  }
}

async function playSegment(path: string) {
  try {
    await invoke('open_file', { path });
  } catch (e) {
    console.error("播放失败", e);
  }
}
</script>

<style scoped>
/* 优雅的自定义滚动条 */
.custom-scrollbar::-webkit-scrollbar { width: 6px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #cbd5e1; border-radius: 3px; }
.custom-scrollbar::-webkit-scrollbar-thumb:hover { background: #94a3b8; }

/* 遮罩层使用 Flex 居中 */
.export-result-overlay {
  position: fixed; inset: 0;
  background: rgba(15, 23, 42, 0.4); backdrop-filter: blur(3px);
  z-index: 9999;
  display: flex; justify-content: center; align-items: center;
}

/* 卡片基础样式 */
.export-modal-content {
  background: #ffffff; width: 660px; max-width: 90vw; /* 稍微加宽一点以容纳更多按钮 */
  border-radius: 16px; box-shadow: 0 20px 40px -10px rgba(0, 0, 0, 0.2);
  overflow: hidden; display: flex; flex-direction: column;
}

/* ================= 头部 ================= */
.modal-header {
  padding: 1.5rem 1.5rem 1rem 1.5rem;
  display: flex; justify-content: space-between; align-items: flex-start;
}
.header-left { display: flex; align-items: center; gap: 12px; }
.success-icon { font-size: 2rem; background: #ecfdf5; border: 1px solid #a7f3d0; border-radius: 12px; width: 48px; height: 48px; display: flex; align-items: center; justify-content: center; }
.header-text h2 { margin: 0 0 4px 0; font-size: 1.25rem; color: #111827; font-weight: 800; }
.subtitle { margin: 0; font-size: 0.85rem; color: #64748b; }
.subtitle strong { color: #2563eb; }

.header-right { display: flex; align-items: center; gap: 12px; }
.time-badge { background: #f8fafc; color: #475569; padding: 4px 10px; border-radius: 20px; font-size: 0.8rem; font-weight: 600; border: 1px solid #e2e8f0; display: inline-flex; align-items: center; gap: 4px; }
.time-badge strong { color: #059669; font-size: 0.95rem; font-family: monospace; font-weight: 800; }
.close-btn { background: transparent; border: none; color: #94a3b8; font-size: 1rem; cursor: pointer; transition: 0.2s; padding: 4px; border-radius: 4px; }
.close-btn:hover { background: #f1f5f9; color: #1e293b; }

/* ================= 主体列表 ================= */
.modal-body { padding: 0 1.5rem; max-height: 50vh; overflow-y: auto; margin-bottom: 1rem; }
.list-container { border: 1px solid #e2e8f0; border-radius: 8px; overflow: hidden; }

.list-row { display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; border-bottom: 1px solid #f1f5f9; background: #fdfdfd; transition: 0.2s; }
.list-row:last-child { border-bottom: none; }
.list-row:hover { background: #f8fafc; }

.row-left { display: flex; align-items: center; gap: 10px; flex: 1; min-width: 0; }
.check-mark { color: #10b981; font-weight: bold; }
.row-idx { color: #94a3b8; font-family: monospace; font-size: 0.8rem; font-weight: 600; }
.row-filename { font-family: 'Consolas', monospace; font-size: 0.85rem; color: #334155; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 500; }

/* 🌟 核心升级：按钮组布局与极客风色彩系统 */
.row-right { opacity: 0; transform: translateX(5px); transition: all 0.2s; display: flex; gap: 6px; }
.list-row:hover .row-right { opacity: 1; transform: translateX(0); }
.action-btn { background: white; border: 1px solid #cbd5e1; color: #475569; padding: 4px 10px; border-radius: 6px; font-size: 0.75rem; font-weight: 600; cursor: pointer; transition: 0.2s; display: flex; align-items: center; gap: 4px;}

/* 专属动作 Hover 色系 */
.split-btn:hover { border-color: #8b5cf6; color: #7c3aed; background: #f5f3ff; }
.marker-btn:hover { border-color: #f59e0b; color: #d97706; background: #fffbeb; }
.play-btn:hover { border-color: #10b981; color: #059669; background: #ecfdf5; }

/* ================= 底部 ================= */
.modal-footer { padding: 1rem 1.5rem; background: #f8fafc; border-top: 1px solid #e5e7eb; display: flex; justify-content: space-between; align-items: center; }
.footer-path { font-size: 0.75rem; color: #64748b; font-family: monospace; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 350px; }
.footer-actions { display: flex; gap: 10px; }

.primary-btn, .secondary-btn { padding: 0.5rem 1rem; border-radius: 8px; font-weight: 600; font-size: 0.9rem; cursor: pointer; transition: 0.2s; border: none; }
.primary-btn { background: #2563eb; color: white; }
.primary-btn:hover { background: #1d4ed8; }
.secondary-btn { background: white; color: #334155; border: 1px solid #cbd5e1; }
.secondary-btn:hover { background: #f1f5f9; }

/* ================= 🌟 动画 ================= */
.modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 0.3s ease; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }

.modal-fade-enter-active .export-modal-content { animation: modalPopIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275); }
.modal-fade-leave-active .export-modal-content { animation: modalPopOut 0.25s ease forwards; }

@keyframes modalPopIn {
  0% { opacity: 0; transform: scale(0.9) translateY(20px); }
  100% { opacity: 1; transform: scale(1) translateY(0); }
}
@keyframes modalPopOut {
  0% { opacity: 1; transform: scale(1) translateY(0); }
  100% { opacity: 0; transform: scale(0.95) translateY(10px); }
}
</style>