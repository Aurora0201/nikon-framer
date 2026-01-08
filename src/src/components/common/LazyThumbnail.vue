<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  path: { type: String, required: true }
});

const imgUrl = ref(null);
const isVisible = ref(false); // 是否进入视口
const elRef = ref(null);
const isHovering = ref(false);
const mousePos = ref({ x: 0, y: 0 });

let observer = null;

// 🟢 1. 生成缩略图的核心逻辑
const loadThumbnail = async () => {
  if (imgUrl.value) return; // 已加载过

  try {
    // 调用我们刚写的 Rust 新接口 (返回 200px 的 jpeg bytes)
    const bytes = await invoke('generate_thumbnail', { filePath: props.path });
    const blob = new Blob([new Uint8Array(bytes)], { type: 'image/jpeg' });
    imgUrl.value = URL.createObjectURL(blob);
  } catch (err) {
    // 失败静默处理，显示占位符即可
    // console.warn("Thumb failed:", err);
  }
};

// 🟢 2. 懒加载观察者
onMounted(() => {
  observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting) {
      isVisible.value = true;
      loadThumbnail();
      observer.disconnect(); // 加载一次后就断开，省资源
    }
  }, { 
    rootMargin: '100px' // 提前 100px 加载，体验更流畅
  });
  
  if (elRef.value) observer.observe(elRef.value);
});

onUnmounted(() => {
  if (observer) observer.disconnect();
  // 🟢 务必释放内存！
  if (imgUrl.value) URL.revokeObjectURL(imgUrl.value);
});

// 🟢 3. 鼠标追踪 (用于悬停显示位置)
const onMouseMove = (e) => {
  if (!isHovering.value) return;
  // 让预览图稍微偏移一点，别挡住鼠标
  mousePos.value = { x: e.clientX + 15, y: e.clientY + 15 };
};
</script>

<template>
  <div 
    class="thumb-wrapper" 
    ref="elRef"
    @mouseenter="isHovering = true"
    @mouseleave="isHovering = false"
    @mousemove="onMouseMove"
  >
    <img 
      v-if="imgUrl" 
      :src="imgUrl" 
      class="thumb-img" 
      loading="lazy" 
      alt="thumb" 
      draggable="false"
    />
    
    <div v-else class="thumb-placeholder">
      <span v-if="isVisible" class="loading-dot"></span>
      <span v-else>📷</span>
    </div>

    <Teleport to="body">
      <div 
        v-if="isHovering && imgUrl" 
        class="hover-preview-popover"
        :style="{ top: mousePos.y + 'px', left: mousePos.x + 'px' }"
      >
        <img :src="imgUrl" class="popover-img" />
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.thumb-wrapper {
  width: 44px; /* 固定列表内尺寸 */
  height: 44px;
  border-radius: 4px;
  overflow: hidden;
  background: #222;
  border: 1px solid #333;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  cursor: help; /* 提示用户可以查看详情 */
}

.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover; /* 关键：填满小方块 */
  display: block;
}

.thumb-placeholder {
  color: #444;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%; height: 100%;
}

.loading-dot {
  width: 6px; height: 6px; background: #666; border-radius: 50%;
  animation: pulse 1s infinite;
}

@keyframes pulse { 0% { opacity: 0.3; } 50% { opacity: 1; } 100% { opacity: 0.3; } }
</style>

<style>
/* 悬浮预览大图 (全局样式) */
.hover-preview-popover {
  position: fixed;
  z-index: 9999;
  
  /* 🟢 修改 1: 移除固定宽高，改为 max 限制 */
  width: auto;
  height: auto;
  max-width: 300px;  /* 限制最大宽度，防止横图太大 */
  max-height: 300px; /* 限制最大高度，防止竖图超出屏幕 */
  
  background: #1a1a1a;
  border: 2px solid var(--nikon-yellow);
  border-radius: 6px;
  box-shadow: 0 10px 30px rgba(0,0,0,0.8);
  
  /* 🟢 修改 2: 让容器紧贴图片大小，不留黑边 */
  display: flex;     
  align-items: center;
  justify-content: center;
  
  overflow: hidden;
  pointer-events: none;
  
  /* 动画保持不变 */
  animation: pop-in 0.15s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.popover-img {
  /* 🟢 修改 3: 图片自适应，不再强制拉伸裁切 */
  display: block;
  width: auto;
  height: auto;
  max-width: 100%;
  max-height: 100%;
  
  /* 移除 object-fit: cover */
}

@keyframes pop-in {
  from { transform: scale(0.8); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}
</style>