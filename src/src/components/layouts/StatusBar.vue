<script setup>
import { store } from '../../store/index.js';
import { useBatchProcess } from '../../composables/useBatchProcess.js';

// 逻辑层完全保持原样，不作任何修改
const { 
  handleBatchClick, 
  handleSingleClick, 
  isSingleDisabled, 
  buttonText, 
  buttonClass, 
  buttonCursor,
  canStop 
} = useBatchProcess();
</script>

<template>
  <div class="status-lens-container">
    <div class="status-content">
      
      <div class="status-left">
        <div class="indicator-wrapper">
          <span class="indicator" :class="store.statusType"></span>
          <span class="indicator-glow" :class="store.statusType"></span>
        </div>
        <span class="text" :title="store.statusText">{{ store.statusText }}</span>
      </div>

      <div class="status-center">
        <div v-if="store.isProcessing" class="progress-box">
          <div class="progress-track">
            <div class="progress-fill" :style="{ width: store.progress.percent + '%' }">
              <div class="fill-highlight"></div>
            </div>
          </div>
          <span class="progress-num">
            <span class="num-current">{{ store.progress.current }}</span>
            <span class="num-divider">/</span>
            <span class="num-total">{{ store.progress.total }}</span>
            <span class="num-percent">{{ store.progress.percent }}%</span>
          </span>
        </div>
      </div>

      <div class="status-right">
        <button 
          class="nikon-btn single-mode"
          :disabled="isSingleDisabled"
          @click="handleSingleClick"
          title="仅处理当前选中的图片"
        >
          生成选中
        </button>

        <button 
          class="nikon-btn batch-mode"
          :class="buttonClass"
          :style="{ cursor: buttonCursor }"
          :disabled="store.isProcessing && !canStop"
          @click="handleBatchClick"
        >
          {{ buttonText }}
        </button>
      </div>

    </div>
  </div>
</template>

<style scoped>
/* =========================================
   1. 容器样式 (保持玻璃质感)
   ========================================= */
.status-lens-container {
  width: 100%;
  height: 100%;
  position: relative;
  border-radius: var(--app-radius);
  
  background: rgba(255, 255, 255, 0.015);
  backdrop-filter: blur(16px) saturate(120%);
  -webkit-backdrop-filter: blur(16px) saturate(120%);
  
  box-shadow: 
    0 0 0 1px rgba(0, 0, 0, 0.6),
    inset 0 0 0 1px rgba(255, 255, 255, 0.12),
    inset 0 1px 0 0 rgba(255, 255, 255, 0.35),
    inset 0 -1px 0 0 rgba(0, 0, 0, 0.5),
    inset 0 0 20px rgba(255, 255, 255, 0.02),
    0 12px 30px -8px rgba(0, 0, 0, 0.7);

  border: none;
  user-select: none;
  overflow: hidden;
  z-index: 10;
}

.status-content {
  position: relative;
  z-index: 10;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 20px;
  padding-right: 10px;
}

/* =========================================
   2. 左侧状态 (Status Left)
   ========================================= */
