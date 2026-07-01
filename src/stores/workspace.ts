import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useWorkspaceStore = defineStore('workspace', () => {
    const currentWorkspace = ref<string>('');

    // 初始化时从后端获取真实的工作区路径
    async function initWorkspace() {
        try {
            currentWorkspace.value = await invoke<string>('get_workspace_path');
        } catch (error) {
            console.error("无法获取工作区路径:", error);
        }
    }

    return { currentWorkspace, initWorkspace };
});