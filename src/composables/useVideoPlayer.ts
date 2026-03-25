// src/composables/useVideoPlayer.ts
import { ref, onMounted, onUnmounted } from 'vue';

export function useVideoPlayer() {
    const videoRef = ref<HTMLVideoElement | null>(null);
    const playerContainerRef = ref<HTMLDivElement | null>(null);

    // 基础状态
    const isPlaying = ref(false);
    const currentTime = ref(0);
    const duration = ref(0);
    const playbackRate = ref(1.0);
    const volume = ref(1.0);
    const isMuted = ref(false);
    const isFullscreen = ref(false);

    // OSD 提示系统
    const osdVisible = ref(false);
    const osdText = ref('');
    let osdTimer: number | null = null;

    function triggerOSD(msg: string) {
        osdText.value = msg; osdVisible.value = true;
        if (osdTimer) clearTimeout(osdTimer);
        osdTimer = window.setTimeout(() => { osdVisible.value = false; }, 1500);
    }

    // 核心控制方法
    function togglePlay() {
        if (!videoRef.value) return;
        if (videoRef.value.paused) { videoRef.value.play(); triggerOSD("▶ 播放"); }
        else { videoRef.value.pause(); triggerOSD("⏸ 暂停"); }
    }

    function seekTo(time: number) {
        if (videoRef.value) {
            const safeTime = Math.max(0, Math.min(time, duration.value));
            videoRef.value.currentTime = safeTime;
            currentTime.value = safeTime;
            // resetHideTimer();
        }
    }

    function setSpeed(spd: number) {
        playbackRate.value = spd;
        if (videoRef.value) videoRef.value.playbackRate = spd;
        triggerOSD(`倍速: ${spd.toFixed(2)}x`);
    }

    function toggleMute() {
        if (!videoRef.value) return;
        videoRef.value.muted = !videoRef.value.muted;
        isMuted.value = videoRef.value.muted;
        if (!isMuted.value && volume.value === 0) { volume.value = 0.5; videoRef.value.volume = 0.5; }
        triggerOSD(isMuted.value ? "🔇 静音" : `🔊 音量: ${Math.round(volume.value * 100)}%`);
    }

    function toggleFullscreen() {
        if (!document.fullscreenElement) { playerContainerRef.value?.requestFullscreen().catch(e => console.log(e)); }
        else { document.exitFullscreen(); }
    }

    function handleFullscreenChange() {
        isFullscreen.value = !!document.fullscreenElement;
    }

    // 事件绑定生命周期
    onMounted(() => {
        document.addEventListener('fullscreenchange', handleFullscreenChange);
    });

    onUnmounted(() => {
        document.removeEventListener('fullscreenchange', handleFullscreenChange);
        if (osdTimer) clearTimeout(osdTimer);
    });

    return {
        videoRef, playerContainerRef,
        isPlaying, currentTime, duration, playbackRate, volume, isMuted, isFullscreen,
        osdVisible, osdText, triggerOSD,
        togglePlay, seekTo, setSpeed, toggleMute, toggleFullscreen
    };
}