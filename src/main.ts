// src/main.ts
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { router } from './router';
import App from './App.vue';
import './assets/global.css'; // 🌟 核心：引入全局设计系统

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount('#app');