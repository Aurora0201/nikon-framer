<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {thumbnailLoader} from '../../composables/thumbnailManager.js'

const props = defineProps({
  path: { type: String, required: true }
});

const imgUrl = ref(null);
const isVisible = ref(false); // 是否进入视口
const elRef = ref(null);
const isHovering = ref(false);
const mousePos = ref({ x: 0, y: 0 });

let observer = null;

onMounted(() => {
  observer = new IntersectionObserver((entries) => {
    const entry = entries[0];
    
    if (entry.isIntersecting) {
      // A. 进入视口：请求加载
      isVisible.value = true;
      
      if (!imgUrl.value) {
        thumbnailLoader.add(
          props.path,
          // 成功回调
          (base64Str) => {
            // 这里已经是异步回调了，检查一下组件是否还在 (防止内存泄漏)
            if (!elRef.value) return; 
            // 🟢 直接赋值，不需要 createObjectURL 了
            imgUrl.value = base64Str;
            observer.disconnect();
          },
          // 失败回调
          (err) => { /* console.warn(err) */ }
        );
      }
    } else {
      // B. 🟢 离开视口：取消加载
      // 如果用户滚得太快，这张图还没来得及发给 Rust 就被划走了，
      // 这里会把它从队列里删掉，极大地节省资源。
      if (!imgUrl.value) {
        thumbnailLoader.remove(props.path);
      }
    }
  }, { 
    rootMargin: '100px', // 预加载范围
    threshold: 0.1       // 出现 10% 就算进入
  });
  
  if (elRef.value) observer.observe(elRef.value);
});

onUnmounted(() => {
  if (observer) observer.disconnect();
  // 组件销毁时，也尝试从队列移除（双重保险）
  thumbnailLoader.remove(props.path);
  
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
      decoding="async"
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