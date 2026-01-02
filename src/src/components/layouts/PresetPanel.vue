<script setup>
import { store } from '../../store.js';
// 🟢 1. 引入新写的骨架屏组件
import PresetSkeleton from '../common/PresetSkeleton.vue';

// 🟢 1. 增强版图片加载器 (带调试日志)
const getImageUrl = (filename) => {
  try {
    // 打印尝试解析的路径，按 F12 在 Console 查看
    // console.log(`[Debug] 尝试解析图片: ${filename}`);
    
    // 注意：../../assets/presets/ 必须与你的实际目录结构完全一致
    const url = new URL(`../../assets/presets/${filename}`, import.meta.url).href;
    
    return url;
  } catch (e) {
    console.error(`[Error] 图片路径解析失败: ${filename}`, e);
    return ''; // 返回空字符串防止崩溃
  }
};

// 🟢 2. 图片加载错误处理
const handleImgError = (e, filename) => {
  console.error(`❌ [加载失败] 无法加载图片: ${filename}`);
  console.error(`   -> 浏览器尝试请求的地址: ${e.target.src}`);
  
  // 可选：设置一张兜底的“图片裂开”占位图，或者给个背景色
  e.target.style.backgroundColor = '#333';
  e.target.alt = "图片丢失";
};
</script>

<template>
  <div class="panel-header">
    <span>🎨 效果预设 (Presets)</span>
  </div>

  <div class="panel-body">

    <div v-if="store.isLoadingPresets" class="skeleton-list">
      <PresetSkeleton v-for="n in 3" :key="n" />
    </div>
    
    <div v-else-if="store.currentPresets.length === 0" class="empty-state">
      <div class="emoji">🖼️</div>
      <div>请在左侧选择<br>白底或透明模式</div>
    </div>

    <div v-else class="preset-list">
      <div 
        v-for="preset in store.currentPresets" 
        :key="preset.id"
        class="preset-card"
        :class="{ active: store.activePresetId === preset.id }"
        @click="store.applyPreset(preset)"
      >
        <div class="img-wrapper">
          <img :src="getImageUrl(preset.img)" class="preset-img" loading="lazy" />
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
/* 🟢 完全复用刚才修复滚动条问题的样式 
*/

/* 头部固定高度 */
.panel-header {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  background: #1a1a1a;
  border-bottom: 1px solid #333;
  font-weight: 600;
  font-size: 0.9em;
  color: #ccc;
  
  /* 防止头部被压缩 */
  flex-shrink: 0; 
}

/* 核心滚动区域 */
.panel-body {
  /* 1. 占据剩余空间 */
  flex: 1;
  
  /* 2. 关键：在 Flex 子元素中启用滚动，必须设置 min-height: 0 */
  min-height: 0; 
  
  /* 3. 开启垂直滚动 */
  overflow-y: auto; 
  
  padding: 12px;
}

/* 🟢 新增：专门用于包裹列表的容器，负责间距 */
.skeleton-list,
.preset-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 卡片样式 */
.preset-card {
  background-color: #222;
  border: 1px solid #333;
  border-radius: 6px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  
  /* 关键：禁止卡片被压缩 */
  flex-shrink: 0; 
}

.preset-card:hover {
  border-color: #666;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
}

.preset-card.active {
  border-color: var(--nikon-yellow);
  background-color: #2a2a2a;
}

/* 图片容器 */
.img-wrapper {
  width: 100%;
  height: auto; 
  background: #111;
  position: relative;
  overflow: hidden;
  border-bottom: 1px solid #333;
  min-height: 80px; 
}

.preset-img {
  width: 100%;
  height: auto;
  display: block;
  transition: transform 0.4s ease;
}

.preset-card:hover .preset-img {
  transform: scale(1.05); 
}

/* 选中覆盖层 */
.active-overlay {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(255, 225, 0, 0.1);
  display: flex; align-items: center; justify-content: center;
}

.check-icon {
  background: var(--nikon-yellow);
  color: #000;
  width: 24px; height: 24px;
  border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-weight: bold; font-size: 14px;
  box-shadow: 0 2px 5px rgba(0,0,0,0.3);
}

/* 信息区域 */
.info-wrapper { padding: 10px 12px; }
.title { font-weight: 600; font-size: 0.9em; color: #e0e0e0; margin-bottom: 4px; }
.desc { font-size: 0.75em; color: #777; }

/* 空状态 */
.empty-state {
  margin-top: 40px;
  text-align: center;
  color: #555;
  font-size: 0.9em;
}
.emoji { font-size: 2em; margin-bottom: 10px; }
</style>