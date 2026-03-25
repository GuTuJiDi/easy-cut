// src/router/index.ts
import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router';
import { appModules } from '../config/modules';

// 1. 动态生成功能路由
const dynamicRoutes: RouteRecordRaw[] = appModules.map(mod => ({
    path: mod.path,
    name: mod.id,
    component: mod.component,
}));

// 2. 组合最终路由 (默认重定向到注册表的第一个有效功能)
const routes: RouteRecordRaw[] = [
    {
        path: '/',
        redirect: appModules.find(m => !m.disabled)?.path || '/split'
    },
    ...dynamicRoutes
];

export const router = createRouter({
    history: createWebHashHistory(),
    routes,
});

// 全局路由守卫 (商业化拦截预留处)
router.beforeEach((to, from, next) => {
    // 例如：在这里可以通过 pinia 判断，如果 to 对应的模块是 isPro 且用户未付费，则弹窗拦截
    next();
});