.status-left {
  display: flex; align-items: center; gap: 12px; flex: 1;
  font-size: 0.9em; color: rgba(255, 255, 255, 0.75); font-weight: 500;
  text-shadow: 0 1px 2px rgba(0,0,0,0.9);
}
.indicator-wrapper { position: relative; width: 8px; height: 8px; display: flex; align-items: center; justify-content: center; }
.indicator { width: 8px; height: 8px; border-radius: 50%; background-color: #555; transition: all 0.3s; z-index: 2; box-shadow: 0 1px 2px rgba(0,0,0,0.5); }
.indicator-glow { position: absolute; width: 100%; height: 100%; border-radius: 50%; opacity: 0; transition: all 0.3s; z-index: 1; filter: blur(2px); }

.indicator.normal { background-color: rgba(255, 255, 255, 0.3); }
.indicator.success { background-color: #4caf50; }
.indicator-glow.success { background-color: #4caf50; opacity: 0.6; }
.indicator.loading { background-color: var(--nikon-yellow); }
.indicator-glow.loading { background-color: var(--nikon-yellow); opacity: 0.8; animation: pulse-light 1s infinite; }
.indicator.error { background-color: #ff5252; }
.indicator-glow.error { background-color: #ff5252; opacity: 0.6; }
@keyframes pulse-light { 0% { opacity: 0.4; transform: scale(1); } 100% { opacity: 0; transform: scale(2.5); } }
.text { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; letter-spacing: 0.5px; }

/* =========================================
   3. 中间进度 (Progress Center)
   ========================================= */
.status-center { flex: 2; display: flex; justify-content: center; align-items: center; height: 100%; }
.progress-box { width: 100%; max-width: 420px; display: flex; align-items: center; gap: 16px; }

.progress-track {
  flex: 1; height: 6px; background: rgba(0, 0, 0, 0.5); border-radius: 3px; overflow: hidden;
  box-shadow: inset 0 1px 3px rgba(0,0,0,0.8), 0 1px 0 rgba(255,255,255,0.05); position: relative;
}
.progress-fill {
  height: 100%; width: 0%; background: linear-gradient(90deg, #d4bb00, var(--nikon-yellow));
  transition: width 0.2s linear; position: relative; box-shadow: 0 0 10px rgba(255, 215, 0, 0.3);
}
.fill-highlight { position: absolute; top: 0; left: 0; right: 0; height: 1px; background: rgba(255,255,255,0.6); opacity: 0.5; }

.progress-num {
  font-family: 'Inter Display', sans-serif; font-variant-numeric: tabular-nums; font-size: 0.85em;
  color: rgba(255, 255, 255, 0.6); min-width: 120px; text-align: right; display: flex; justify-content: flex-end; gap: 4px;
  text-shadow: 0 1px 2px rgba(0,0,0,0.8);
}
.num-current { color: #fff; font-weight: 600; }
.num-divider { opacity: 0.4; }
.num-percent { color: var(--nikon-yellow); margin-left: 6px; font-weight: 600; }

/* =========================================
   4. 右侧按钮 (Buttons) - 修正版
   ========================================= */
.status-right {
  display: flex; justify-content: flex-end; align-items: center; flex: 1; gap: 10px;
}

/* 基础按钮样式 */
.nikon-btn {
  height: 34px;
  padding: 0 14px;
  min-width: 80px;
  display: flex; justify-content: center; align-items: center;
  font-size: 0.85em; font-weight: 700; text-transform: uppercase; letter-spacing: 0.5px;
  
  /* 🟢 修复：圆角调整为 6px (适配外层的 12px) */
  border-radius: 8px;
  
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.25, 1, 0.5, 1);
  user-select: none;
  position: relative;
  overflow: hidden;
  box-shadow: 0 2px 5px rgba(0,0,0,0.3);
}

/* --- single-mode (对应之前的次要按钮样式) --- */
.nikon-btn.single-mode {
  background: rgba(255, 255, 255, 0.05);
  color: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 1px 0 rgba(255,255,255,0.1);
}
.nikon-btn.single-mode:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  color: #4b3b00;
  border-color: rgba(255, 255, 255, 0.15);
  /* 🟢 修复：移除了 transform 位移 */
}

/* --- batch-mode (主按钮：透亮黄水晶) --- */
/* --- batch-mode (主按钮：明亮通透的淡色水晶) --- */
/* --- batch-mode (主按钮：纯净奶油金) --- */
/* --- batch-mode (主按钮：鲜艳磨砂黄水晶) --- */
.nikon-btn.batch-mode {
  background: linear-gradient(180deg, #ffe100, #ffc400);
  
  /* 🟢 修改点：不再用纯黑 */
  /* 使用 85% 透明度的黑，让它自然融合 */
  color: rgba(0, 0, 0, 0.85); 
  
  border: none;
  
  /* 🟢 新增：雕刻质感 */
  /* 底部加一条半透明白线，模拟凹陷的高光 */
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.4);
  
  /* 现有的 Shadow 保持不变 */
  box-shadow: 
    inset 0 1px 0 rgba(255, 255, 255, 0.7),
    inset 0 0 10px rgba(255, 255, 255, 0.2),
    inset 0 -1px 0 rgba(180, 130, 0, 0.1),
    0 2px 4px rgba(0, 0, 0, 0.3);
}

.nikon-btn.batch-mode:hover:not(:disabled) {
  filter: brightness(1.05);
  /* 悬停时文字颜色稍微加深一点 */
  color: rgba(0, 0, 0, 0.95);
  box-shadow: 
    inset 0 1px 0 rgba(255, 255, 255, 0.9),
    0 4px 10px rgba(255, 200, 0, 0.3);
}

.nikon-btn.batch-mode:active:not(:disabled) {
  background: #ffc400;
  transform: translateY(1px);
  /* 按下时，文字凹陷感消失，变成普通的内阴影 */
  text-shadow: none;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.2);
  color: #000;
}

/* --- 禁用态 (覆盖所有模式) --- */
.nikon-btn:disabled, 
.nikon-btn.processing-mode {
  background: rgba(255, 255, 255, 0.02) !important;
  color: rgba(255, 255, 255, 0.2) !important;
  border: 1px solid rgba(255, 255, 255, 0.02) !important;
  box-shadow: none !important;
  cursor: not-allowed;
}

/* --- 停止按钮 (高优先级覆盖) --- */
.nikon-btn.can-stop {
  background: linear-gradient(180deg, #ff5252, #d32f2f) !important;
  color: white !important;
  border: none !important;
  box-shadow: inset 0 1px 0 rgba(255,255,255,0.4), 0 2px 4px rgba(0,0,0,0.4) !important;
  animation: pulse-red-btn 2s infinite;
}
.nikon-btn.can-stop:hover { filter: brightness(1.1); }

@keyframes pulse-red-btn { 
  0% { box-shadow: inset 0 1px 0 rgba(255,255,255,0.4), 0 0 0 0 rgba(211, 47, 47, 0.6); } 
  70% { box-shadow: inset 0 1px 0 rgba(255,255,255,0.4), 0 0 0 6px rgba(211, 47, 47, 0); } 
  100% { box-shadow: inset 0 1px 0 rgba(255,255,255,0.4), 0 0 0 0 rgba(211, 47, 47, 0); } 
}
</style>