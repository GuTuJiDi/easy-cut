// src/stores/settings.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useSettingsStore = defineStore('settings', () => {
    const autoSave = ref(true);
    const trashDays = ref(30);

    // 从后端拉取最新设置
    async function fetchSettings() {
        try {
            const res = await invoke<any>('get_app_settings');
            autoSave.value = res.autoSaveEnabled;
            trashDays.value = res.trashRetentionDays;
        } catch (e) {
            console.error("获取设置失败", e);
        }
    }

    // 同步设置到后端
    async function saveSettings(newAutoSave: boolean, newDays: number) {
        autoSave.value = newAutoSave;
        trashDays.value = newDays;
        await invoke('update_app_settings', {
            settings: { autoSaveEnabled: newAutoSave, trashRetentionDays: newDays }
        });
    }

    return { autoSave, trashDays, fetchSettings, saveSettings };
});