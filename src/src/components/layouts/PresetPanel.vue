<script setup>
import { store } from '../../store/index.js';
import PresetSkeleton from '../common/PresetSkeleton.vue';

// 图片错误处理
const handleImgError = (e) => {
  e.target.style.backgroundColor = '#333';
  e.target.alt = "图片丢失";
};
</script>

<template>
  <div class="panel-header">
    <span>🎨 样式选择 ({{ store.settings.style }})</span>
  </div>

  <div class="panel-body">

    <div v-if="store.isLoadingPresets" class="skeleton-list">
      <PresetSkeleton v-for="n in 3" :key="n" />
    </div>
    
    <div v-else-if="store.currentPresets.length === 0" class="empty-state">
      <div class="emoji">😕</div>
      <div>该分类下暂无预设</div>
    </div>

    <div v-else class="preset-list">
      <div 
        v-for="preset in store.currentPresets" 
        :key="preset.id"
        class="preset-card"
        :class="{ active: store.activePresetId === preset.id }"
        @click="store.applyPreset(preset.id)"
      >
        <div class="img-wrapper">
          <img 
            :src="preset.img" 
            class="preset-img" 
            loading="lazy" 
            @error="handleImgError"
          />
          <div class="active-overlay" v-if="store.activePresetId === preset.id">
            <div class="check-icon">✓</div>
          </div>
        </div>

        <div class="info-wrapper">
          <div class="title">{{ preset.name }}</div>
          <div class="desc">{{ preset.desc }}</div>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
/* 样式保持不变 */
.panel-header {
  height: 40px; display: flex; align-items: center; padding: 0 16px;
  background: transparent; 
  border-bottom: 1px solid var(--border-color);
  font-weight: 600; font-size: 0.9em; 
  color: var(--text-sub); 
  flex-shrink: 0; 
}
.panel-body { flex: 1; min-height: 0; overflow-y: auto; padding: 12px; background-color: transparent;}
.skeleton-list, .preset-list { display: flex; flex-direction: column; gap: 12px; }

.preset-card {
  background-color: var(--card-bg); 
  border: 1px solid var(--border-color); 
  border-radius: 8px;
  overflow: hidden; cursor: pointer; transition: all 0.2s ease;
  position: relative; flex-shrink: 0; 
}
.preset-card:hover { 
  border-color: var(--text-sub); 
  transform: translateY(-2px); 
  box-shadow: 0 4px 12px rgba(0,0,0,0.1); 
}
.preset-card.active { 
  border-color: var(--nikon-yellow); 
  background-color: var(--input-bg); 
}

.img-wrapper { 
  width: 100%; height: auto; 
  background: var(--bg-color); 
  position: relative; overflow: hidden; 
  border-bottom: 1px solid var(--border-color); 
  min-height: 80px; 
}
.preset-img { width: 100%; height: auto; display: block; transition: transform 0.4s ease; }
.preset-card:hover .preset-img { transform: scale(1.05); }

.active-overlay { position: absolute; top: 0; left: 0; right: 0; bottom: 0; background: rgba(255, 225, 0, 0.1); display: flex; align-items: center; justify-content: center; }
.check-icon { background: var(--nikon-yellow); color: #000; width: 24px; height: 24px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: bold; font-size: 14px; box-shadow: 0 2px 5px rgba(0,0,0,0.3); }

.info-wrapper { padding: 10px 12px; }
.title { font-weight: 600; font-size: 0.9em; color: var(--text-main); margin-bottom: 4px; }
.desc { font-size: 0.75em; color: var(--text-sub); }

.empty-state { margin-top: 40px; text-align: center; color: var(--text-sub); font-size: 0.9em; }
.emoji { font-size: 2em; margin-bottom: 10px; }
</style>