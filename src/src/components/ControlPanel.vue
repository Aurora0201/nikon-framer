<script setup>
// 这里以后会通过 defineProps 和 defineEmits 来和父组件通信
</script>

<template>
  <div>
    <label for="style-select">边框样式 / Frame Style</label>
    <select id="style-select">
      <option value="BottomWhite">简约白底 (Bottom White)</option>
      <option value="GaussianBlur">高斯模糊 (Gaussian Blur)</option>
    </select>
  </div>

  <div>
    <label for="font-select">字体文件 / Font (reads from ./fonts)</label>
    <div class="font-row">
      <select id="font-select">
        <option value="default">加载中...</option>
      </select>
      <button id="refresh-fonts-btn" class="icon-btn" title="刷新字体列表">🔄</button>
    </div>
  </div>

  <div>
    <label for="font-weight-select">字体粗细 / Font Weight</label>
    <select id="font-weight-select">
      <option value="Normal">正常 (Normal)</option>
      <option value="Medium">中粗 (Medium)</option>
      <option value="Bold">加粗 (Bold)</option>
      <option value="ExtraBold">特粗 (Extra Bold)</option>
    </select>
  </div>

  <div id="shadow-control-group" style="display: none;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 5px;">
      <label for="shadow-input" style="margin:0;">阴影强度 / Shadow Intensity</label>
      <span id="shadow-val-display" style="font-size: 0.9em; color: #ffe100;">1.0</span>
    </div>
    <input type="range" id="shadow-input" min="0" max="2" step="0.1" value="1.0" style="width: 100%; cursor: pointer;">
  </div>
</template>