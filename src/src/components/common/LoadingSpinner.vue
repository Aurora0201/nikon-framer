<script setup>
defineProps({
  text: {
    type: String,
    default: '处理中...'
  },
  // 支持两种模式：overlay (覆盖在内容上) 或 block (独立占据空间)
  mode: {
    type: String,
    default: 'overlay' // 'overlay' | 'block'
  }
});
</script>

<template>
  <div class="loading-container" :class="mode">
    <div class="spinner"></div>
    <div class="loading-text">{{ text }}</div>
  </div>
</template>

<style scoped>
.loading-container {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 12px;
  z-index: 50;
}

/* 覆盖模式：半透明黑色背景，绝对定位 */
.loading-container.overlay {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(2px);
}

/* 区块模式：相对定位，适合列表加载 */
.loading-container.block {
  position: relative;
  padding: 40px 0;
  width: 100%;
}

/* 纯 CSS 动画 Spinner */
.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top-color: var(--nikon-yellow, #ffe100); /* 使用你的全局变量 */
  animation: spin 1s ease-in-out infinite;
}

.loading-text {
  color: #fff;
  font-size: 0.85em;
  font-weight: 500;
  letter-spacing: 1px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>