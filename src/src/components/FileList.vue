<script setup>
// 将来这里会接收 fileQueue 数据
</script>

<template>
  <div class="batch-controls">
    <label>处理列表 / Processing Queue</label>
    
    <div class="batch-btn-group">
      <button id="add-files-btn" class="secondary-btn">+ 添加文件 (Files)</button>
      <button id="add-folder-btn" class="secondary-btn">+ 添加文件夹 (Folder)</button>
    </div>

    <div id="drop-zone" class="drop-zone">
      <div id="empty-tip">
        <p>拖拽照片到此处</p>
        <p style="font-size: 0.8em; opacity: 0.7;">(Drag & Drop photos here)</p>
      </div>
      <ul id="file-list" class="file-list">
        </ul>
    </div>

    <div class="queue-stats">
      <span id="queue-count">0 张照片</span>
      <button id="clear-list-btn" style="background:none; border:none; color:#777; font-size:inherit; cursor: pointer;">清空列表 (Clear)</button>
    </div>
  </div>
</template>