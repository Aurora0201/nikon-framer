<template>
  <div class="skeleton-card">
    <div class="skeleton-img shimmer"></div>
    
    <div class="skeleton-info">
      <div class="skeleton-title shimmer"></div>
      <div class="skeleton-desc shimmer"></div>
    </div>
  </div>
</template>

<style scoped>
.skeleton-card {
  background-color: #222;
  border: 1px solid #333;
  border-radius: 6px;
  overflow: hidden;
  position: relative;
  /* 保持和真实卡片一样的尺寸感 */
  min-height: 140px; 
}

.skeleton-info {
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 基础形状 */
.skeleton-img {
  width: 100%;
  height: 90px; /* 模拟图片高度 */
  background-color: #2a2a2a;
}

.skeleton-title {
  width: 60%;
  height: 14px;
  background-color: #2a2a2a;
  border-radius: 4px;
}

.skeleton-desc {
  width: 90%;
  height: 10px;
  background-color: #2a2a2a;
  border-radius: 4px;
}

/* 🌟 核心：扫光动画 (Shimmer Effect) */
.shimmer {
  position: relative;
  overflow: hidden;
  background: #2a2a2a;
}

.shimmer::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.05) 50%,
    rgba(255, 255, 255, 0) 100%
  );
  transform: translateX(-100%);
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  100% {
    transform: translateX(100%);
  }
}
</style>