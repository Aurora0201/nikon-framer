<script setup>
defineProps({
  activePresetId: String
});
</script>

<template>
  <div class="controls-area">
    <div class="control-row" v-if="activePresetId">
        <label style="color: #666; font-size: 0.75em;">
            当前模式: {{ activePresetId }}
        </label>
    </div>
    <div class="control-row" v-else>
       <label style="color: #444; font-size: 0.75em;">暂无参数配置</label>
    </div>
  </div>
</template>

<style scoped>
.controls-area {
  height: 100px;
  background: #111;
  border-top: 1px solid #333;
  padding: 20px;
}
.control-row { margin-bottom: 15px; }
label { display: block; color: #888; font-size: 0.85em; margin-bottom: 8px; }
</style>