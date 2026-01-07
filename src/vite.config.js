import { defineConfig, searchForWorkspaceRoot } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from "path"; // 🟢 需要引入 path 模块

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      // 🟢 新增别名：将 "@fonts" 指向 Rust 的资源目录
      "@fonts": path.resolve(__dirname, "../src-tauri/assets/fonts"),
    },
  },
  // 🟢 允许 Vite 访问 src-tauri 目录 (这是安全限制，必须显式开启)
  server: {
    // 🟢 解决 403 Forbidden 的核心配置
    fs: {
      // 方式 1: 简单粗暴，允许为 Vite 服务的根目录的上级目录提供服务
      // 这通常能覆盖 src-tauri
      allow: [
        // 自动搜索工作区根目录 (推荐)
        searchForWorkspaceRoot(process.cwd()),
        // 显式添加 src-tauri 目录，双重保险
        path.resolve(__dirname, "../src-tauri"),
      ],
      
      // 方式 2 (如果上面还是不行): 关闭严格模式 (仅用于调试，不推荐长期使用，但能立刻验证问题)
      // strict: false, 
    },
  },
});
