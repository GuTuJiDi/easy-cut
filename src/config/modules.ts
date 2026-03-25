// src/config/modules.ts
import type { RouteComponent } from 'vue-router';

// 1. 定义模块的标准化接口 (TypeScript 带来极强的类型提示)
export interface AppModule {
    id: string;          // 唯一标识符，也是路由的 name
    name: string;        // 左侧菜单显示的名称
    icon: string;        // 菜单图标 (可以是 emoji 或 SVG 类名)
    path: string;        // 路由路径
    component: () => Promise<RouteComponent>; // 懒加载的 Vue 组件
    isPro?: boolean;     // 【商业化预埋】是否为付费解锁功能
    disabled?: boolean;  // 是否在开发中 (置灰不可点击)
    group?: 'main' | 'settings'; // 所属分组：主功能区 or 底部设置区
}

// 2. 核心大字典：易剪的模块注册表 (以后新增功能，只改这里！)
export const appModules: AppModule[] = [
    {
        id: 'split',
        name: '基础无损分割',
        icon: '🔪',
        path: '/split',
        component: () => import('../views/SplitView.vue'),
        group: 'main'
    },
    {
        id: 'marker',
        name: '智能打轴标记',
        icon: '🏷️',
        path: '/marker',
        // 注意：这里我们将原先庞大的页面降级为组件，用外壳包裹，方便未来接入 AI
        component: () => import('../views/MarkerWrapperView.vue'),
        group: 'main'
    },
    {
        id: 'audio',
        name: '音频极速分离',
        icon: '🎵',
        path: '/audio',
        component: () => import('../views/AudioView.vue'), // 占位
        isPro: true,
        group: 'main'
    },
    /* --- 👇 未来新功能的热插拔演示 👇 --- */
    {
        id: 'smart-split',
        name: '视频格式转换',
        icon: '✨',
        path: '/smart-split',
        component: () => import('../views/TransformFormat.vue'),
        isPro: true,
        group: 'main'
    },
    /* --- 👆 取消注释即可直接生效 👆 --- */
    {
        id: 'settings',
        name: '全局偏好设置',
        icon: '⚙️',
        path: '/settings',
        component: () => import('../views/SettingsView.vue'), // 占位
        group: 'settings'
    }
];