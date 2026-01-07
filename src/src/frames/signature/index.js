// src/frames/signature/index.js
import { defineAsyncComponent, markRaw } from 'vue'; // 🟢 引入这个
import { config } from './config';

// 🟢 使用 markRaw 包裹组件定义
// 这相当于给雕塑贴个条子："我是死物，保安请忽略我"
// 这样 Vue 就会跳过它，不再把它变成 Proxy，警告消除，性能提升。
const Layer = markRaw(defineAsyncComponent(() => import('./Layer.vue')));
const Panel = markRaw(defineAsyncComponent(() => import('./Panel.vue')));

export default {
  ...config,
  layerComponent: Layer,
  panelComponent: Panel
};