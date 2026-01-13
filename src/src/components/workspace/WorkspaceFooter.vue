<script setup>
import { computed } from 'vue';
import { store } from '../../store/index.js';

defineProps({ activePresetId: String });

// 获取当前模式的面板组件
const activePanelComponent = computed(() => {
  return store.currentModeConfig?.panelComponent;
});

// 仅保留基础逻辑，不添加多余的业务函数
</script>

<template>
  <div class="footer-layout">
    
    <div class="footer-header">
       <div class="header-content" v-if="activePresetId">
          <span class="label">当前模式: {{ store.settings.style || '默认' }}</span>
      </div>
      <div class="header-content" v-else>
         <span class="label">未选择模式</span>
      </div>
    </div>

    <div class="footer-body-scroll">
      <component :is="activePanelComponent" v-if="activePanelComponent" />
    </div>

  </div>
</template>

<style scoped>
/* =========================================
   核心布局逻辑 (Layout Logic Only)
   ========================================= */

/* 1. 根容器 */
.footer-layout {
  height: 100%;          /* 关键：继承 WorkspacePanel 传来的 height */
  display: flex;         /* 启用 Flex 布局 */
  flex-direction: column;/* 垂直排列 */
  background: transparent;   /* 基础背景色，防止透明穿透 */
  overflow: hidden;      /* 防止整体溢出 */
  box-sizing: border-box;
}

/* 2. 顶部和底部区域 (固定不缩放) */
.footer-header,
.footer-actions {
  flex-shrink: 0;        /* 关键：空间不足时，这两块绝对不能被压扁 */
  padding: 10px 20px;
  border-top: 1px solid #333; /* 视觉分隔 */
  border-bottom: 1px solid #333;
}

/* 3. 中间滚动区域 (核心修复) */
.footer-body-scroll {
  flex: 1;               /* 占据 Header 和 Actions 之外的所有空间 */
  overflow-y: auto;      /* 内容溢出时显示垂直滚动条 */
  min-height: 0;         /* 🔥 核心修复：允许 Flex 子项收缩到比内容更小，触发滚动条 */
  
  padding: 15px 20px;    /* 内部间距 */
  position: relative;
  /* background-color: transparent; */
  border-radius: var(--app-radius);
}

/* =========================================
   基础视觉样式 (Minimal Styling)
   ========================================= */
.label {
  color: #888;
  font-size: 0.8em;
}

/* 滚动条微调 (可选，为了不难看) */
.footer-body-scroll::-webkit-scrollbar { width: 6px; }
.footer-body-scroll::-webkit-scrollbar-thumb { background: #444; border-radius: 3px; }
.footer-body-scroll::-webkit-scrollbar-track { background: transparent; }
</style>