<script setup>
// 将来接收 progress 和 status 数据
</script>

<template>
  <div id="status-container" style="margin-top: 20px;">
    
    <div id="progress-container" class="progress-container" style="display: none; margin-bottom: 10px;">
      <div class="progress-bar-bg" style="background: #eee; height: 8px; border-radius: 4px; overflow: hidden;">
        <div id="progress-fill" class="progress-bar-fill" style="background: #4caf50; height: 100%; width: 0%; transition: width 0.3s;"></div>
      </div>
      <div id="progress-text" class="progress-text" style="text-align: right; font-size: 12px; color: #666; margin-top: 4px;">0%</div>
    </div>

    <div id="loading-spinner" class="spinner" style="display: none;"></div>
    
    <p id="status">准备就绪，请添加照片。</p>
  </div>
</template>