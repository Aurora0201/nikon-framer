<script setup>
import { store } from '../../store/index.js';
import { useBatchProcess } from '../../composables/useBatchProcess.js';

// 引入批处理逻辑
const { 
  handleBatchClick, 
  buttonText, 
  buttonClass, 
  buttonCursor,
  canStop 
} = useBatchProcess();

</script>

<template>
  <div class="status-container">
    
    <div class="status-left">
      <span 
        class="indicator" 
        :class="store.statusType"
      >●</span>
      <span class="text" :title="store.statusText">
        {{ store.statusText }}
      </span>
    </div>

    <div class="status-center">
      <div v-if="store.isProcessing" class="progress-box">
        <div class="progress-track">
          <div 
            class="progress-fill" 
            :style="{ width: store.progress.percent + '%' }"
          ></div>
        </div>
        <span class="progress-num">
          {{ store.progress.current }} / {{ store.progress.total }} 
          ({{ store.progress.percent }}%)
        </span>
      </div>
    </div>

    <div class="status-right">
      <button 
        class="action-btn"
        :class="buttonClass"
        :style="{ cursor: buttonCursor }"
        :disabled="store.isProcessing && !canStop"
        @click="handleBatchClick"
      >
        {{ buttonText }}
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 容器布局 */
.status-container {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
  background: #1a1a1a;
  
  /* 🔴 修复：移除 border-top，避免与父容器产生双重边框 */
  border-top: none; 
  
  user-select: none;
}

/* --- 左侧状态 --- */
.status-left {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 0.9em; /* 稍微调大一点，易读 */
  color: #888;
  flex: 1;
  overflow: hidden;
  font-weight: 500;
}

.text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: 0.5px;
}

.indicator { font-size: 10px; transition: color 0.3s; }
.indicator.normal { color: #555; }
.indicator.success { color: #4caf50; text-shadow: 0 0 5px rgba(76, 175, 80, 0.4); }
.indicator.loading { color: var(--nikon-yellow); animation: blink 1s infinite; }
.indicator.error { color: #ff5252; }

@keyframes blink {
  0% { opacity: 1; }
  50% { opacity: 0.3; }
  100% { opacity: 1; }
}

/* --- 中间进度条 --- */
.status-center {
  flex: 2;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 0 20px;
}

.progress-box {
  width: 100%;
  max-width: 450px; /* 稍微加宽 */
  display: flex;
  align-items: center;
  gap: 15px;
}

.progress-track {
  flex: 1;
  /* 🟢 修复：加粗到 8px，视觉更饱满 */
  height: 8px; 
  background: #333;
  border-radius: 4px; /* 圆角对应增加 */
  overflow: hidden;
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.3); /* 增加内阴影，增加槽深感 */
}

.progress-fill {
  height: 100%;
  background: var(--nikon-yellow);
  /* 移除光晕，保持扁平硬朗风格，避免看起来“糊” */
  transition: width 0.2s linear; 
}

.progress-num {
  /* 保持系统字体 + 等宽数字 */
  font-family: inherit;
  font-variant-numeric: tabular-nums;
  
  font-size: 0.85em;
  color: #bbb; /* 稍微亮一点，提高对比度 */
  min-width: 100px;
  text-align: right;
  font-weight: 500;
}

/* --- 右侧按钮 --- */
.status-right {
  display: flex;
  justify-content: flex-end;
  flex: 1;
}

.action-btn {
  background: var(--nikon-yellow, #ffe100);
  color: #111;
  border: none;
  padding: 6px 18px; /* 加大按钮点击区 */
  font-size: 0.85em;
  font-weight: 700;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
  min-width: 130px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  box-shadow: 0 2px 5px rgba(0,0,0,0.2);
}

.action-btn:hover {
  background: #ffeb3b;
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(0,0,0,0.3);
}

.action-btn:active {
  transform: translateY(0);
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
}

.action-btn:disabled,
.action-btn.processing-mode {
  background: #333;
  color: #666;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}

.action-btn.can-stop {
  background: #d32f2f;
  color: white;
}
.action-btn.can-stop:hover {
  background: #f44336;
}
</style>