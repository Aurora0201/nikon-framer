<script setup>
</script>

<template>
  <div id="modal" class="modal" style="display: none;">
    <div class="preview-container" style="position:fixed; top:0; left:0; width:100%; height:100%; background:rgba(0,0,0,0.8); display:flex; flex-direction:column; justify-content:center; align-items:center; z-index:1000;">
      <img id="preview-img" src="" alt="Preview" style="max-width:90%; max-height:80%; box-shadow: 0 0 20px rgba(0,0,0,0.5);">
      <button id="close-modal" style="margin-top:15px; padding: 8px 20px; cursor:pointer;">关闭预览 (Close)</button>
    </div>
  </div>
</template>