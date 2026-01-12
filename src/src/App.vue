<script setup>
import { onMounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

// 引入组件
import ResourcePanel from './components/layouts/ResourcePanel.vue';
import PresetPanel from './components/layouts/PresetPanel.vue';
import WorkspacePanel from './components/layouts/WorkspacePanel.vue';
import StatusBar from './components/layouts/StatusBar.vue';
import WindowHeader from './components/layouts/WindowHeader.vue';

// 引入全局逻辑
import { useGlobalEvents } from './composables/useGlobalEvents';

const appWindow = getCurrentWindow();

onMounted(() => {
  // 阻止默认行为，让应用感觉像原生软件
  document.addEventListener('dragstart', (e) => e.preventDefault());
  document.addEventListener('contextmenu', (e) => e.preventDefault());
});

useGlobalEvents();
</script>

<template>
  <div class="app-layout">
    
    <WindowHeader />

    <div class="content-wrapper">
      
      <div class="main-viewport">
        <aside class="panel col-1">
          <ResourcePanel />
        </aside>

        <aside class="panel col-2">
          <PresetPanel />
        </aside>

        <section class="panel col-3">
          <WorkspacePanel />
        </section>
      </div>

      <footer class="bottom-bar">
        <StatusBar />
      </footer>
      
    </div>

  </div>
</template>

<style scoped>
/* 🟢 1. 窗口实体 (The Window Body) */
.app-layout {
  width: 100vw;
  height: 100vh;
  
  /* 关键：从 style.css 读取背景色 */
  /* 因为 html/body 是透明的，这里必须上色，否则窗口是透明的 */
  background-color: var(--bg-color); 
  color: var(--text-main);
  
  display: flex;
  flex-direction: column;
  padding: 0; 
  
  /* 关键：从 style.css 读取圆角 (12px) */
  /* 这决定了你整个 APP 窗口的圆润程度 */
  border-radius: var(--app-radius);
  
  /* 关键：裁切溢出，确保窗口四个角是圆的，不会有直角内容漏出来 */
  overflow: hidden;
  
  /* 可选：加一个极细的边框，增强窗口在深色壁纸上的轮廓感 */
  /* border: 1px solid rgba(255, 255, 255, 0.08); */
  /* box-shadow: 0 0 30px rgba(0, 0, 0, 0.5);  */
  
  border: 1px solid rgba(255, 255, 255, 0.05); 
}

/* 🟢 2. 内容布局层 */
.content-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  
  /* 这里的 padding 控制内容距离窗口边缘的间距 */
  /* 上0 (因为有标题栏)，左右下 12px */
  padding: 0 12px 12px 12px;
  gap: 12px;
}

/* 🟢 3. 主视口 (中间黑色的工作区) */
/* 🟢 3. 主视口 (The Giant Glass Slab) - 核心改造 */
.main-viewport {
  flex: 1; 
  min-height: 0; 
  
  display: grid;
  grid-template-columns: 280px 220px minmax(0, 1fr);
  
  /* --- A. 玻璃材质 --- */
  background: rgba(20, 20, 20, 0.25);
  
  /* 保持强力磨砂 */
  backdrop-filter: blur(24px) saturate(120%);
  -webkit-backdrop-filter: blur(24px) saturate(120%);
  
  /* --- B. 形状 --- */
  border-radius: 16px; 
  overflow: hidden;
  border: none; 

  /* --- C. 光影雕刻 (边界强化重点) --- */
  box-shadow: 
    /* 1. [增强] 外部深色切割线：几乎纯黑，将玻璃从背景中彻底剥离 */
    0 0 0 1px rgba(0, 0, 0, 0.8),
    
    /* 2. [增强] 内部轮廓光：让整圈边缘都有清晰的界限 (0.08 -> 0.15) */
    inset 0 0 0 1px rgba(255, 255, 255, 0.15),
    
    /* 3. [爆发] 顶部锐利棱镜高光：这是质感的关键 (0.2 -> 0.5) */
    /* 这会让玻璃看起来像是有倒角的厚玻璃 */
    inset 0 1px 0 0 rgba(255, 255, 255, 0.5),
    
    /* 4. [加深] 底部厚度感：加深底部阴影，增加沉稳感 */
    inset 0 -1px 0 0 rgba(0, 0, 0, 0.6),
    
    /* 5. [补充] 内部体积光：让中心稍微亮一点，反衬边缘的黑 */
    inset 0 0 40px rgba(255, 255, 255, 0.02),

    /* 6. [加深] 悬浮投影：让它浮起来 */
    0 20px 50px -10px rgba(0, 0, 0, 0.7);

  position: relative;
  z-index: 10; /* 确保层级 */
}

/* --- D. 噪点纹理 (增加高级感) --- */
.main-viewport::before {
  content: "";
  position: absolute; inset: 0;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.6' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)' opacity='0.06'/%3E%3C/svg%3E");
  opacity: 0.4;
  mix-blend-mode: overlay;
  pointer-events: none;
  z-index: 0; /* 在最底层 */
}

/* 面板通用样式 */
.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
}

/* 🟢 4. 子面板 (必须透明化！) */
/* 以前这里是实色背景，现在必须去掉，否则会挡住 main-viewport 的玻璃效果 */

.col-1 { 
  background-color: transparent; 
  /* 右侧分割线：用半透明线替代 */
  border-right: 1px solid rgba(255, 255, 255, 0.05); 
  border-bottom: 1px solid rgba(255, 255, 255, 0.05); 
}

.col-2 { 
  background-color: transparent; 
  /* 如果觉得中间栏需要稍微深一点以区分，可以用极低透明度的黑 */
  background: rgba(0, 0, 0, 0.15); 
  border-bottom: 1px solid rgba(255, 255, 255, 0.05); 
  border-right: 1px solid rgba(255, 255, 255, 0.05); 
}

.col-3 { 
  background-color: transparent; 
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  border-right: 1px solid rgba(255, 255, 255, 0.05);  
}

/* 🟢 5. 底部栏容器 (保持透明占位) */
.bottom-bar {
  height: 55px; 
  flex-shrink: 0; 
  background: transparent;
  border: none;
  padding: 0; 
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>