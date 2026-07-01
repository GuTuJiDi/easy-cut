import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 注意：第三个参数 { persist: true } 是魔法生效的关键
export const useSettingsStore = defineStore('settings', () => {
    const autoSave = ref(true);
    const trashDays = ref(30);
    const maxConcurrentTasks = ref(4); // 默认 4 线程

    // 仅从后端拉取后端关心的设置
    async function fetchSettings() {
        try {
            const res = await invoke<any>('get_app_settings');
            autoSave.value = res.autoSaveEnabled;
            trashDays.value = res.trashRetentionDays;
            // 并发数不需要拉取，插件会自动从本地缓存中恢复它
        } catch (e) {
            console.error("获取设置失败", e);
        }
    }

    async function saveSettings(newAutoSave: boolean, newDays: number, newTasks: number) {
        autoSave.value = newAutoSave;
        trashDays.value = newDays;
        maxConcurrentTasks.value = newTasks;

        // 插件会自动把上面的变量保存到本地。我们只需把后端关心的部分发给 Rust。
        await invoke('update_app_settings', {
            settings: { autoSaveEnabled: newAutoSave, trashRetentionDays: newDays }
        });
    }

    return { autoSave, trashDays, maxConcurrentTasks, fetchSettings, saveSettings };
}, {
    persist: true // 🌟 魔法：一键开启持久化！
});