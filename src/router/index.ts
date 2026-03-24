// src/router/index.ts
import { createRouter, createWebHashHistory } from 'vue-router';

// 我们先预先规划好这些视图，即便有的文件还没写
const routes = [
    { path: '/', redirect: '/split' }, // 默认打开无损分割页
    {
        path: '/split',
        name: 'Split',
        component: () => import('../views/SplitView.vue') // 懒加载
    },
    {
        path: '/marker',
        name: 'Marker',
        component: () => import('../views/MarkerView.vue')
    },
    {
        path: '/audio',
        name: 'Audio',
        component: () => import('../views/AudioView.vue')
    },
    {
        path: '/settings',
        name: 'Settings',
        component: () => import('../views/SettingsView.vue')
    }
];

export const router = createRouter({
    history: createWebHashHistory(),
    routes,
});