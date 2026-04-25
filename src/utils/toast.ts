// src/utils/toast.ts

// 1. 定义支持的提示类型，提供严格的类型约束
export type ToastType = 'info' | 'success' | 'warning' | 'error';

/**
 * 全局 Toast 提示函数
 * @param message 提示信息文本
 * @param type 提示类型，决定颜色 (默认 'info')
 * @param duration 显示时长（毫秒），默认 3000ms
 */
export const showToast = (
    message: string,
    type: ToastType = 'info',
    duration: number = 3000
): void => {
    // 确保在浏览器环境下执行 (防止 SSR 报错)
    if (typeof document === 'undefined') return;

    // 2. 创建 DOM 容器
    const toastEl = document.createElement('div');

    // 3. 基础样式 (配合 CSS 过渡动画)
    toastEl.style.position = 'fixed';
    toastEl.style.top = '20px';
    toastEl.style.left = '50%';
    toastEl.style.transform = 'translateX(-50%)';
    toastEl.style.padding = '10px 20px';
    toastEl.style.borderRadius = '6px';
    toastEl.style.color = '#fff';
    toastEl.style.fontSize = '14px';
    toastEl.style.zIndex = '9999';
    toastEl.style.boxShadow = '0 4px 12px rgba(0,0,0,0.15)';
    toastEl.style.opacity = '0';
    toastEl.style.transition = 'all 0.3s cubic-bezier(0.18, 0.89, 0.32, 1.28)';

    // 增强体验：防止超长报错文本撑爆屏幕
    toastEl.style.maxWidth = '80vw';
    toastEl.style.wordBreak = 'break-word';

    // 4. 根据 type 配置背景颜色
    // 使用 Record 约束对象键值，确保类型安全
    const typeColors: Record<ToastType, string> = {
        info: '#909399',
        success: '#67C23A',
        warning: '#E6A23C',
        error: '#F56C6C'
    };
    toastEl.style.backgroundColor = typeColors[type];

    // 5. 设置内容并挂载到 body
    toastEl.innerText = message;
    document.body.appendChild(toastEl);

    // 6. 触发出现动画
    requestAnimationFrame(() => {
        toastEl.style.opacity = '1';
        toastEl.style.top = '40px'; // 向下平移一点，形成弹出效果
    });

    // 7. 定时销毁
    setTimeout(() => {
        // 触发消失动画
        toastEl.style.opacity = '0';
        toastEl.style.top = '20px';

        // 动画结束后移除 DOM (300ms 需要和 transition 的时间保持一致)
        setTimeout(() => {
            if (document.body.contains(toastEl)) {
                document.body.removeChild(toastEl);
            }
        }, 300);
    }, duration);
};